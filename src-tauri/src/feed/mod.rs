use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use self::{ado_pr::AdoPrFeed, config::FeedConfig, github_pr::GithubPrFeed, shell::ShellFeed};

pub mod ado_pr;
pub mod concurrent;
pub mod config;
pub mod dependency;
pub mod field_overrides;
pub mod github_pr;
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

#[allow(dead_code)] // Used by notification subsystem (tasks 05-07).
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
}

impl FeedRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            feeds: Vec::new(),
            errored: Vec::new(),
        }
    }

    /// Registers a feed implementation.
    pub fn register(&mut self, feed: Arc<dyn Feed>) {
        self.feeds.push(feed);
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

    Ok(registry)
}

pub(crate) fn instantiate_feed(config: &FeedConfig) -> Result<Arc<dyn Feed>> {
    match config.feed_type.as_str() {
        "github-pr" => {
            GithubPrFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>)
        }
        "ado-pr" => AdoPrFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>),
        "shell" => ShellFeed::from_config(config).map(|feed| Arc::new(feed) as Arc<dyn Feed>),
        unknown => Err(anyhow::anyhow!("unknown feed type `{unknown}`")),
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
}
