use std::{collections::HashSet, str::FromStr};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use toml::Value;

use crate::feed::{
    ado_common::validate_hosted_ado_organization, config::FeedConfig, Field, FieldDefinition,
    FieldType, FieldValue, StatusKind,
};

pub(super) fn required_string<'a>(config: &'a FeedConfig, key: &str) -> Result<&'a str> {
    config
        .type_specific
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow!(
                "feed `{}` (type ado-pipelines) is missing required `{key}` string",
                config.name
            )
        })
}

pub(super) fn validate_organization(raw: &str) -> Result<String> {
    validate_hosted_ado_organization(raw)
}

pub(super) fn validate_project(raw: &str) -> Result<String> {
    validate_plain_value(raw, "project", 256)
}

pub(super) fn validate_folder(raw: &str) -> Result<String> {
    let folder = validate_plain_value(raw, "folder", 1024)?;
    if !folder.starts_with('\\') || folder.contains('/') {
        bail!("`folder` must be an exact Azure DevOps folder path beginning with `\\`");
    }
    Ok(folder)
}

fn validate_plain_value(raw: &str, name: &str, max_len: usize) -> Result<String> {
    let value = raw.trim();
    if value.is_empty() {
        bail!("`{name}` must not be empty");
    }
    if value.len() > max_len || value.chars().any(char::is_control) {
        bail!("`{name}` contains unsupported characters or is too long");
    }
    Ok(value.to_string())
}

pub(super) fn parse_pipeline_ids(
    value: &Value,
    prefix: &str,
    max_pipelines: usize,
) -> Result<Vec<u32>> {
    let values = value
        .as_array()
        .ok_or_else(|| anyhow!("{prefix}: `pipeline_ids` must be an array of integers"))?;
    if values.is_empty() {
        bail!("{prefix}: `pipeline_ids` must contain at least one ID");
    }
    if values.len() > max_pipelines {
        bail!("{prefix}: `pipeline_ids` must contain at most {max_pipelines} IDs");
    }

    let mut seen = HashSet::new();
    values
        .iter()
        .map(|value| {
            let id = value.as_integer().ok_or_else(|| {
                anyhow!("{prefix}: every `pipeline_ids` entry must be an integer")
            })?;
            let id = u32::try_from(id)
                .ok()
                .filter(|id| *id <= i32::MAX as u32 && *id > 0)
                .ok_or_else(|| {
                    anyhow!("{prefix}: pipeline IDs must be positive 32-bit integers")
                })?;
            if !seen.insert(id) {
                bail!("{prefix}: `pipeline_ids` must not contain duplicate ID {id}");
            }
            Ok(id)
        })
        .collect()
}

pub(super) fn base_field_definitions() -> Vec<FieldDefinition> {
    vec![
        definition("status", "Status", FieldType::Status, "Latest run status"),
        definition(
            "branch",
            "Branch",
            FieldType::Text,
            "Latest run source branch",
        ),
        definition("run", "Run", FieldType::Text, "Latest run number or name"),
        definition(
            "event",
            "Event",
            FieldType::Text,
            "Latest run trigger reason",
        ),
        definition("link", "Link", FieldType::Url, "Latest run or pipeline URL"),
    ]
}

fn definition(
    name: &str,
    label: &str,
    field_type: FieldType,
    description: &str,
) -> FieldDefinition {
    FieldDefinition {
        name: name.to_string(),
        label: label.to_string(),
        field_type,
        description: description.to_string(),
    }
}

pub(super) fn field(name: &str, label: &str, value: FieldValue) -> Field {
    Field {
        name: name.to_string(),
        label: label.to_string(),
        value,
    }
}

pub(super) fn map_run_status(status: Option<&str>, result: Option<&str>) -> FieldValue {
    match status.unwrap_or_default().to_ascii_lowercase().as_str() {
        "notstarted" | "postponed" => status_field("queued", StatusKind::Waiting),
        "inprogress" => status_field("running", StatusKind::Running),
        "cancelling" => status_field("cancelling", StatusKind::Running),
        "completed" => match result.unwrap_or_default().to_ascii_lowercase().as_str() {
            "succeeded" => status_field("passing", StatusKind::Idle),
            "failed" => status_field("failing", StatusKind::AttentionNegative),
            "partiallysucceeded" => {
                status_field("partially succeeded", StatusKind::AttentionNegative)
            }
            "canceled" => status_field("cancelled", StatusKind::AttentionNegative),
            _ => status_field("unknown", StatusKind::Idle),
        },
        _ => status_field("unknown", StatusKind::Idle),
    }
}

pub(super) fn status_field(value: &str, kind: StatusKind) -> FieldValue {
    FieldValue::Status {
        value: value.to_string(),
        kind,
    }
}

pub(super) fn build_browser_url(
    organization: &str,
    project: &str,
    definition_id: u32,
    build_id: Option<u32>,
) -> Result<String> {
    let mut url = reqwest::Url::parse(&format!("{organization}/"))?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| anyhow!("organization URL cannot be used as a base URL"))?;
        segments.pop_if_empty().push(project).push("_build");
        if build_id.is_some() {
            segments.push("results");
        }
    }
    let mut query = url.query_pairs_mut();
    match build_id {
        Some(id) => {
            query
                .append_pair("buildId", &id.to_string())
                .append_pair("view", "results");
        }
        None => {
            query.append_pair("definitionId", &definition_id.to_string());
        }
    }
    drop(query);
    Ok(url.to_string())
}

pub(super) fn parse_iso_to_unix_ms(value: Option<&str>) -> Option<u64> {
    let timestamp = jiff::Timestamp::from_str(value?).ok()?;
    u64::try_from(timestamp.as_millisecond()).ok()
}

#[derive(Debug, Deserialize)]
pub(super) struct DefinitionsResponse {
    pub(super) value: Vec<AdoPipelineDefinition>,
    #[serde(default, alias = "continuationToken")]
    pub(super) continuation_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AdoPipelineDefinition {
    pub(super) id: u32,
    pub(super) name: String,
    #[serde(default)]
    pub(super) path: Option<String>,
    #[serde(default, rename = "latestBuild")]
    pub(super) latest_build: Option<AdoBuild>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AdoBuild {
    pub(super) id: Option<u32>,
    pub(super) status: Option<String>,
    pub(super) result: Option<String>,
    pub(super) build_number: Option<String>,
    pub(super) source_branch: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) queue_time: Option<String>,
}
