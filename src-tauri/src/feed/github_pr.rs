use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use toml::Value;

use crate::feed::{
    config::{FeedConfig, FieldOverride},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    github_common::{
        ensure_gh_available, looks_like_gh_auth_error, non_zero_exit_context, GH_COMMAND_TIMEOUT,
        GH_UNAUTHENTICATED_MESSAGE,
    },
    process::{CommandInvocation, ProcessRunner, TokioProcessRunner},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 120;
const MAX_ACTIVITIES_PER_FEED: usize = 20;

/// Feed that polls GitHub pull requests via `gh pr list`.
pub struct GithubPrFeed {
    name: String,
    repo: String,
    user: Option<String>,
    interval: Duration,
    retain_for: Option<Duration>,
    config_overrides: HashMap<String, FieldOverride>,
    process_runner: Arc<dyn ProcessRunner>,
}

impl GithubPrFeed {
    /// Builds a GitHub PR feed from parsed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        Self::from_config_with_runner(config, Arc::new(TokioProcessRunner))
    }

    /// Builds a GitHub PR feed with an injected process runner (used by tests).
    pub fn from_config_with_runner(
        config: &FeedConfig,
        process_runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self> {
        let repo = config
            .type_specific
            .get("repo")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type github-pr) is missing required `repo` string",
                    config.name
                )
            })?
            .trim()
            .to_string();

        if repo.is_empty() {
            bail!(
                "feed `{}` (type github-pr) requires non-empty `repo`",
                config.name
            );
        }

        let user = config
            .type_specific
            .get("user")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        Ok(Self {
            name: config.name.clone(),
            repo,
            user,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            config_overrides: config.field_overrides.clone(),
            process_runner,
        })
    }
}

#[async_trait::async_trait]
impl Feed for GithubPrFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "github-pr"
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn retain_for(&self) -> Option<Duration> {
        self.retain_for
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        apply_definition_overrides(
            base_field_definitions(),
            &HashMap::new(),
            &self.config_overrides,
        )
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        ensure_gh_available(self.process_runner.as_ref()).await?;

        let mut args = vec![
            "pr".to_string(),
            "list".to_string(),
            "--repo".to_string(),
            self.repo.clone(),
        ];

        if let Some(ref user) = self.user {
            args.push("--author".to_string());
            args.push(user.clone());
        }

        args.extend([
            "--state".to_string(),
            "open".to_string(),
            "--limit".to_string(),
            "20".to_string(),
            "--json".to_string(),
            "number,title,url,isDraft,labels,mergeable,reviewDecision,statusCheckRollup"
                .to_string(),
        ]);

        let invocation =
            CommandInvocation::new("gh", args.iter().map(String::as_str), GH_COMMAND_TIMEOUT);

        let command_display = invocation.display();
        let output = self
            .process_runner
            .run(invocation)
            .await
            .map_err(|error| anyhow!("failed invoking `{command_display}`: {error}"))?;

        if !output.succeeded() {
            if looks_like_gh_auth_error(&output.stdout, &output.stderr) {
                bail!(GH_UNAUTHENTICATED_MESSAGE);
            }

            bail!(
                "`{command_display}` failed with {}",
                non_zero_exit_context(output.exit_code, &output.stdout, &output.stderr)
            );
        }

        let prs = serde_json::from_str::<Vec<GhPullRequest>>(&output.stdout)
            .map_err(|error| anyhow!("failed parsing `gh pr list` JSON output: {error}"))?;

        let activities = prs
            .into_iter()
            .map(|pr| map_pr_to_activity(pr, &self.config_overrides))
            .take(MAX_ACTIVITIES_PER_FEED)
            .collect();

        Ok(activities)
    }
}

fn base_field_definitions() -> Vec<FieldDefinition> {
    vec![
        FieldDefinition {
            name: "review".to_string(),
            label: "Review".to_string(),
            field_type: FieldType::Status,
            description: "Current review decision".to_string(),
        },
        FieldDefinition {
            name: "checks".to_string(),
            label: "Checks".to_string(),
            field_type: FieldType::Status,
            description: "CI checks state".to_string(),
        },
        FieldDefinition {
            name: "mergeable".to_string(),
            label: "Mergeable".to_string(),
            field_type: FieldType::Status,
            description: "Whether the PR can be merged".to_string(),
        },
        FieldDefinition {
            name: "draft".to_string(),
            label: "Draft".to_string(),
            field_type: FieldType::Status,
            description: "Draft status".to_string(),
        },
        FieldDefinition {
            name: "labels".to_string(),
            label: "Labels".to_string(),
            field_type: FieldType::Text,
            description: "Applied PR labels".to_string(),
        },
    ]
}

fn map_pr_to_activity(
    pr: GhPullRequest,
    config_overrides: &HashMap<String, FieldOverride>,
) -> Activity {
    let review = map_review_decision(pr.review_decision.as_deref());
    let mergeable = map_mergeable_state(pr.mergeable.as_deref());
    let draft = map_draft(pr.is_draft);
    let checks = map_checks_status(pr.status_check_rollup.as_deref());
    let labels = map_labels(pr.labels.as_deref());

    let fields = apply_activity_overrides(
        vec![
            Field {
                name: "review".to_string(),
                label: "Review".to_string(),
                value: review,
            },
            Field {
                name: "checks".to_string(),
                label: "Checks".to_string(),
                value: checks,
            },
            Field {
                name: "mergeable".to_string(),
                label: "Mergeable".to_string(),
                value: mergeable,
            },
            Field {
                name: "draft".to_string(),
                label: "Draft".to_string(),
                value: draft,
            },
            Field {
                name: "labels".to_string(),
                label: "Labels".to_string(),
                value: labels,
            },
        ],
        &HashMap::new(),
        config_overrides,
    );

    Activity {
        id: pr
            .url
            .clone()
            .unwrap_or_else(|| format!("{}/pull/{}", pr.repo_hint(), pr.number)),
        title: format!("#{} {}", pr.number, pr.title),
        fields,
        retained: false,
        retained_at_unix_ms: None,
        sort_ts: None,
        action: None,
    }
}

fn map_review_decision(review_decision: Option<&str>) -> FieldValue {
    match review_decision.unwrap_or_default() {
        "APPROVED" => status_field("approved", StatusKind::AttentionPositive),
        "CHANGES_REQUESTED" => status_field("changes requested", StatusKind::AttentionNegative),
        "REVIEW_REQUIRED" => status_field("awaiting", StatusKind::Waiting),
        _ => status_field("none", StatusKind::Idle),
    }
}

fn map_mergeable_state(mergeable: Option<&str>) -> FieldValue {
    match mergeable.unwrap_or_default() {
        "MERGEABLE" => status_field("yes", StatusKind::Idle),
        "CONFLICTING" => status_field("no", StatusKind::AttentionNegative),
        "UNKNOWN" => status_field("unknown", StatusKind::Running),
        _ => status_field("unknown", StatusKind::Idle),
    }
}

fn map_draft(is_draft: bool) -> FieldValue {
    if is_draft {
        status_field("yes", StatusKind::AttentionPositive)
    } else {
        status_field("no", StatusKind::Idle)
    }
}

fn map_labels(labels: Option<&[GhLabel]>) -> FieldValue {
    let mut names: Vec<String> = labels
        .unwrap_or_default()
        .iter()
        .map(|label| label.name.clone())
        .collect();

    names.sort();

    FieldValue::Text {
        value: names.join(", "),
    }
}

fn map_checks_status(status_check_rollup: Option<&[GhCheckEntry]>) -> FieldValue {
    let Some(entries) = status_check_rollup else {
        return status_field("none", StatusKind::Idle);
    };

    if entries.is_empty() {
        return status_field("none", StatusKind::Idle);
    }

    let mut has_pending = false;
    let mut has_success = false;

    for entry in entries {
        match entry {
            GhCheckEntry::CheckRun {
                status, conclusion, ..
            } => {
                let status = status.to_ascii_uppercase();
                let conclusion = conclusion
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_uppercase();

                if status != "COMPLETED" {
                    if matches!(
                        status.as_str(),
                        "PENDING" | "IN_PROGRESS" | "QUEUED" | "WAITING" | "REQUESTED"
                    ) {
                        has_pending = true;
                        continue;
                    }

                    has_pending = true;
                    continue;
                }

                if matches!(
                    conclusion.as_str(),
                    "FAILURE" | "ERROR" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED"
                ) {
                    return status_field("failing", StatusKind::AttentionNegative);
                }

                if matches!(conclusion.as_str(), "SUCCESS" | "SKIPPED" | "NEUTRAL") {
                    has_success = true;
                    continue;
                }

                if matches!(conclusion.as_str(), "STALE" | "STARTUP_FAILURE") {
                    has_pending = true;
                    continue;
                }
            }
            GhCheckEntry::StatusContext { state, .. } => {
                let state = state.to_ascii_uppercase();

                if matches!(state.as_str(), "FAILURE" | "ERROR") {
                    return status_field("failing", StatusKind::AttentionNegative);
                }

                if matches!(state.as_str(), "PENDING" | "EXPECTED") {
                    has_pending = true;
                    continue;
                }

                if state == "SUCCESS" {
                    has_success = true;
                }
            }
            GhCheckEntry::Unknown => {
                has_pending = true;
            }
        }
    }

    if has_pending {
        return status_field("pending", StatusKind::Running);
    }

    if has_success {
        return status_field("passing", StatusKind::Idle);
    }

    status_field("none", StatusKind::Idle)
}

fn status_field(value: &str, kind: StatusKind) -> FieldValue {
    FieldValue::Status {
        value: value.to_string(),
        kind,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPullRequest {
    number: u64,
    title: String,
    url: Option<String>,
    #[serde(default)]
    is_draft: bool,
    #[serde(default)]
    labels: Option<Vec<GhLabel>>,
    #[serde(default)]
    mergeable: Option<String>,
    #[serde(default)]
    review_decision: Option<String>,
    #[serde(default)]
    status_check_rollup: Option<Vec<GhCheckEntry>>,
    #[serde(default)]
    head_repository_owner: Option<GhOwner>,
    #[serde(default)]
    head_repository: Option<GhRepo>,
}

impl GhPullRequest {
    fn repo_hint(&self) -> String {
        match (&self.head_repository_owner, &self.head_repository) {
            (Some(owner), Some(repo)) => format!("{}/{}", owner.login, repo.name),
            _ => "unknown/unknown".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GhOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GhRepo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
enum GhCheckEntry {
    CheckRun {
        status: String,
        #[serde(default)]
        conclusion: Option<String>,
    },
    StatusContext {
        state: String,
    },
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use async_trait::async_trait;
    use tokio::sync::Mutex;
    use toml::{Table, Value};

    use crate::feed::{
        config::{FeedConfig, FieldOverride},
        process::{CommandError, CommandInvocation, CommandOutput, ProcessRunner},
        Feed, FieldValue, StatusKind,
    };

    use super::{
        map_checks_status, map_labels, map_mergeable_state, map_review_decision, GithubPrFeed,
    };

    use crate::feed::github_common::{GH_MISSING_MESSAGE, GH_UNAUTHENTICATED_MESSAGE};

    #[derive(Clone)]
    struct StubRunner {
        responses: Arc<Mutex<Vec<std::result::Result<CommandOutput, CommandError>>>>,
        invocations: Arc<Mutex<Vec<CommandInvocation>>>,
    }

    impl StubRunner {
        fn new(responses: Vec<std::result::Result<CommandOutput, CommandError>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
                invocations: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn commands(&self) -> Vec<String> {
            self.invocations
                .lock()
                .await
                .iter()
                .map(CommandInvocation::display)
                .collect()
        }
    }

    #[async_trait]
    impl ProcessRunner for StubRunner {
        async fn run(
            &self,
            invocation: CommandInvocation,
        ) -> std::result::Result<CommandOutput, CommandError> {
            self.invocations.lock().await.push(invocation);
            self.responses.lock().await.remove(0)
        }
    }

    #[tokio::test]
    async fn poll_invokes_single_pr_list_request_after_preflight() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  ✓ Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: gh_list_json_fixture(),
                stderr: String::new(),
            }),
        ]));

        let feed = GithubPrFeed::from_config_with_runner(&base_config(), runner.clone())
            .expect("feed should build");

        let activities = feed.poll().await.expect("poll should succeed");
        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].title, "#42 Add feed scaffold");
        assert_eq!(
            activities[0].id,
            "https://github.com/personal/cortado/pull/42"
        );

        let commands = runner.commands().await;
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "gh --version");
        assert_eq!(commands[1], "gh auth status");
        assert_eq!(
            commands[2],
            "gh pr list --repo personal/cortado --author @me --state open --limit 20 --json \"number,title,url,isDraft,labels,mergeable,reviewDecision,statusCheckRollup\""
        );
    }

    #[tokio::test]
    async fn poll_without_user_omits_author_flag() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  ✓ Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
        ]));

        let feed = GithubPrFeed::from_config_with_runner(&base_config_no_user(), runner.clone())
            .expect("feed should build");

        feed.poll().await.expect("poll should succeed");

        let commands = runner.commands().await;
        assert_eq!(
            commands[2],
            "gh pr list --repo personal/cortado --state open --limit 20 --json \"number,title,url,isDraft,labels,mergeable,reviewDecision,statusCheckRollup\""
        );
    }

    #[tokio::test]
    async fn poll_with_specific_user_passes_author_flag() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  ✓ Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
        ]));

        let mut config = base_config();
        config
            .type_specific
            .insert("user".to_string(), Value::String("octocat".to_string()));

        let feed = GithubPrFeed::from_config_with_runner(&config, runner.clone())
            .expect("feed should build");

        feed.poll().await.expect("poll should succeed");

        let commands = runner.commands().await;
        assert_eq!(
            commands[2],
            "gh pr list --repo personal/cortado --author octocat --state open --limit 20 --json \"number,title,url,isDraft,labels,mergeable,reviewDecision,statusCheckRollup\""
        );
    }

    #[tokio::test]
    async fn poll_missing_gh_binary_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![Err(CommandError::NotFound {
            program: "gh".to_string(),
        })]));

        let feed = GithubPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");
        assert_eq!(error.to_string(), GH_MISSING_MESSAGE);
    }

    #[tokio::test]
    async fn poll_unauthenticated_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "Run `gh auth login`".to_string(),
            }),
        ]));

        let feed = GithubPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");
        assert_eq!(error.to_string(), GH_UNAUTHENTICATED_MESSAGE);
    }

    #[tokio::test]
    async fn poll_non_auth_api_failure_includes_context() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  ✓ Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "GraphQL error: resource not accessible".to_string(),
            }),
        ]));

        let feed = GithubPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");

        assert!(error
            .to_string()
            .contains("GraphQL error: resource not accessible"));
    }

    #[tokio::test]
    async fn empty_pr_set_returns_empty_success() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  ✓ Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
        ]));

        let feed = GithubPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let activities = feed.poll().await.expect("should succeed");
        assert!(activities.is_empty());
    }

    #[tokio::test]
    async fn provided_fields_and_activity_fields_apply_overrides() {
        let mut config = base_config();
        config.field_overrides.insert(
            "labels".to_string(),
            FieldOverride {
                visible: Some(false),
                label: Some("Tag list".to_string()),
            },
        );

        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  ✓ Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: gh_list_json_fixture(),
                stderr: String::new(),
            }),
        ]));

        let feed = GithubPrFeed::from_config_with_runner(&config, runner).expect("builds");

        let fields = feed.provided_fields();
        let labels_meta = fields
            .iter()
            .find(|definition| definition.name == "labels")
            .expect("labels metadata exists");
        assert_eq!(labels_meta.label, "Tag list");

        let activities = feed.poll().await.expect("polls");
        for activity in activities {
            assert!(activity.fields.iter().all(|field| field.name != "labels"));
        }
    }

    #[test]
    fn review_mapping_is_deterministic() {
        assert_status(
            map_review_decision(Some("APPROVED")),
            "approved",
            StatusKind::AttentionPositive,
        );
        assert_status(
            map_review_decision(Some("CHANGES_REQUESTED")),
            "changes requested",
            StatusKind::AttentionNegative,
        );
        assert_status(
            map_review_decision(Some("REVIEW_REQUIRED")),
            "awaiting",
            StatusKind::Waiting,
        );
        assert_status(map_review_decision(None), "none", StatusKind::Idle);
    }

    #[test]
    fn mergeable_mapping_is_deterministic() {
        assert_status(
            map_mergeable_state(Some("MERGEABLE")),
            "yes",
            StatusKind::Idle,
        );
        assert_status(
            map_mergeable_state(Some("CONFLICTING")),
            "no",
            StatusKind::AttentionNegative,
        );
        assert_status(
            map_mergeable_state(Some("UNKNOWN")),
            "unknown",
            StatusKind::Running,
        );
        assert_status(map_mergeable_state(None), "unknown", StatusKind::Idle);
    }

    #[test]
    fn labels_are_sorted_lexicographically() {
        #[derive(serde::Deserialize)]
        struct Wrapper {
            labels: Vec<super::GhLabel>,
        }

        let parsed: Wrapper =
            serde_json::from_str(r#"{"labels":[{"name":"z"},{"name":"a"},{"name":"m"}]}"#)
                .expect("json");

        let FieldValue::Text { value } = map_labels(Some(&parsed.labels)) else {
            panic!("expected text");
        };

        assert_eq!(value, "a, m, z");
    }

    #[test]
    fn checks_mapping_failing_pending_passing_and_none() {
        let parsed: Vec<super::GhCheckEntry> = serde_json::from_str(
            r#"[
                {"__typename":"CheckRun","status":"COMPLETED","conclusion":"FAILURE"}
            ]"#,
        )
        .expect("json");

        assert_status(
            map_checks_status(Some(&parsed)),
            "failing",
            StatusKind::AttentionNegative,
        );

        let parsed: Vec<super::GhCheckEntry> = serde_json::from_str(
            r#"[
                {"__typename":"CheckRun","status":"IN_PROGRESS","conclusion":null}
            ]"#,
        )
        .expect("json");

        assert_status(
            map_checks_status(Some(&parsed)),
            "pending",
            StatusKind::Running,
        );

        let parsed: Vec<super::GhCheckEntry> = serde_json::from_str(
            r#"[
                {"__typename":"CheckRun","status":"COMPLETED","conclusion":"SUCCESS"},
                {"__typename":"StatusContext","state":"SUCCESS"}
            ]"#,
        )
        .expect("json");

        assert_status(
            map_checks_status(Some(&parsed)),
            "passing",
            StatusKind::Idle,
        );

        assert_status(map_checks_status(None), "none", StatusKind::Idle);
    }

    #[test]
    fn from_config_requires_repo() {
        let mut config = base_config();
        config.type_specific.remove("repo");

        let error = match GithubPrFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("repo should be required"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("missing required `repo`"));

        let mut config = base_config();
        config.type_specific.remove("user");
        let feed =
            GithubPrFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("missing user should mean no filter");
        assert_eq!(feed.user, None);

        let mut config = base_config();
        config
            .type_specific
            .insert("user".to_string(), Value::String("   ".to_string()));
        let feed =
            GithubPrFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("blank user should mean no filter");
        assert_eq!(feed.user, None);
    }

    fn assert_status(value: FieldValue, expected_value: &str, expected_kind: StatusKind) {
        let FieldValue::Status { value, kind } = value else {
            panic!("expected status field");
        };

        assert_eq!(value, expected_value);
        assert_eq!(kind, expected_kind);
    }

    fn base_config() -> FeedConfig {
        let mut type_specific = Table::new();
        type_specific.insert(
            "repo".to_string(),
            Value::String("personal/cortado".to_string()),
        );
        type_specific.insert("user".to_string(), Value::String("@me".to_string()));

        FeedConfig {
            name: "My PRs".to_string(),
            feed_type: "github-pr".to_string(),
            interval: Some(Duration::from_secs(60)),
            retain: None,
            notify: None,
            type_specific,
            field_overrides: HashMap::new(),
        }
    }

    /// Config with no user filter (show all authors).
    fn base_config_no_user() -> FeedConfig {
        let mut config = base_config();
        config.type_specific.remove("user");
        config
    }

    fn gh_list_json_fixture() -> String {
        r#"[
            {
                "number": 42,
                "title": "Add feed scaffold",
                "url": "https://github.com/personal/cortado/pull/42",
                "isDraft": false,
                "labels": [{"name": "phase-1"}, {"name": "feed"}],
                "mergeable": "MERGEABLE",
                "reviewDecision": "REVIEW_REQUIRED",
                "statusCheckRollup": [
                    {"__typename": "CheckRun", "status": "COMPLETED", "conclusion": "SUCCESS"}
                ]
            },
            {
                "number": 35,
                "title": "Improve shell feed docs",
                "url": "https://github.com/personal/cortado/pull/35",
                "isDraft": true,
                "labels": [{"name": "documentation"}, {"name": "needs-work"}],
                "mergeable": "CONFLICTING",
                "reviewDecision": "CHANGES_REQUESTED",
                "statusCheckRollup": [
                    {"__typename": "CheckRun", "status": "COMPLETED", "conclusion": "FAILURE"}
                ]
            }
        ]"#
        .to_string()
    }
}
