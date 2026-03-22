use anyhow::{anyhow, Result};
use toml::Value;

use crate::feed::{
    config::FeedConfig, Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 120;

/// Stub feed that returns hardcoded GitHub pull request activities.
pub struct GithubPrFeed {
    name: String,
    repo: String,
    interval: u64,
}

impl GithubPrFeed {
    /// Builds a GitHub PR feed from a parsed feed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        let repo = config
            .type_specific
            .get("repo")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type github-pr) is missing required `repo` string",
                    config.name
                )
            })?;

        Ok(Self {
            name: config.name.clone(),
            repo: repo.to_string(),
            interval: config.interval.unwrap_or(DEFAULT_INTERVAL_SECONDS),
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

    fn provided_fields(&self) -> Vec<FieldDefinition> {
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

    async fn poll(&self) -> Result<Vec<Activity>> {
        let interval_note = format!("stub interval {}s", self.interval);

        Ok(vec![
            Activity {
                id: format!("{}/pull/42", self.repo),
                title: format!("#42 Add feed scaffold ({interval_note})"),
                fields: vec![
                    Field {
                        name: "review".to_string(),
                        label: "Review".to_string(),
                        value: FieldValue::Status {
                            value: "awaiting".to_string(),
                            severity: StatusKind::Pending,
                        },
                    },
                    Field {
                        name: "checks".to_string(),
                        label: "Checks".to_string(),
                        value: FieldValue::Status {
                            value: "passing".to_string(),
                            severity: StatusKind::Success,
                        },
                    },
                    Field {
                        name: "mergeable".to_string(),
                        label: "Mergeable".to_string(),
                        value: FieldValue::Status {
                            value: "yes".to_string(),
                            severity: StatusKind::Success,
                        },
                    },
                    Field {
                        name: "draft".to_string(),
                        label: "Draft".to_string(),
                        value: FieldValue::Status {
                            value: "no".to_string(),
                            severity: StatusKind::Neutral,
                        },
                    },
                    Field {
                        name: "labels".to_string(),
                        label: "Labels".to_string(),
                        value: FieldValue::Text {
                            value: "feed, phase-1".to_string(),
                        },
                    },
                ],
            },
            Activity {
                id: format!("{}/pull/38", self.repo),
                title: "#38 Fix tray icon alignment".to_string(),
                fields: vec![
                    Field {
                        name: "review".to_string(),
                        label: "Review".to_string(),
                        value: FieldValue::Status {
                            value: "approved".to_string(),
                            severity: StatusKind::Success,
                        },
                    },
                    Field {
                        name: "checks".to_string(),
                        label: "Checks".to_string(),
                        value: FieldValue::Status {
                            value: "passing".to_string(),
                            severity: StatusKind::Success,
                        },
                    },
                    Field {
                        name: "mergeable".to_string(),
                        label: "Mergeable".to_string(),
                        value: FieldValue::Status {
                            value: "yes".to_string(),
                            severity: StatusKind::Success,
                        },
                    },
                    Field {
                        name: "draft".to_string(),
                        label: "Draft".to_string(),
                        value: FieldValue::Status {
                            value: "no".to_string(),
                            severity: StatusKind::Neutral,
                        },
                    },
                    Field {
                        name: "labels".to_string(),
                        label: "Labels".to_string(),
                        value: FieldValue::Text {
                            value: "ui".to_string(),
                        },
                    },
                ],
            },
            Activity {
                id: format!("{}/pull/35", self.repo),
                title: "#35 Improve shell feed docs".to_string(),
                fields: vec![
                    Field {
                        name: "review".to_string(),
                        label: "Review".to_string(),
                        value: FieldValue::Status {
                            value: "changes requested".to_string(),
                            severity: StatusKind::Warning,
                        },
                    },
                    Field {
                        name: "checks".to_string(),
                        label: "Checks".to_string(),
                        value: FieldValue::Status {
                            value: "failing".to_string(),
                            severity: StatusKind::Error,
                        },
                    },
                    Field {
                        name: "mergeable".to_string(),
                        label: "Mergeable".to_string(),
                        value: FieldValue::Status {
                            value: "no".to_string(),
                            severity: StatusKind::Error,
                        },
                    },
                    Field {
                        name: "draft".to_string(),
                        label: "Draft".to_string(),
                        value: FieldValue::Status {
                            value: "yes".to_string(),
                            severity: StatusKind::Pending,
                        },
                    },
                    Field {
                        name: "labels".to_string(),
                        label: "Labels".to_string(),
                        value: FieldValue::Text {
                            value: "documentation, needs-work".to_string(),
                        },
                    },
                ],
            },
        ])
    }
}
