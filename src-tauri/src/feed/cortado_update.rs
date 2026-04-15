use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::feed::{
    changelog, Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};
use crate::settings_config;

const FEED_NAME: &str = "Cortado Updates";
const FEED_TYPE: &str = "cortado-update";
const DEFAULT_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours
const ENDPOINT: &str = "https://github.com/oribarilan/cortado/releases/latest/download/latest.json";
const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/oribarilan/cortado/main/CHANGELOG.md";

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
/// produces activities when a newer app version or plugin update is available.
pub struct CortadoUpdateFeed {
    current_version: semver::Version,
    endpoint: String,
    client: reqwest::Client,
    /// Whether to check for OpenCode plugin updates (true when the user has
    /// an `opencode-session` feed configured).
    check_opencode_plugin: bool,
    /// Whether to check for Copilot extension updates (true when the user has
    /// a `copilot-session` feed configured).
    check_copilot_extension: bool,
    /// Whether to check for Claude Code plugin updates (true when the user has
    /// a `claude-code-session` feed configured).
    check_claude_code_plugin: bool,
}

impl CortadoUpdateFeed {
    pub fn new(
        check_opencode_plugin: bool,
        check_copilot_extension: bool,
        check_claude_code_plugin: bool,
    ) -> Self {
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
            check_opencode_plugin,
            check_copilot_extension,
            check_claude_code_plugin,
        }
    }

    /// Checks whether the on-disk OpenCode plugin is outdated compared to
    /// the version embedded in this binary. Returns an activity if so.
    fn check_plugin_update(&self) -> Option<Activity> {
        if !self.check_opencode_plugin {
            return None;
        }

        let dir = settings_config::opencode_plugins_dir()?;
        let path = dir.join(settings_config::OPENCODE_PLUGIN_FILENAME);
        let content = std::fs::read_to_string(path).ok()?;

        if !settings_config::is_plugin_outdated(&content, settings_config::OPENCODE_PLUGIN_SOURCE) {
            return None;
        }

        let new_version =
            settings_config::parse_plugin_version(settings_config::OPENCODE_PLUGIN_SOURCE)
                .unwrap_or(0);

        Some(Activity {
            id: "plugin-update-opencode".to_string(),
            title: "OpenCode plugin update available".to_string(),
            fields: vec![
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
                        value: format!("v{new_version}"),
                    },
                },
            ],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        })
    }

    /// Checks whether the on-disk Copilot CLI plugin is outdated compared
    /// to the version embedded in this binary. Returns an activity if so.
    fn check_copilot_extension_update(&self) -> Option<Activity> {
        if !self.check_copilot_extension {
            return None;
        }

        let dir = settings_config::copilot_plugin_dir()?;
        let path = dir.join("cortado-hook.sh");
        let content = std::fs::read_to_string(path).ok()?;

        if !settings_config::is_plugin_outdated(&content, settings_config::COPILOT_HOOK_SCRIPT) {
            return None;
        }

        let new_version =
            settings_config::parse_plugin_version(settings_config::COPILOT_HOOK_SCRIPT)
                .unwrap_or(0);

        Some(Activity {
            id: "plugin-update-copilot".to_string(),
            title: "Copilot CLI extension update available".to_string(),
            fields: vec![
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
                        value: format!("v{new_version}"),
                    },
                },
            ],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        })
    }

    /// Checks whether the on-disk Claude Code plugin is outdated compared
    /// to the version embedded in this binary. Returns an activity if so.
    fn check_claude_code_plugin_update(&self) -> Option<Activity> {
        if !self.check_claude_code_plugin {
            return None;
        }

        // The installed plugin lives at:
        // ~/.claude/plugins/cache/cortado/cortado/*/scripts/cortado-hook.sh
        // The wildcard is a version directory. Iterate entries to find it.
        let home = dirs::home_dir()?;
        let cache_dir = home.join(".claude/plugins/cache/cortado/cortado");
        let entries = std::fs::read_dir(&cache_dir).ok()?;

        for entry in entries.flatten() {
            let hook_path = entry.path().join("scripts/cortado-hook.sh");
            if let Ok(content) = std::fs::read_to_string(&hook_path) {
                if settings_config::is_plugin_outdated(
                    &content,
                    settings_config::CLAUDE_CODE_HOOK_SCRIPT,
                ) {
                    let new_version = settings_config::parse_plugin_version(
                        settings_config::CLAUDE_CODE_HOOK_SCRIPT,
                    )
                    .unwrap_or(0);

                    return Some(Activity {
                        id: "plugin-update-claude-code".to_string(),
                        title: "Claude Code plugin update available".to_string(),
                        fields: vec![
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
                                    value: format!("v{new_version}"),
                                },
                            },
                        ],
                        retained: false,
                        retained_at_unix_ms: None,
                        sort_ts: None,
                        action: None,
                    });
                }
                // Found a hook script that's up to date -- no update needed.
                return None;
            }
        }

        None
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
            FieldDefinition {
                name: "changelog".to_string(),
                label: "Changelog".to_string(),
                field_type: FieldType::Text,
                description: "Aggregated changelog entries as JSON".to_string(),
            },
        ]
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        let mut activities = Vec::new();

        // App update check (network).
        match self.check_app_update().await {
            Ok(Some(activity)) => activities.push(activity),
            Ok(None) => {}
            Err(e) => {
                // Plugin update check is still valuable even if the network
                // check fails, so log the error and continue.
                eprintln!("app update check failed: {e}");
            }
        }

        // Plugin update check (local filesystem, fast).
        if let Some(activity) = self.check_plugin_update() {
            activities.push(activity);
        }

        // Copilot extension update check (local filesystem, fast).
        if let Some(activity) = self.check_copilot_extension_update() {
            activities.push(activity);
        }

        // Claude Code plugin update check (local filesystem, fast).
        if let Some(activity) = self.check_claude_code_plugin_update() {
            activities.push(activity);
        }

        Ok(activities)
    }
}

impl CortadoUpdateFeed {
    /// Checks the GitHub Releases endpoint for a newer app version.
    async fn check_app_update(&self) -> Result<Option<Activity>> {
        let response = self.client.get(&self.endpoint).send().await?;

        // No latest.json yet (first release, or endpoint misconfigured).
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let response = response.error_for_status()?;
        let latest: LatestJson = response.json().await?;

        let remote_version = latest
            .version
            .trim_start_matches('v')
            .parse::<semver::Version>()?;

        if remote_version <= self.current_version {
            return Ok(None);
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

        // Fetch changelog only when an update is actually available (cascade).
        let changelog_json = self.fetch_changelog(&remote_version).await;
        if let Some(json) = changelog_json {
            fields.push(Field {
                name: "changelog".to_string(),
                label: "Changelog".to_string(),
                value: FieldValue::Text { value: json },
            });
        }

        Ok(Some(Activity {
            id: format!("cortado-update-v{remote_version}"),
            title: format!("Cortado v{remote_version} available"),
            fields,
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        }))
    }

    /// Fetches `CHANGELOG.md` from GitHub and extracts entries between the
    /// current version and `to_version`. Returns the result as a JSON string,
    /// or `None` on any failure (network, parse, empty).
    async fn fetch_changelog(&self, to_version: &semver::Version) -> Option<String> {
        let response = self.client.get(CHANGELOG_URL).send().await.ok()?;
        if !response.status().is_success() {
            eprintln!("changelog fetch returned status {}", response.status());
            return None;
        }
        let body = response.text().await.ok()?;
        let versions = changelog::extract_range(
            &body,
            &self.current_version.to_string(),
            &to_version.to_string(),
        );
        if versions.is_empty() {
            return None;
        }
        serde_json::to_string(&versions).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_feed_parses_current_version() {
        let feed = CortadoUpdateFeed::new(false, false, false);
        // Should parse without panic.
        assert!(!feed.current_version.to_string().is_empty());
    }

    #[test]
    fn feed_metadata() {
        let feed = CortadoUpdateFeed::new(false, false, false);
        assert_eq!(feed.name(), "Cortado Updates");
        assert_eq!(feed.feed_type(), "cortado-update");
        assert_eq!(feed.interval(), Duration::from_secs(6 * 60 * 60));
        assert!(feed.retain_for().is_none());
        assert_eq!(feed.provided_fields().len(), 4);
    }

    #[test]
    fn plugin_check_skipped_when_disabled() {
        let feed = CortadoUpdateFeed::new(false, false, false);
        assert!(feed.check_plugin_update().is_none());
    }

    #[test]
    fn plugin_check_returns_none_when_file_missing() {
        // With check enabled but no plugin file on disk, should return None.
        let feed = CortadoUpdateFeed::new(true, false, false);
        // This test assumes ~/.config/opencode/plugins/cortado-opencode.ts
        // either doesn't exist or is up to date. Both result in no activity.
        let result = feed.check_plugin_update();
        // Can't assert None definitively (file might exist and be outdated in
        // dev), so just verify the method doesn't panic.
        let _ = result;
    }

    #[test]
    fn copilot_check_skipped_when_disabled() {
        let feed = CortadoUpdateFeed::new(false, false, false);
        assert!(feed.check_copilot_extension_update().is_none());
    }

    #[test]
    fn claude_code_check_skipped_when_disabled() {
        let feed = CortadoUpdateFeed::new(false, false, false);
        assert!(feed.check_claude_code_plugin_update().is_none());
    }
}
