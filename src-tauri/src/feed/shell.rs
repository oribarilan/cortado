use anyhow::{anyhow, Result};
use toml::Value;

use crate::feed::{
    config::FeedConfig, Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 30;

/// Stub feed that returns hardcoded command output for a single activity.
pub struct ShellFeed {
    name: String,
    command: String,
    field_name: String,
    field_label: String,
    field_type: FieldType,
    interval: u64,
}

impl ShellFeed {
    /// Builds a shell feed from parsed feed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        let command = config
            .type_specific
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type shell) is missing required `command` string",
                    config.name
                )
            })?;

        let field_name = config
            .type_specific
            .get("field_name")
            .and_then(Value::as_str)
            .unwrap_or("output")
            .to_string();

        let field_type = match config
            .type_specific
            .get("field_type")
            .and_then(Value::as_str)
            .unwrap_or("text")
        {
            "text" => FieldType::Text,
            "status" => FieldType::Status,
            "number" => FieldType::Number,
            "url" => FieldType::Url,
            other => {
                return Err(anyhow!(
                    "feed `{}` (type shell) has invalid `field_type` `{other}`; expected one of text|status|number|url",
                    config.name
                ));
            }
        };

        let default_label = if field_name == "output" {
            "Output".to_string()
        } else {
            field_name.clone()
        };

        let field_label = config
            .type_specific
            .get("label")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| {
                config
                    .field_overrides
                    .get(&field_name)
                    .and_then(|override_cfg| override_cfg.label.clone())
            })
            .unwrap_or(default_label);

        let _field_visible = config
            .field_overrides
            .get(&field_name)
            .and_then(|override_cfg| override_cfg.visible)
            .unwrap_or(true);

        Ok(Self {
            name: config.name.clone(),
            command: command.to_string(),
            field_name,
            field_label,
            field_type,
            interval: config.interval.unwrap_or(DEFAULT_INTERVAL_SECONDS),
        })
    }
}

#[async_trait::async_trait]
impl Feed for ShellFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "shell"
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        vec![FieldDefinition {
            name: self.field_name.clone(),
            label: self.field_label.clone(),
            field_type: self.field_type.clone(),
            description: format!("Output of command: {}", self.command),
        }]
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        let fake_output = format!(
            "stub output: command `{}` refreshed every {}s",
            self.command, self.interval
        );

        let value = match &self.field_type {
            FieldType::Text => FieldValue::Text { value: fake_output },
            FieldType::Status => FieldValue::Status {
                value: "ok".to_string(),
                severity: StatusKind::Neutral,
            },
            FieldType::Number => FieldValue::Number {
                value: self.interval as f64,
            },
            FieldType::Url => FieldValue::Url {
                value: "https://example.com".to_string(),
            },
        };

        Ok(vec![Activity {
            id: format!("shell:{}", self.name),
            title: self.command.clone(),
            fields: vec![Field {
                name: self.field_name.clone(),
                label: self.field_label.clone(),
                value,
            }],
        }])
    }
}
