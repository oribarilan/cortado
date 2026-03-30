use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use self::{
    ado_pr::AdoPrFeed,
    config::FeedConfig,
    github_actions::GithubActionsFeed,
    github_pr::GithubPrFeed,
    harness::{copilot::CopilotProvider, feed::HarnessFeed},
    http_health::HttpHealthFeed,
    shell::ShellFeed,
};

pub mod ado_pr;
pub mod concurrent;
pub mod config;
pub mod dependency;
pub mod field_overrides;
pub mod github_actions;
pub mod github_common;
pub mod github_pr;
pub mod harness;
pub mod http_health;
pub mod process;
pub mod runtime;
pub mod shell;

pub use runtime::{BackgroundPoller, FeedSnapshotCache};

/// Controls how feed registry construction handles invalid feed entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryBuildMode {
    /// Keep valid feeds and surface invalid ones as per-feed config-error snapshots.
    Tolerant,
    /// Fail the whole build on the first invalid feed entry.
    Strict,
}

/// Supported field data kinds.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Status,
    Number,
    Url,
}

/// Semantic status indicating who needs to act next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusKind {
    /// My turn — something's wrong (red).
    #[serde(rename = "attention-negative")]
    AttentionNegative,
    /// My turn — go do the thing (green).
    #[serde(rename = "attention-positive")]
    AttentionPositive,
    /// Someone else's turn (yellow).
    #[serde(rename = "waiting")]
    Waiting,
    /// Machine working (pulsing blue).
    #[serde(rename = "running")]
    Running,
    /// Nothing happening (gray).
    #[serde(rename = "idle")]
    Idle,
}

impl StatusKind {
    /// Priority rank for rollup: higher value wins.
    ///
    /// Matches the frontend `kindPriority()` in `App.tsx`.
    pub fn priority(self) -> u8 {
        match self {
            StatusKind::AttentionNegative => 5,
            StatusKind::Waiting => 4,
            StatusKind::Running => 3,
            StatusKind::AttentionPositive => 2,
            StatusKind::Idle => 1,
        }
    }

    /// Derives the rollup kind for an activity from its status fields.
    ///
    /// Returns the highest-priority `StatusKind` across all status fields,
    /// defaulting to `Idle` if no status fields exist.
    /// Retained activities always roll up as `Idle`.
    pub fn rollup_for_activity(activity: &Activity) -> StatusKind {
        if activity.retained {
            return StatusKind::Idle;
        }

        activity
            .fields
            .iter()
            .filter_map(|field| match &field.value {
                FieldValue::Status { kind, .. } => Some(*kind),
                _ => None,
            })
            .max_by_key(|kind| kind.priority())
            .unwrap_or(StatusKind::Idle)
    }

    /// Human-friendly display name for notifications.
    pub fn human_name(self) -> &'static str {
        match self {
            StatusKind::AttentionNegative => "needs attention",
            StatusKind::AttentionPositive => "ready to go",
            StatusKind::Waiting => "waiting",
            StatusKind::Running => "in progress",
            StatusKind::Idle => "idle",
        }
    }
}

/// Value payload for a field on an activity.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FieldValue {
    Text { value: String },
    Status { value: String, kind: StatusKind },
    Number { value: f64 },
    Url { value: String },
}

impl FieldValue {
    /// Returns the type discriminator as a string.
    pub fn field_type(&self) -> &str {
        match self {
            FieldValue::Text { .. } => "text",
            FieldValue::Status { .. } => "status",
            FieldValue::Number { .. } => "number",
            FieldValue::Url { .. } => "url",
        }
    }

    /// Returns the display-friendly value string.
    pub fn display_value(&self) -> String {
        match self {
            FieldValue::Text { value }
            | FieldValue::Status { value, .. }
            | FieldValue::Url { value } => value.clone(),
            FieldValue::Number { value } => {
                if value.fract() == 0.0 {
                    format!("{}", *value as i64)
                } else {
                    format!("{value:.2}")
                }
            }
        }
    }
}

/// Metadata describing a field that a feed can provide.
#[derive(Debug, Clone, Serialize)]
pub struct FieldDefinition {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub description: String,
}

/// A named field rendered on an activity.
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub label: String,
    pub value: FieldValue,
}

/// A single tracked item discovered by a feed.
#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    pub id: String,
    pub title: String,
    pub fields: Vec<Field>,
    #[serde(default)]
    pub retained: bool,
    #[serde(skip)]
    pub retained_at_unix_ms: Option<u64>,
    /// Optional sort hint: unix millis of last activity (most recent = highest).
    /// Used as tiebreaker within the same status kind. Not serialized to frontend.
    #[serde(skip)]
    pub sort_ts: Option<u64>,
}

/// Poll result for one feed, including optional feed-level error.
#[derive(Debug, Clone, Serialize)]
pub struct FeedSnapshot {
    pub name: String,
    pub feed_type: String,
    pub activities: Vec<Activity>,
    pub provided_fields: Vec<FieldDefinition>,
    pub error: Option<String>,
}

/// Feed contract implemented by each feed type.
#[async_trait]
pub trait Feed: Send + Sync {
    fn name(&self) -> &str;
    fn feed_type(&self) -> &str;
    fn interval(&self) -> Duration;
    fn retain_for(&self) -> Option<Duration>;
    fn provided_fields(&self) -> Vec<FieldDefinition>;
    async fn poll(&self) -> Result<Vec<Activity>>;
}

/// In-memory registry of active and config-errored feeds.
pub struct FeedRegistry {
    feeds: Vec<Arc<dyn Feed>>,
    errored: Vec<FeedSnapshot>,
    /// References to harness feeds for session lookup (focus_session).
    harness_feeds: Vec<Arc<harness::feed::HarnessFeed>>,
}

impl FeedRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            feeds: Vec::new(),
            errored: Vec::new(),
            harness_feeds: Vec::new(),
        }
    }

    /// Registers a feed implementation.
    pub fn register(&mut self, feed: Arc<dyn Feed>) {
        self.feeds.push(feed);
    }

    /// Registers a harness feed (also registers it as a normal feed).
    pub fn register_harness(&mut self, feed: Arc<harness::feed::HarnessFeed>) {
        self.feeds.push(feed.clone() as Arc<dyn Feed>);
        self.harness_feeds.push(feed);
    }

    /// Registers a feed-shaped config error so it can be rendered in the UI.
    pub fn register_error(&mut self, name: String, feed_type: String, error: String) {
        self.errored.push(FeedSnapshot {
            name,
            feed_type,
            activities: Vec::new(),
            provided_fields: Vec::new(),
            error: Some(error),
        });
    }

    /// Returns active feed implementations in registration order.
    pub fn active_feeds(&self) -> &[Arc<dyn Feed>] {
        &self.feeds
    }

    /// Finds a harness session by ID across all harness feeds.
    pub fn find_harness_session(&self, session_id: &str) -> Option<harness::SessionInfo> {
        for feed in &self.harness_feeds {
            if let Some(session) = feed.find_session(session_id) {
                return Some(session);
            }
        }
        None
    }

    /// Returns any cached harness session (for capabilities detection).
    pub fn any_harness_session(&self) -> Option<harness::SessionInfo> {
        for feed in &self.harness_feeds {
            if let Some(session) = feed.any_cached_session() {
                return Some(session);
            }
        }
        None
    }

    /// Returns cache seed snapshots in registration order.
    pub fn initial_snapshots(&self) -> Vec<FeedSnapshot> {
        let mut snapshots = Vec::with_capacity(self.feeds.len() + self.errored.len());

        for feed in &self.feeds {
            snapshots.push(FeedSnapshot {
                name: feed.name().to_string(),
                feed_type: feed.feed_type().to_string(),
                activities: Vec::new(),
                provided_fields: feed.provided_fields(),
                error: None,
            });
        }

        snapshots.extend(self.errored.iter().cloned());
        snapshots
    }
}

/// Loads and builds the feed registry from `feeds.toml`.
pub fn load_feed_registry(mode: RegistryBuildMode) -> Result<FeedRegistry> {
    let configs = config::load_feeds_config()?;
    build_feed_registry_from_configs(configs, mode)
}

/// Builds a feed registry from parsed configs.
pub fn build_feed_registry_from_configs(
    configs: Vec<FeedConfig>,
    mode: RegistryBuildMode,
) -> Result<FeedRegistry> {
    let mut registry = FeedRegistry::new();

    for config in configs {
        let feed_name = config.name.clone();
        let feed_type = config.feed_type.clone();

        if feed_type == "copilot-session" {
            match instantiate_harness_feed(&config) {
                Ok(feed) => registry.register_harness(feed),
                Err(error) => {
                    if mode == RegistryBuildMode::Strict {
                        return Err(anyhow::anyhow!(
                            "feed `{feed_name}` (`{feed_type}`) failed validation: {error}"
                        ));
                    }
                    registry.register_error(feed_name, feed_type, error.to_string());
                }
            }
        } else {
            match instantiate_feed(&config) {
                Ok(feed) => registry.register(feed),
                Err(error) => {
                    if mode == RegistryBuildMode::Strict {
                        return Err(anyhow::anyhow!(
                            "feed `{feed_name}` (`{feed_type}`) failed validation: {error}"
                        ));
                    }
                    registry.register_error(feed_name, feed_type, error.to_string());
                }
            }
        }
    }

    Ok(registry)
}

pub(crate) fn instantiate_feed(config: &FeedConfig) -> Result<Arc<dyn Feed>> {
    match config.feed_type.as_str() {
        "github-pr" => {
            GithubPrFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>)
        }
        "ado-pr" => AdoPrFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>),
        "shell" => ShellFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>),
        "http-health" => {
            HttpHealthFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>)
        }
        "github-actions" => {
            GithubActionsFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>)
        }
        unknown => Err(anyhow::anyhow!("unknown feed type `{unknown}`")),
    }
}

pub fn instantiate_harness_feed(config: &FeedConfig) -> Result<Arc<HarnessFeed>> {
    match config.feed_type.as_str() {
        "copilot-session" => {
            let provider = Box::new(CopilotProvider::new()?);
            HarnessFeed::from_config(config, provider).map(Arc::new)
        }
        other => Err(anyhow::anyhow!("unknown harness feed type `{other}`")),
    }
}

impl Default for FeedRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity_with_statuses(statuses: &[(&str, StatusKind)]) -> Activity {
        let fields = statuses
            .iter()
            .map(|(name, kind)| Field {
                name: name.to_string(),
                label: name.to_string(),
                value: FieldValue::Status {
                    value: "test".to_string(),
                    kind: *kind,
                },
            })
            .collect();

        Activity {
            id: "test".to_string(),
            title: "Test".to_string(),
            fields,
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
        }
    }

    #[test]
    fn priority_ordering_matches_spec() {
        assert!(StatusKind::AttentionNegative.priority() > StatusKind::Waiting.priority());
        assert!(StatusKind::Waiting.priority() > StatusKind::Running.priority());
        assert!(StatusKind::Running.priority() > StatusKind::AttentionPositive.priority());
        assert!(StatusKind::AttentionPositive.priority() > StatusKind::Idle.priority());
    }

    #[test]
    fn rollup_picks_highest_priority_kind() {
        let activity = activity_with_statuses(&[
            ("review", StatusKind::AttentionPositive),
            ("checks", StatusKind::Running),
            ("mergeable", StatusKind::Idle),
        ]);

        assert_eq!(
            StatusKind::rollup_for_activity(&activity),
            StatusKind::Running
        );
    }

    #[test]
    fn rollup_defaults_to_idle_with_no_status_fields() {
        let activity = Activity {
            id: "test".to_string(),
            title: "Test".to_string(),
            fields: vec![Field {
                name: "label".to_string(),
                label: "Labels".to_string(),
                value: FieldValue::Text {
                    value: "wip".to_string(),
                },
            }],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
        };

        assert_eq!(StatusKind::rollup_for_activity(&activity), StatusKind::Idle);
    }

    #[test]
    fn rollup_retained_is_always_idle() {
        let mut activity = activity_with_statuses(&[("checks", StatusKind::AttentionNegative)]);
        activity.retained = true;

        assert_eq!(StatusKind::rollup_for_activity(&activity), StatusKind::Idle);
    }

    #[test]
    fn human_names_are_lowercase() {
        assert_eq!(
            StatusKind::AttentionNegative.human_name(),
            "needs attention"
        );
        assert_eq!(StatusKind::AttentionPositive.human_name(), "ready to go");
        assert_eq!(StatusKind::Waiting.human_name(), "waiting");
        assert_eq!(StatusKind::Running.human_name(), "in progress");
        assert_eq!(StatusKind::Idle.human_name(), "idle");
    }

    // --- FieldValue tests ---

    #[test]
    fn field_type_returns_correct_discriminator() {
        assert_eq!(
            FieldValue::Text {
                value: "x".to_string()
            }
            .field_type(),
            "text"
        );
        assert_eq!(
            FieldValue::Status {
                value: "ok".to_string(),
                kind: StatusKind::Idle,
            }
            .field_type(),
            "status"
        );
        assert_eq!(FieldValue::Number { value: 1.0 }.field_type(), "number");
        assert_eq!(
            FieldValue::Url {
                value: "https://x.com".to_string()
            }
            .field_type(),
            "url"
        );
    }

    #[test]
    fn display_value_text_returns_clone() {
        let fv = FieldValue::Text {
            value: "hello world".to_string(),
        };
        assert_eq!(fv.display_value(), "hello world");
    }

    #[test]
    fn display_value_status_returns_display_text() {
        let fv = FieldValue::Status {
            value: "approved".to_string(),
            kind: StatusKind::AttentionPositive,
        };
        assert_eq!(fv.display_value(), "approved");
    }

    #[test]
    fn display_value_number_integer_omits_decimals() {
        let fv = FieldValue::Number { value: 42.0 };
        assert_eq!(fv.display_value(), "42");
    }

    #[test]
    fn display_value_number_fractional_shows_two_decimals() {
        let fv = FieldValue::Number { value: 1.23456 };
        assert_eq!(fv.display_value(), "1.23");
    }

    #[test]
    fn display_value_number_zero() {
        let fv = FieldValue::Number { value: 0.0 };
        assert_eq!(fv.display_value(), "0");
    }

    #[test]
    fn display_value_number_negative_fractional() {
        let fv = FieldValue::Number { value: -7.5 };
        assert_eq!(fv.display_value(), "-7.50");
    }

    #[test]
    fn display_value_url_returns_clone() {
        let fv = FieldValue::Url {
            value: "https://example.com".to_string(),
        };
        assert_eq!(fv.display_value(), "https://example.com");
    }

    // --- FeedRegistry tests ---

    #[test]
    fn registry_starts_empty() {
        let registry = FeedRegistry::new();
        assert!(registry.active_feeds().is_empty());
        assert!(registry.initial_snapshots().is_empty());
    }

    #[test]
    fn registry_default_is_empty() {
        let registry = FeedRegistry::default();
        assert!(registry.active_feeds().is_empty());
    }

    #[test]
    fn registry_register_error_produces_error_snapshot() {
        let mut registry = FeedRegistry::new();
        registry.register_error(
            "bad-feed".to_string(),
            "shell".to_string(),
            "missing command".to_string(),
        );

        assert!(registry.active_feeds().is_empty());

        let snapshots = registry.initial_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].name, "bad-feed");
        assert_eq!(snapshots[0].feed_type, "shell");
        assert_eq!(snapshots[0].error.as_deref(), Some("missing command"));
        assert!(snapshots[0].activities.is_empty());
    }

    #[test]
    fn initial_snapshots_includes_feeds_then_errors() {
        use std::sync::Arc;

        struct DummyFeed;

        #[async_trait::async_trait]
        impl Feed for DummyFeed {
            fn name(&self) -> &str {
                "dummy"
            }
            fn feed_type(&self) -> &str {
                "test"
            }
            fn interval(&self) -> std::time::Duration {
                std::time::Duration::from_secs(30)
            }
            fn retain_for(&self) -> Option<std::time::Duration> {
                None
            }
            fn provided_fields(&self) -> Vec<FieldDefinition> {
                vec![FieldDefinition {
                    name: "status".to_string(),
                    label: "Status".to_string(),
                    field_type: FieldType::Status,
                    description: "test".to_string(),
                }]
            }
            async fn poll(&self) -> anyhow::Result<Vec<Activity>> {
                Ok(vec![])
            }
        }

        let mut registry = FeedRegistry::new();
        registry.register(Arc::new(DummyFeed));
        registry.register_error(
            "broken".to_string(),
            "shell".to_string(),
            "oops".to_string(),
        );

        let snapshots = registry.initial_snapshots();
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].name, "dummy");
        assert!(snapshots[0].error.is_none());
        assert_eq!(snapshots[1].name, "broken");
        assert!(snapshots[1].error.is_some());
    }

    // --- build_feed_registry_from_configs tests ---

    #[test]
    fn build_registry_tolerant_mode_keeps_valid_and_records_errors() {
        use config::FeedConfig;
        use std::collections::HashMap;
        use toml::Table;

        let configs = vec![
            FeedConfig {
                name: "Good".to_string(),
                feed_type: "shell".to_string(),
                interval: None,
                retain: None,
                notify: None,
                type_specific: {
                    let mut t = Table::new();
                    t.insert(
                        "command".to_string(),
                        toml::Value::String("echo hi".to_string()),
                    );
                    t
                },
                field_overrides: HashMap::new(),
            },
            FeedConfig {
                name: "Bad".to_string(),
                feed_type: "nonexistent-type".to_string(),
                interval: None,
                retain: None,
                notify: None,
                type_specific: Table::new(),
                field_overrides: HashMap::new(),
            },
        ];

        let registry = build_feed_registry_from_configs(configs, RegistryBuildMode::Tolerant)
            .expect("tolerant mode should not fail");

        assert_eq!(registry.active_feeds().len(), 1);
        assert_eq!(registry.active_feeds()[0].name(), "Good");

        let snapshots = registry.initial_snapshots();
        let error_snapshot = snapshots.iter().find(|s| s.name == "Bad").unwrap();
        assert!(error_snapshot.error.is_some());
    }

    #[test]
    fn build_registry_strict_mode_fails_on_first_error() {
        use config::FeedConfig;
        use std::collections::HashMap;
        use toml::Table;

        let configs = vec![FeedConfig {
            name: "Bad".to_string(),
            feed_type: "nonexistent-type".to_string(),
            interval: None,
            retain: None,
            notify: None,
            type_specific: Table::new(),
            field_overrides: HashMap::new(),
        }];

        let err = match build_feed_registry_from_configs(configs, RegistryBuildMode::Strict) {
            Ok(_) => panic!("strict mode should fail"),
            Err(e) => e,
        };

        assert!(err.to_string().contains("Bad"));
        assert!(err.to_string().contains("nonexistent-type"));
    }

    #[test]
    fn instantiate_feed_unknown_type_returns_error() {
        use config::FeedConfig;
        use std::collections::HashMap;
        use toml::Table;

        let config = FeedConfig {
            name: "X".to_string(),
            feed_type: "foobar".to_string(),
            interval: None,
            retain: None,
            notify: None,
            type_specific: Table::new(),
            field_overrides: HashMap::new(),
        };

        let err = match instantiate_feed(&config) {
            Ok(_) => panic!("unknown type should fail"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("unknown feed type `foobar`"));
    }

    // --- rollup edge case: single highest-priority field wins ---

    #[test]
    fn rollup_single_attention_negative_beats_all() {
        let activity = activity_with_statuses(&[
            ("review", StatusKind::AttentionPositive),
            ("checks", StatusKind::AttentionNegative),
            ("merge", StatusKind::Waiting),
        ]);

        assert_eq!(
            StatusKind::rollup_for_activity(&activity),
            StatusKind::AttentionNegative
        );
    }

    #[test]
    fn rollup_empty_fields_returns_idle() {
        let activity = Activity {
            id: "test".to_string(),
            title: "Test".to_string(),
            fields: Vec::new(),
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
        };
        assert_eq!(StatusKind::rollup_for_activity(&activity), StatusKind::Idle);
    }
}
