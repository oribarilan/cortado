use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::feed::{self, config, Activity};

/// Frontend-friendly feed config entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfigDto {
    pub name: String,
    #[serde(rename = "type")]
    pub feed_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<bool>,
    #[serde(default)]
    pub type_specific: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub fields: HashMap<String, FieldOverrideDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOverrideDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn duration_to_string(d: Duration) -> String {
    let secs = d.as_secs();
    if secs == 0 {
        return "0s".to_string();
    }
    if secs.is_multiple_of(3600) {
        format!("{}h", secs / 3600)
    } else if secs.is_multiple_of(60) {
        format!("{}m", secs / 60)
    } else {
        format!("{secs}s")
    }
}

fn feed_config_to_dto(config: &config::FeedConfig) -> FeedConfigDto {
    let type_specific = config
        .type_specific
        .iter()
        .map(|(k, v)| {
            let json_val = toml_value_to_json(v);
            (k.clone(), json_val)
        })
        .collect();

    let fields = config
        .field_overrides
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                FieldOverrideDto {
                    visible: v.visible,
                    label: v.label.clone(),
                },
            )
        })
        .collect();

    FeedConfigDto {
        name: config.name.clone(),
        feed_type: config.feed_type.clone(),
        interval: config.interval.map(duration_to_string),
        retain: config.retain.map(duration_to_string),
        notify: config.notify,
        type_specific,
        fields,
    }
}

fn toml_value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Integer(i) => serde_json::json!(*i),
        Value::Float(f) => serde_json::json!(*f),
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(toml_value_to_json).collect()),
        Value::Table(table) => {
            let map: serde_json::Map<String, serde_json::Value> = table
                .iter()
                .map(|(k, v)| (k.clone(), toml_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

fn dto_to_toml_document(feeds: &[FeedConfigDto]) -> String {
    let mut output = String::new();

    for (i, feed) in feeds.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str("[[feed]]\n");
        output.push_str(&format!("name = {}\n", toml_quote(&feed.name)));
        output.push_str(&format!("type = {}\n", toml_quote(&feed.feed_type)));

        // Type-specific keys
        for (key, value) in &feed.type_specific {
            output.push_str(&format!("{key} = {}\n", json_value_to_toml_inline(value)));
        }

        if let Some(ref interval) = feed.interval {
            output.push_str(&format!("interval = {}\n", toml_quote(interval)));
        }
        if let Some(ref retain) = feed.retain {
            output.push_str(&format!("retain = {}\n", toml_quote(retain)));
        }

        if let Some(notify) = feed.notify {
            output.push_str(&format!("notify = {notify}\n"));
        }

        // Field overrides
        for (field_name, override_dto) in &feed.fields {
            if override_dto.visible.is_none() && override_dto.label.is_none() {
                continue;
            }
            output.push_str(&format!("\n[feed.fields.{field_name}]\n"));
            if let Some(visible) = override_dto.visible {
                output.push_str(&format!("visible = {visible}\n"));
            }
            if let Some(ref label) = override_dto.label {
                output.push_str(&format!("label = {}\n", toml_quote(label)));
            }
        }
    }

    output
}

fn toml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn json_value_to_toml_inline(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => toml_quote(s),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "\"\"".to_string(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_value_to_toml_inline).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(_) => "{}".to_string(),
    }
}

#[tauri::command]
pub fn get_feeds_config() -> Result<Vec<FeedConfigDto>, String> {
    let configs = config::load_feeds_config().map_err(|e| e.to_string())?;
    Ok(configs.iter().map(feed_config_to_dto).collect())
}

#[tauri::command]
pub fn save_feeds_config(feeds: Vec<FeedConfigDto>) -> Result<(), String> {
    let toml_str = dto_to_toml_document(&feeds);

    // Validate by parsing the generated TOML through the existing parser
    let _parsed = config::parse_feeds_config_str(&toml_str).map_err(|e| e.to_string())?;

    let config_path = config::feeds_config_path().map_err(|e| e.to_string())?;

    // Create parent directory if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed creating config directory: {e}"))?;
    }

    // Back up existing file
    if config_path.exists() {
        let backup_path = config_path.with_extension("toml.bak");
        fs::copy(&config_path, &backup_path)
            .map_err(|e| format!("failed backing up config: {e}"))?;
    }

    // Write new config
    fs::write(&config_path, &toml_str).map_err(|e| format!("failed writing config: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_config_path() -> Result<String, String> {
    let path = config::feeds_config_path().map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn open_config_file() -> Result<(), String> {
    let config_path = config::feeds_config_path().map_err(|e| e.to_string())?;

    // Create parent dir and empty file if it doesn't exist
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed creating config directory: {e}"))?;
        }
        fs::write(&config_path, "").map_err(|e| format!("failed creating config file: {e}"))?;
    }

    Command::new("open")
        .arg(&config_path)
        .spawn()
        .map_err(|e| format!("failed to open config file: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn reveal_config_file() -> Result<(), String> {
    let config_path = config::feeds_config_path().map_err(|e| e.to_string())?;

    // Create parent dir and empty file if it doesn't exist
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed creating config directory: {e}"))?;
        }
        fs::write(&config_path, "").map_err(|e| format!("failed creating config file: {e}"))?;
    }

    Command::new("open")
        .arg("-R")
        .arg(&config_path)
        .spawn()
        .map_err(|e| format!("failed to reveal config file: {e}"))?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DepCheckResult {
    pub installed: bool,
}

#[tauri::command]
pub fn check_feed_dependency(feed_type: String) -> DepCheckResult {
    let binary = match feed_type.as_str() {
        "github-pr" => "gh",
        "ado-pr" => "az",
        _ => return DepCheckResult { installed: true },
    };

    let installed = Command::new("which")
        .arg(binary)
        .output()
        .is_ok_and(|out| out.status.success());

    DepCheckResult { installed }
}

/// Ad-hoc poll result for the "Test" button in settings.
#[derive(Debug, Serialize)]
pub struct TestFeedResult {
    pub success: bool,
    pub error: Option<String>,
    pub activities: Vec<TestActivity>,
}

#[derive(Debug, Serialize)]
pub struct TestActivity {
    pub title: String,
    pub status: Option<String>,
}

fn dto_to_feed_config(dto: &FeedConfigDto) -> Result<config::FeedConfig, String> {
    let toml_str = dto_to_toml_document(std::slice::from_ref(dto));
    let mut configs = config::parse_feeds_config_str(&toml_str).map_err(|e| e.to_string())?;
    configs
        .pop()
        .ok_or_else(|| "failed to parse feed config".to_string())
}

fn activity_to_test_activity(a: &Activity) -> TestActivity {
    let status = a.fields.iter().find_map(|f| {
        if f.value.field_type() == "status" {
            Some(f.value.display_value())
        } else {
            None
        }
    });

    TestActivity {
        title: a.title.clone(),
        status,
    }
}

#[tauri::command]
pub async fn test_feed(feed_dto: FeedConfigDto) -> TestFeedResult {
    let feed_config = match dto_to_feed_config(&feed_dto) {
        Ok(c) => c,
        Err(e) => {
            return TestFeedResult {
                success: false,
                error: Some(format!("Invalid config: {e}")),
                activities: Vec::new(),
            }
        }
    };

    let feed = match feed::instantiate_feed(&feed_config) {
        Ok(f) => f,
        Err(e) => {
            return TestFeedResult {
                success: false,
                error: Some(e.to_string()),
                activities: Vec::new(),
            }
        }
    };

    match feed.poll().await {
        Ok(activities) => TestFeedResult {
            success: true,
            error: None,
            activities: activities.iter().map(activity_to_test_activity).collect(),
        },
        Err(e) => TestFeedResult {
            success: false,
            error: Some(e.to_string()),
            activities: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_to_string_formats_correctly() {
        assert_eq!(duration_to_string(Duration::from_secs(30)), "30s");
        assert_eq!(duration_to_string(Duration::from_secs(300)), "5m");
        assert_eq!(duration_to_string(Duration::from_secs(3600)), "1h");
        assert_eq!(duration_to_string(Duration::from_secs(90)), "90s");
    }

    #[test]
    fn dto_to_toml_round_trips_through_parser() {
        let feeds = vec![
            FeedConfigDto {
                name: "My PRs".into(),
                feed_type: "github-pr".into(),
                interval: Some("5m".into()),
                retain: None,
                notify: None,
                type_specific: [("repo".into(), serde_json::json!("org/frontend"))]
                    .into_iter()
                    .collect(),
                fields: HashMap::new(),
            },
            FeedConfigDto {
                name: "Deploy".into(),
                feed_type: "shell".into(),
                interval: Some("30s".into()),
                retain: Some("1h".into()),
                notify: Some(false),
                type_specific: [("command".into(), serde_json::json!("./check.sh"))]
                    .into_iter()
                    .collect(),
                fields: [(
                    "status".into(),
                    FieldOverrideDto {
                        visible: Some(true),
                        label: Some("Deploy Status".into()),
                    },
                )]
                .into_iter()
                .collect(),
            },
        ];

        let toml_str = dto_to_toml_document(&feeds);
        let parsed =
            config::parse_feeds_config_str(&toml_str).expect("generated TOML should parse cleanly");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "My PRs");
        assert_eq!(parsed[1].name, "Deploy");
        assert_eq!(
            parsed[1]
                .field_overrides
                .get("status")
                .unwrap()
                .label
                .as_deref(),
            Some("Deploy Status")
        );
    }

    #[test]
    fn toml_quote_escapes_special_chars() {
        assert_eq!(toml_quote("hello"), "\"hello\"");
        assert_eq!(toml_quote("he\"llo"), "\"he\\\"llo\"");
        assert_eq!(toml_quote("back\\slash"), "\"back\\\\slash\"");
    }
}
