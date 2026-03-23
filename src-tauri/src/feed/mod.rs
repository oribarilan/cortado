use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub mod config;
pub mod dependency;
pub mod field_overrides;
pub mod github_pr;
pub mod process;
pub mod runtime;
pub mod shell;

pub use runtime::{BackgroundPoller, FeedSnapshotCache};

/// Supported field data kinds.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Status,
    Number,
    Url,
}

/// Visual severity for status fields.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusKind {
    Success,
    Warning,
    Error,
    Pending,
    Neutral,
}

/// Value payload for a field on an activity.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FieldValue {
    Text { value: String },
    Status { value: String, severity: StatusKind },
    Number { value: f64 },
    Url { value: String },
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
    fn interval_seconds(&self) -> u64;
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

impl Default for FeedRegistry {
    fn default() -> Self {
        Self::new()
    }
}
