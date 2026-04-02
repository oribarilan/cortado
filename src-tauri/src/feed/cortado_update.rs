use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::feed::{Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind};

const FEED_NAME: &str = "Cortado Updates";
const FEED_TYPE: &str = "cortado-update";
const DEFAULT_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours
const ENDPOINT: &str = "https://github.com/oribarilan/cortado/releases/latest/download/latest.json";

/// Response shape from the Tauri updater `latest.json` endpoint.
#[derive(Deserialize)]
struct LatestJson {
    version: String,
    #[serde(default)]
    notes: Option<String>,
    // Deserialized but not currently used; kept for future display.
    #[serde(default)]
    #[allow(dead_code)]
    pub_date: Option<String>,
}

/// Built-in feed that checks for Cortado updates via `latest.json`.
///
/// Unlike user-configured feeds, this is always registered and not parsed
/// from `feeds.toml`. It polls the GitHub Releases updater endpoint and
/// produces a single activity when a newer version is available.
pub struct CortadoUpdateFeed {
    current_version: semver::Version,
    endpoint: String,
    client: reqwest::Client,
}

impl CortadoUpdateFeed {
    pub fn new() -> Self {
        let current_version = env!("CARGO_PKG_VERSION")
            .parse::<semver::Version>()
            .expect("CARGO_PKG_VERSION must be valid semver");

        Self {
            current_version,
            endpoint: ENDPOINT.to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("failed to build HTTP client"),
        }
    }
}

#[async_trait]
impl Feed for CortadoUpdateFeed {
    fn name(&self) -> &str {
        FEED_NAME
    }

    fn feed_type(&self) -> &str {
        FEED_TYPE
    }

    fn interval(&self) -> Duration {
        DEFAULT_INTERVAL
    }

    fn retain_for(&self) -> Option<Duration> {
        None
    }

    fn hide_when_empty(&self) -> bool {
        true
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        vec![
            FieldDefinition {
                name: "status".to_string(),
                label: "Status".to_string(),
                field_type: FieldType::Status,
                description: "Update availability status".to_string(),
            },
            FieldDefinition {
                name: "version".to_string(),
                label: "Version".to_string(),
                field_type: FieldType::Text,
                description: "Available version".to_string(),
            },
            FieldDefinition {
                name: "notes".to_string(),
                label: "Release notes".to_string(),
                field_type: FieldType::Text,
                description: "Release notes for the available version".to_string(),
            },
        ]
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        let response = self.client.get(&self.endpoint).send().await?;

        // No latest.json yet (first release, or endpoint misconfigured).
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        let response = response.error_for_status()?;
        let latest: LatestJson = response.json().await?;

        let remote_version = latest
            .version
            .trim_start_matches('v')
            .parse::<semver::Version>()?;

        if remote_version <= self.current_version {
            return Ok(Vec::new());
        }

        let mut fields = vec![
            Field {
                name: "status".to_string(),
                label: "Status".to_string(),
                value: FieldValue::Status {
                    value: "update available".to_string(),
                    kind: StatusKind::AttentionPositive,
                },
            },
            Field {
                name: "version".to_string(),
                label: "Version".to_string(),
                value: FieldValue::Text {
                    value: format!("v{remote_version}"),
                },
            },
        ];

        if let Some(notes) = &latest.notes {
            let trimmed = notes.trim();
            if !trimmed.is_empty() {
                fields.push(Field {
                    name: "notes".to_string(),
                    label: "Release notes".to_string(),
                    value: FieldValue::Text {
                        value: trimmed.to_string(),
                    },
                });
            }
        }

        Ok(vec![Activity {
            id: format!("cortado-update-v{remote_version}"),
            title: format!("Cortado v{remote_version} available"),
            fields,
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_feed_parses_current_version() {
        let feed = CortadoUpdateFeed::new();
        // Should parse without panic.
        assert!(!feed.current_version.to_string().is_empty());
    }

    #[test]
    fn feed_metadata() {
        let feed = CortadoUpdateFeed::new();
        assert_eq!(feed.name(), "Cortado Updates");
        assert_eq!(feed.feed_type(), "cortado-update");
        assert_eq!(feed.interval(), Duration::from_secs(6 * 60 * 60));
        assert!(feed.retain_for().is_none());
        assert_eq!(feed.provided_fields().len(), 3);
    }
}
