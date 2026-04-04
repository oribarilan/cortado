use std::{collections::HashMap, sync::Mutex, time::Duration};

use anyhow::Result;

use crate::{
    feed::{
        config::{FeedConfig, FieldOverride},
        field_overrides::{apply_activity_overrides, apply_definition_overrides},
        Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
    },
    terminal_focus,
};

use super::{HarnessProvider, SessionInfo, SessionStatus};

const DEFAULT_INTERVAL_SECONDS: u64 = 30;

/// Resolved terminal focus info for a session, cached per session ID.
#[derive(Debug, Clone)]
struct FocusInfo {
    app_name: String,
    has_tmux: bool,
}

/// Generic feed that delegates session discovery to a `HarnessProvider`.
///
/// Maps provider-agnostic `SessionInfo` into `Activity` for the UI.
/// Adding a new harness requires only a new `HarnessProvider` implementation —
/// zero changes to this struct.
pub struct HarnessFeed {
    name: String,
    provider: Box<dyn HarnessProvider>,
    interval: Duration,
    explicit_overrides: HashMap<String, FieldOverride>,
    config_overrides: HashMap<String, FieldOverride>,
    /// Cached sessions from last poll (for focus_session lookup).
    cached_sessions: Mutex<Vec<SessionInfo>>,
    /// Cached focus info per session ID. Resolved once per session lifetime
    /// (terminal app doesn't change for a given PID tree).
    cached_focus_info: Mutex<HashMap<String, FocusInfo>>,
}

impl HarnessFeed {
    /// Builds a harness feed from parsed feed config and a provider.
    pub fn from_config(config: &FeedConfig, provider: Box<dyn HarnessProvider>) -> Result<Self> {
        Ok(Self {
            name: config.name.clone(),
            provider,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            explicit_overrides: HashMap::new(),
            config_overrides: config.field_overrides.clone(),
            cached_sessions: Mutex::new(Vec::new()),
            cached_focus_info: Mutex::new(HashMap::new()),
        })
    }

    /// Returns a cached `SessionInfo` by session ID (for focus_session lookup).
    #[allow(dead_code)] // Used by focus_session command (task 03).
    pub fn find_session(&self, session_id: &str) -> Option<SessionInfo> {
        self.cached_sessions
            .lock()
            .ok()?
            .iter()
            .find(|s| s.id == session_id)
            .cloned()
    }

    /// Returns any cached session (for capabilities detection).
    #[allow(dead_code)] // Available for future use.
    pub fn any_cached_session(&self) -> Option<SessionInfo> {
        self.cached_sessions.lock().ok()?.first().cloned()
    }

    /// Returns directories to watch for file changes, if the provider supports it.
    pub fn watch_paths(&self) -> Option<Vec<std::path::PathBuf>> {
        self.provider.watch_paths()
    }

    /// Resolves focus info for a session, caching the result.
    /// The PID ancestry walk only runs once per session — the terminal app
    /// doesn't change for a given process tree.
    fn resolve_focus_info(&self, session: &SessionInfo) -> FocusInfo {
        if let Ok(cache) = self.cached_focus_info.lock() {
            if let Some(info) = cache.get(&session.id) {
                return info.clone();
            }
        }

        let info = build_focus_info(session);

        if let Ok(mut cache) = self.cached_focus_info.lock() {
            cache.insert(session.id.clone(), info.clone());
        }

        info
    }

    fn base_field_definitions() -> Vec<FieldDefinition> {
        vec![
            FieldDefinition {
                name: "status".to_string(),
                label: "Status".to_string(),
                field_type: FieldType::Status,
                description: "Session status (working, idle, question, etc)".to_string(),
            },
            FieldDefinition {
                name: "summary".to_string(),
                label: "Summary".to_string(),
                field_type: FieldType::Text,
                description: "Agent-generated session summary".to_string(),
            },
            FieldDefinition {
                name: "last_active".to_string(),
                label: "Last active".to_string(),
                field_type: FieldType::Text,
                description: "Timestamp of last session activity".to_string(),
            },
            FieldDefinition {
                name: "repo".to_string(),
                label: "Repo".to_string(),
                field_type: FieldType::Text,
                description: "Repository name".to_string(),
            },
            FieldDefinition {
                name: "branch".to_string(),
                label: "Branch".to_string(),
                field_type: FieldType::Text,
                description: "Git branch name".to_string(),
            },
            FieldDefinition {
                name: "focus_app".to_string(),
                label: "Terminal".to_string(),
                field_type: FieldType::Text,
                description: "Detected terminal app name".to_string(),
            },
            FieldDefinition {
                name: "focus_has_tmux".to_string(),
                label: "tmux".to_string(),
                field_type: FieldType::Text,
                description: "Whether tmux was detected in process ancestry".to_string(),
            },
        ]
    }
}

#[async_trait::async_trait]
impl Feed for HarnessFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        self.provider.feed_type()
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn retain_for(&self) -> Option<Duration> {
        None
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        apply_definition_overrides(
            Self::base_field_definitions(),
            &self.explicit_overrides,
            &self.config_overrides,
        )
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        let sessions = self.provider.discover_sessions()?;
        let sessions = deduplicate_sessions(sessions);

        // Cache for focus_session lookup.
        if let Ok(mut cache) = self.cached_sessions.lock() {
            *cache = sessions.clone();
        }

        // Prune cached focus info for sessions no longer present.
        if let Ok(mut info_cache) = self.cached_focus_info.lock() {
            let active_ids: std::collections::HashSet<&str> =
                sessions.iter().map(|s| s.id.as_str()).collect();
            info_cache.retain(|id, _| active_ids.contains(id.as_str()));
        }

        let activities = sessions
            .iter()
            .map(|session| {
                let focus_info = self.resolve_focus_info(session);
                session_to_activity(
                    session,
                    &focus_info,
                    &self.explicit_overrides,
                    &self.config_overrides,
                )
            })
            .collect();

        Ok(activities)
    }
}

/// Returns a numeric urgency priority for a session status.
///
/// Higher values = more urgent. Used to surface the most actionable session
/// when multiple sessions share the same working directory (e.g., two coding
/// agent instances in the same repo). Attention-needed statuses (question,
/// approval) take precedence over active work, which takes precedence over idle.
fn status_priority(status: SessionStatus) -> u8 {
    match status {
        SessionStatus::Question | SessionStatus::Approval => 2,
        SessionStatus::Working => 1,
        SessionStatus::Idle | SessionStatus::Unknown => 0,
    }
}

/// Deduplicates sessions by working directory with status-priority selection.
///
/// Multiple sessions can exist for the same cwd (e.g., two OpenCode instances
/// in the same repo). We keep the session with the most urgent status — so
/// "question" or "approval" (attention needed) beats "working", which beats
/// "idle". Ties in status priority are broken by `last_active_at` (most recent
/// wins).
///
/// When dedup collapses multiple sessions into one, the surviving activity
/// gets a stable CWD-derived ID so the UI row doesn't jump when the winner
/// changes between polls.
fn deduplicate_sessions(sessions: Vec<SessionInfo>) -> Vec<SessionInfo> {
    let mut best_by_cwd: HashMap<String, SessionInfo> = HashMap::new();
    let mut had_duplicate: std::collections::HashSet<String> = std::collections::HashSet::new();

    for session in sessions {
        let key = session.cwd.clone();

        match best_by_cwd.get(&key) {
            None => {
                best_by_cwd.insert(key, session);
            }
            Some(existing) => {
                had_duplicate.insert(key.clone());
                let existing_prio = status_priority(existing.status);
                let session_prio = status_priority(session.status);

                let replace = if existing_prio != session_prio {
                    session_prio > existing_prio
                } else {
                    // Same priority — keep the more recently active one.
                    session.last_active_at > existing.last_active_at
                };

                if replace {
                    best_by_cwd.insert(key, session);
                }
            }
        }
    }

    // For CWDs that had duplicates, use a stable CWD-derived ID
    // so the activity row doesn't jump when the winner changes.
    best_by_cwd
        .into_iter()
        .map(|(cwd, mut session)| {
            if had_duplicate.contains(&cwd) {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                cwd.hash(&mut hasher);
                session.id = format!("harness-{:x}", hasher.finish());
            }
            session
        })
        .collect()
}

/// Converts a `SessionInfo` into an `Activity`.
fn session_to_activity(
    session: &SessionInfo,
    focus_info: &FocusInfo,
    explicit_overrides: &HashMap<String, FieldOverride>,
    config_overrides: &HashMap<String, FieldOverride>,
) -> Activity {
    let (status_value, status_kind) = status_to_value_and_kind(session.status);
    let title = format_activity_title(session);

    let mut fields = vec![Field {
        name: "status".to_string(),
        label: "Status".to_string(),
        value: FieldValue::Status {
            value: status_value,
            kind: status_kind,
        },
    }];

    if let Some(summary) = &session.summary {
        fields.push(Field {
            name: "summary".to_string(),
            label: "Summary".to_string(),
            value: FieldValue::Text {
                value: summary.clone(),
            },
        });
    }

    if let Some(last_active) = &session.last_active_at {
        fields.push(Field {
            name: "last_active".to_string(),
            label: "Last active".to_string(),
            value: FieldValue::Text {
                value: format_relative_time(last_active),
            },
        });
    }

    if let Some(repo) = &session.repository {
        fields.push(Field {
            name: "repo".to_string(),
            label: "Repo".to_string(),
            value: FieldValue::Text {
                value: repo.clone(),
            },
        });
    }

    if let Some(branch) = &session.branch {
        fields.push(Field {
            name: "branch".to_string(),
            label: "Branch".to_string(),
            value: FieldValue::Text {
                value: branch.clone(),
            },
        });
    }

    fields.push(Field {
        name: "focus_app".to_string(),
        label: "Terminal".to_string(),
        value: FieldValue::Text {
            value: focus_info.app_name.clone(),
        },
    });

    fields.push(Field {
        name: "focus_has_tmux".to_string(),
        label: "tmux".to_string(),
        value: FieldValue::Text {
            value: if focus_info.has_tmux {
                "yes".to_string()
            } else {
                "no".to_string()
            },
        },
    });

    let fields = apply_activity_overrides(fields, explicit_overrides, config_overrides);

    Activity {
        id: session.id.clone(),
        title,
        fields,
        retained: false,
        retained_at_unix_ms: None,
        sort_ts: parse_iso_to_unix_ms(session.last_active_at.as_deref()),
    }
}

/// Maps `SessionStatus` to (status value string, StatusKind).
fn status_to_value_and_kind(status: SessionStatus) -> (String, StatusKind) {
    match status {
        SessionStatus::Working => ("working".to_string(), StatusKind::Running),
        SessionStatus::Question => ("question".to_string(), StatusKind::AttentionPositive),
        SessionStatus::Approval => ("approval".to_string(), StatusKind::AttentionPositive),
        SessionStatus::Idle => ("idle".to_string(), StatusKind::Idle),
        SessionStatus::Unknown => ("unknown".to_string(), StatusKind::Idle),
    }
}

/// Formats activity title as `{short_repo} @ {branch}`.
///
/// `short_repo` is the repo name without owner (e.g., `cortado` from `oribarilan/cortado`).
/// Falls back to last path component of `cwd` if repo is unknown.
/// Omits `@ {branch}` if branch is unknown.
fn format_activity_title(session: &SessionInfo) -> String {
    let short_name = session
        .repository
        .as_deref()
        .and_then(|repo| repo.rsplit('/').next())
        .unwrap_or_else(|| {
            session
                .cwd
                .rsplit('/')
                .find(|s| !s.is_empty())
                .unwrap_or("session")
        });

    match &session.branch {
        Some(branch) => format!("{short_name} @ {branch}"),
        None => short_name.to_string(),
    }
}

/// Resolves terminal focus info for a session via PID ancestry walk.
fn build_focus_info(session: &SessionInfo) -> FocusInfo {
    let ctx = match terminal_focus::build_context_for_label(session) {
        Some(ctx) => ctx,
        None => {
            return FocusInfo {
                app_name: "terminal".to_string(),
                has_tmux: false,
            }
        }
    };

    FocusInfo {
        app_name: ctx
            .terminal_app_name
            .unwrap_or_else(|| "terminal".to_string()),
        has_tmux: ctx.tmux_server_pid.is_some(),
    }
}

/// Formats an ISO 8601 timestamp as a relative time string (e.g., "2m ago").
fn format_relative_time(iso_timestamp: &str) -> String {
    let Ok(ts) = jiff::Timestamp::from_str(iso_timestamp) else {
        return iso_timestamp.to_string();
    };

    let now = jiff::Timestamp::now();
    let diff = now.since(ts);

    let Ok(span) = diff else {
        return iso_timestamp.to_string();
    };

    let total_secs = span.get_seconds();

    if total_secs < 60 {
        "just now".to_string()
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        format!("{mins}m ago")
    } else if total_secs < 86400 {
        let hours = total_secs / 3600;
        format!("{hours}h ago")
    } else {
        let days = total_secs / 86400;
        format!("{days}d ago")
    }
}

use std::str::FromStr;

/// Parses an ISO 8601 timestamp to unix milliseconds.
fn parse_iso_to_unix_ms(iso: Option<&str>) -> Option<u64> {
    let ts = jiff::Timestamp::from_str(iso?).ok()?;
    Some(ts.as_millisecond() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_mapping_working() {
        let (value, kind) = status_to_value_and_kind(SessionStatus::Working);
        assert_eq!(value, "working");
        assert_eq!(kind, StatusKind::Running);
    }

    #[test]
    fn status_mapping_question() {
        let (value, kind) = status_to_value_and_kind(SessionStatus::Question);
        assert_eq!(value, "question");
        assert_eq!(kind, StatusKind::AttentionPositive);
    }

    #[test]
    fn status_mapping_approval() {
        let (value, kind) = status_to_value_and_kind(SessionStatus::Approval);
        assert_eq!(value, "approval");
        assert_eq!(kind, StatusKind::AttentionPositive);
    }

    #[test]
    fn status_mapping_idle() {
        let (value, kind) = status_to_value_and_kind(SessionStatus::Idle);
        assert_eq!(value, "idle");
        assert_eq!(kind, StatusKind::Idle);
    }

    #[test]
    fn status_mapping_unknown() {
        let (value, kind) = status_to_value_and_kind(SessionStatus::Unknown);
        assert_eq!(value, "unknown");
        assert_eq!(kind, StatusKind::Idle);
    }

    #[test]
    fn title_with_repo_and_branch() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "/home/user/repos/cortado".to_string(),
            repository: Some("oribarilan/cortado".to_string()),
            branch: Some("main".to_string()),
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        assert_eq!(format_activity_title(&session), "cortado @ main");
    }

    #[test]
    fn title_with_repo_no_branch() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "/home/user/repos/cortado".to_string(),
            repository: Some("oribarilan/cortado".to_string()),
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        assert_eq!(format_activity_title(&session), "cortado");
    }

    #[test]
    fn title_no_repo_uses_cwd() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "/home/user/repos/my-project".to_string(),
            repository: None,
            branch: Some("feature".to_string()),
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        assert_eq!(format_activity_title(&session), "my-project @ feature");
    }

    #[test]
    fn title_no_repo_no_branch() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "/home/user/repos/my-project".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        assert_eq!(format_activity_title(&session), "my-project");
    }

    #[test]
    fn title_trailing_slash_in_cwd() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "/home/user/repos/my-project/".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        assert_eq!(format_activity_title(&session), "my-project");
    }

    #[test]
    fn session_to_activity_maps_all_fields() {
        let session = SessionInfo {
            id: "test-id".to_string(),
            cwd: "/tmp/project".to_string(),
            repository: Some("user/repo".to_string()),
            branch: Some("main".to_string()),
            status: SessionStatus::Working,
            pid: 123,
            summary: None,
            last_active_at: None,
        };

        let test_focus = FocusInfo {
            app_name: "TestTerminal".to_string(),
            has_tmux: false,
        };

        let activity = session_to_activity(&session, &test_focus, &HashMap::new(), &HashMap::new());

        assert_eq!(activity.id, "test-id");
        assert_eq!(activity.title, "repo @ main");
        assert!(!activity.retained);

        // Should have status, repo, branch, focus_app, focus_has_tmux fields.
        assert_eq!(activity.fields.len(), 5);

        let status_field = &activity.fields[0];
        assert_eq!(status_field.name, "status");
        match &status_field.value {
            FieldValue::Status { value, kind } => {
                assert_eq!(value, "working");
                assert_eq!(*kind, StatusKind::Running);
            }
            _ => panic!("expected Status field"),
        }
    }

    #[test]
    fn session_to_activity_omits_missing_optional_fields() {
        let session = SessionInfo {
            id: "minimal".to_string(),
            cwd: "/tmp".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Unknown,
            pid: 1,
            summary: None,
            last_active_at: None,
        };

        let test_focus = FocusInfo {
            app_name: "terminal".to_string(),
            has_tmux: false,
        };

        let activity = session_to_activity(&session, &test_focus, &HashMap::new(), &HashMap::new());

        // status + focus_app + focus_has_tmux (no repo/branch/summary/last_active).
        assert_eq!(activity.fields.len(), 3);
        assert_eq!(activity.fields[0].name, "status");
    }

    #[test]
    fn deduplicate_attention_beats_working() {
        let sessions = vec![
            SessionInfo {
                id: "working-session".to_string(),
                cwd: "/home/user/project".to_string(),
                repository: Some("user/project".to_string()),
                branch: Some("main".to_string()),
                status: SessionStatus::Working,
                pid: 1,
                summary: None,
                // More recent — but lower priority status.
                last_active_at: Some("2026-01-02T00:00:00Z".to_string()),
            },
            SessionInfo {
                id: "question-session".to_string(),
                cwd: "/home/user/project".to_string(),
                repository: Some("user/project".to_string()),
                branch: Some("main".to_string()),
                status: SessionStatus::Question,
                pid: 2,
                summary: None,
                last_active_at: Some("2026-01-01T00:00:00Z".to_string()),
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].status, SessionStatus::Question);
    }

    #[test]
    fn deduplicate_approval_beats_working() {
        let sessions = vec![
            SessionInfo {
                id: "working".to_string(),
                cwd: "/tmp/repo".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Working,
                pid: 1,
                summary: None,
                last_active_at: Some("2026-01-02T00:00:00Z".to_string()),
            },
            SessionInfo {
                id: "approval".to_string(),
                cwd: "/tmp/repo".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Approval,
                pid: 2,
                summary: None,
                last_active_at: Some("2026-01-01T00:00:00Z".to_string()),
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].status, SessionStatus::Approval);
    }

    #[test]
    fn deduplicate_working_beats_idle() {
        let sessions = vec![
            SessionInfo {
                id: "idle".to_string(),
                cwd: "/tmp/repo".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Idle,
                pid: 1,
                summary: None,
                last_active_at: Some("2026-01-02T00:00:00Z".to_string()),
            },
            SessionInfo {
                id: "working".to_string(),
                cwd: "/tmp/repo".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Working,
                pid: 2,
                summary: None,
                last_active_at: Some("2026-01-01T00:00:00Z".to_string()),
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].status, SessionStatus::Working);
    }

    #[test]
    fn deduplicate_same_priority_uses_recency() {
        // Both Working — tiebreak by last_active_at.
        let sessions = vec![
            SessionInfo {
                id: "old".to_string(),
                cwd: "/home/user/project".to_string(),
                repository: Some("user/project".to_string()),
                branch: Some("main".to_string()),
                status: SessionStatus::Working,
                pid: 1,
                summary: Some("old session".to_string()),
                last_active_at: Some("2026-01-01T00:00:00Z".to_string()),
            },
            SessionInfo {
                id: "new".to_string(),
                cwd: "/home/user/project".to_string(),
                repository: Some("user/project".to_string()),
                branch: Some("main".to_string()),
                status: SessionStatus::Working,
                pid: 2,
                summary: Some("new session".to_string()),
                last_active_at: Some("2026-01-02T00:00:00Z".to_string()),
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].summary, Some("new session".to_string()));
    }

    #[test]
    fn deduplicate_stable_id_for_duplicates() {
        // When dedup collapses sessions, the surviving activity gets a
        // CWD-derived stable ID (not the original session ID).
        let sessions = vec![
            SessionInfo {
                id: "session-a".to_string(),
                cwd: "/tmp/repo".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Working,
                pid: 1,
                summary: None,
                last_active_at: None,
            },
            SessionInfo {
                id: "session-b".to_string(),
                cwd: "/tmp/repo".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Question,
                pid: 2,
                summary: None,
                last_active_at: None,
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert!(
            deduped[0].id.starts_with("harness-"),
            "expected stable CWD-derived ID, got: {}",
            deduped[0].id
        );
    }

    #[test]
    fn deduplicate_single_session_keeps_original_id() {
        let sessions = vec![SessionInfo {
            id: "my-session-id".to_string(),
            cwd: "/tmp/repo".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        }];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].id, "my-session-id");
    }

    #[test]
    fn deduplicate_keeps_different_cwds() {
        let sessions = vec![
            SessionInfo {
                id: "a".to_string(),
                cwd: "/home/user/project-a".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Idle,
                pid: 1,
                summary: None,
                last_active_at: None,
            },
            SessionInfo {
                id: "b".to_string(),
                cwd: "/home/user/project-b".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Idle,
                pid: 1,
                summary: None,
                last_active_at: None,
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn deduplicate_single_session_unchanged() {
        let sessions = vec![SessionInfo {
            id: "only".to_string(),
            cwd: "/tmp".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        }];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].id, "only");
    }

    #[test]
    fn deduplicate_empty_list() {
        let deduped = deduplicate_sessions(Vec::new());
        assert!(deduped.is_empty());
    }

    #[test]
    fn deduplicate_no_last_active_keeps_last_seen() {
        // Both have None for last_active_at — should keep one deterministically.
        let sessions = vec![
            SessionInfo {
                id: "first".to_string(),
                cwd: "/tmp/same".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Idle,
                pid: 1,
                summary: None,
                last_active_at: None,
            },
            SessionInfo {
                id: "second".to_string(),
                cwd: "/tmp/same".to_string(),
                repository: None,
                branch: None,
                status: SessionStatus::Working,
                pid: 2,
                summary: None,
                last_active_at: None,
            },
        ];

        let deduped = deduplicate_sessions(sessions);
        assert_eq!(deduped.len(), 1);
    }

    #[test]
    fn title_empty_cwd_fallback() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        assert_eq!(format_activity_title(&session), "session");
    }

    #[test]
    fn title_root_cwd() {
        let session = SessionInfo {
            id: "abc".to_string(),
            cwd: "/".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: None,
            last_active_at: None,
        };
        // "/" has no non-empty path component — should fall back to "session".
        assert_eq!(format_activity_title(&session), "session");
    }

    #[test]
    fn relative_time_just_now() {
        let now = jiff::Timestamp::now();
        let iso = now.to_string();
        let result = format_relative_time(&iso);
        assert_eq!(result, "just now");
    }

    #[test]
    fn relative_time_minutes_ago() {
        use jiff::SignedDuration;
        let ts = jiff::Timestamp::now() - SignedDuration::from_mins(5);
        let result = format_relative_time(&ts.to_string());
        assert_eq!(result, "5m ago");
    }

    #[test]
    fn relative_time_hours_ago() {
        use jiff::SignedDuration;
        let ts = jiff::Timestamp::now() - SignedDuration::from_hours(3);
        let result = format_relative_time(&ts.to_string());
        assert_eq!(result, "3h ago");
    }

    #[test]
    fn relative_time_invalid_timestamp() {
        let result = format_relative_time("not-a-timestamp");
        assert_eq!(result, "not-a-timestamp");
    }

    #[test]
    fn parse_iso_to_unix_ms_valid() {
        let ms = parse_iso_to_unix_ms(Some("2026-01-01T00:00:00Z"));
        assert!(ms.is_some());
        assert!(ms.unwrap() > 0);
    }

    #[test]
    fn parse_iso_to_unix_ms_none() {
        assert!(parse_iso_to_unix_ms(None).is_none());
    }

    #[test]
    fn parse_iso_to_unix_ms_invalid() {
        assert!(parse_iso_to_unix_ms(Some("garbage")).is_none());
    }

    #[test]
    fn session_with_summary_includes_field() {
        let session = SessionInfo {
            id: "s".to_string(),
            cwd: "/tmp".to_string(),
            repository: None,
            branch: None,
            status: SessionStatus::Idle,
            pid: 1,
            summary: Some("doing things".to_string()),
            last_active_at: None,
        };

        let focus = FocusInfo {
            app_name: "t".to_string(),
            has_tmux: false,
        };
        let activity = session_to_activity(&session, &focus, &HashMap::new(), &HashMap::new());

        let summary_field = activity.fields.iter().find(|f| f.name == "summary");
        assert!(summary_field.is_some());
        match &summary_field.unwrap().value {
            FieldValue::Text { value } => assert_eq!(value, "doing things"),
            _ => panic!("expected Text field"),
        }
    }
}
