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
pub fn check_feed_dependency(binary: String) -> DepCheckResult {
    let installed = Command::new("which")
        .arg(&binary)
        .output()
        .is_ok_and(|out| out.status.success());

    DepCheckResult { installed }
}

#[derive(Debug, Serialize)]
pub struct SetupCheckResult {
    pub ready: bool,
    pub outdated: bool,
}

#[derive(Debug, Serialize)]
pub struct SetupInstallResult {
    pub success: bool,
    pub error: Option<String>,
}

/// The plugin source, embedded at compile time from the single-file bundle.
pub(crate) const OPENCODE_PLUGIN_SOURCE: &str =
    include_str!("../../plugins/opencode/src/plugin-bundle.ts");

/// The filename written to OpenCode's global plugins directory.
pub(crate) const OPENCODE_PLUGIN_FILENAME: &str = "cortado-opencode.ts";

/// The Copilot CLI plugin hook script, embedded at compile time.
pub(crate) const COPILOT_HOOK_SCRIPT: &str = include_str!("../../plugins/copilot/cortado-hook.sh");

/// The Copilot CLI plugin manifest, embedded at compile time.
pub(crate) const COPILOT_PLUGIN_JSON: &str = include_str!("../../plugins/copilot/plugin.json");

/// The Copilot CLI plugin hooks configuration, embedded at compile time.
pub(crate) const COPILOT_HOOKS_JSON: &str = include_str!("../../plugins/copilot/hooks.json");

/// Returns the path to the OpenCode global plugins directory.
///
/// `~/.config/opencode/plugins/` (follows XDG conventions, same as OpenCode).
pub(crate) fn opencode_plugins_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".config/opencode/plugins"))
}

/// Returns the path to the Copilot CLI installed-plugins directory for the
/// Cortado plugin.
///
/// `~/.copilot/installed-plugins/_direct/copilot/` — where `copilot plugin install`
/// places plugins installed from a local directory named `copilot`.
pub(crate) fn copilot_plugin_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".copilot/installed-plugins/_direct/copilot"))
}

/// Parses a plugin version from a source string.
///
/// Looks for `// cortado-plugin-version: N` in the first 5 lines.
pub(crate) fn parse_plugin_version(source: &str) -> Option<u32> {
    for line in source.lines().take(5) {
        if let Some(rest) = line
            .trim()
            .strip_prefix("// cortado-plugin-version:")
            .or_else(|| line.trim().strip_prefix("# cortado-plugin-version:"))
        {
            return rest.trim().parse().ok();
        }
    }
    None
}

/// Checks whether the cortado-opencode plugin is installed.
///
/// Checks two locations (in order):
/// 1. File exists at `~/.config/opencode/plugins/cortado-opencode.ts`
/// 2. OpenCode's resolved config contains a "cortado" plugin entry
///
/// The first check is fast (filesystem only). The second handles users who
/// installed via npm or the config file directly.
///
/// When installed via the file path, also checks if the on-disk plugin is
/// outdated compared to the version embedded in this binary.
#[tauri::command]
pub fn check_opencode_plugin() -> SetupCheckResult {
    // Fast path: check if the plugin file exists in the global plugins dir.
    if let Some(dir) = opencode_plugins_dir() {
        let plugin_path = dir.join(OPENCODE_PLUGIN_FILENAME);
        if plugin_path.exists() {
            let outdated = std::fs::read_to_string(&plugin_path)
                .map(|content| is_plugin_outdated(&content, OPENCODE_PLUGIN_SOURCE))
                .unwrap_or(false);
            return SetupCheckResult {
                ready: true,
                outdated,
            };
        }
    }

    // Slow path: check OpenCode's resolved config for any cortado plugin entry.
    let output = match Command::new("opencode")
        .args(["debug", "config"])
        .env("NO_COLOR", "1")
        .output()
    {
        Ok(out) if out.status.success() => out,
        _ => {
            return SetupCheckResult {
                ready: false,
                outdated: false,
            }
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    SetupCheckResult {
        ready: config_has_cortado_plugin(&stdout),
        // Can't determine version when installed via config (not our file).
        outdated: false,
    }
}

/// Returns true if the on-disk plugin is older than the embedded source.
pub(crate) fn is_plugin_outdated(on_disk: &str, embedded: &str) -> bool {
    let disk_version = parse_plugin_version(on_disk);
    let embedded_version = parse_plugin_version(embedded);
    match (disk_version, embedded_version) {
        (Some(d), Some(e)) => d < e,
        // No version header on disk means an old pre-versioned plugin.
        (None, Some(_)) => true,
        _ => false,
    }
}

/// Checks whether an OpenCode config JSON string contains a cortado plugin entry.
///
/// Plugin entries can be plain strings (`"cortado-opencode"`) or tuples
/// (`["cortado-opencode", { ... }]`). Any entry whose name contains "cortado"
/// is considered a match.
fn config_has_cortado_plugin(config_json: &str) -> bool {
    let config: serde_json::Value = match serde_json::from_str(config_json) {
        Ok(v) => v,
        Err(_) => return false,
    };

    config
        .get("plugin")
        .and_then(|p| p.as_array())
        .map(|plugins| {
            plugins.iter().any(|entry| {
                let name = entry
                    .as_str()
                    .or_else(|| entry.as_array().and_then(|a| a.first()?.as_str()));
                name.map(|n| n.contains("cortado")).unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Installs the cortado-opencode plugin into OpenCode's global plugins directory.
///
/// Writes the embedded plugin source to `~/.config/opencode/plugins/cortado-opencode.ts`.
/// Creates the directory if it doesn't exist. Overwrites any existing file (idempotent).
#[tauri::command]
pub fn install_opencode_plugin() -> SetupInstallResult {
    let plugins_dir = match opencode_plugins_dir() {
        Some(dir) => dir,
        None => {
            return SetupInstallResult {
                success: false,
                error: Some("Could not determine home directory".to_string()),
            }
        }
    };

    if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
        return SetupInstallResult {
            success: false,
            error: Some(format!(
                "Failed to create plugins directory {}: {e}",
                plugins_dir.display()
            )),
        };
    }

    let plugin_path = plugins_dir.join(OPENCODE_PLUGIN_FILENAME);
    if let Err(e) = std::fs::write(&plugin_path, OPENCODE_PLUGIN_SOURCE) {
        return SetupInstallResult {
            success: false,
            error: Some(format!(
                "Failed to write plugin to {}: {e}",
                plugin_path.display()
            )),
        };
    }

    SetupInstallResult {
        success: true,
        error: None,
    }
}

/// Checks whether the Cortado plugin is installed in Copilot CLI.
///
/// Runs `copilot plugin list` and checks if "cortado" appears in the output.
/// When installed, also checks the on-disk hook script for version staleness.
#[tauri::command]
pub fn check_copilot_extension() -> SetupCheckResult {
    // Run `copilot plugin list` to check if our plugin is registered.
    let output = match Command::new("copilot")
        .args(["plugin", "list"])
        .env("NO_COLOR", "1")
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            return SetupCheckResult {
                ready: false,
                outdated: false,
            }
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.to_lowercase().contains("cortado") {
        return SetupCheckResult {
            ready: false,
            outdated: false,
        };
    }

    // Plugin is installed — check if the hook script is outdated.
    let outdated = copilot_plugin_dir()
        .map(|dir| dir.join("cortado-hook.sh"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| is_plugin_outdated(&content, COPILOT_HOOK_SCRIPT))
        .unwrap_or(false);

    SetupCheckResult {
        ready: true,
        outdated,
    }
}

/// Installs the Cortado plugin into Copilot CLI.
///
/// Writes the embedded plugin files to a temporary directory, then runs
/// `copilot plugin install <path>`. Uninstalls any existing version first
/// to ensure a clean install.
#[tauri::command]
pub fn install_copilot_extension() -> SetupInstallResult {
    let tmp_base =
        std::env::temp_dir().join(format!("cortado-copilot-install-{}", std::process::id()));
    let plugin_dir = tmp_base.join("cortado");

    // Clean up any leftover temp dir from a previous attempt.
    let _ = std::fs::remove_dir_all(&tmp_base);

    if let Err(e) = std::fs::create_dir_all(&plugin_dir) {
        return SetupInstallResult {
            success: false,
            error: Some(format!("Failed to create plugin directory: {e}")),
        };
    }

    // Write all plugin files.
    let files = [
        ("plugin.json", COPILOT_PLUGIN_JSON),
        ("hooks.json", COPILOT_HOOKS_JSON),
        ("cortado-hook.sh", COPILOT_HOOK_SCRIPT),
    ];
    for (name, content) in &files {
        let path = plugin_dir.join(name);
        if let Err(e) = std::fs::write(&path, content) {
            return SetupInstallResult {
                success: false,
                error: Some(format!("Failed to write {name}: {e}")),
            };
        }
    }

    // Make the hook script executable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let hook_path = plugin_dir.join("cortado-hook.sh");
        if let Err(e) = std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))
        {
            return SetupInstallResult {
                success: false,
                error: Some(format!("Failed to set hook permissions: {e}")),
            };
        }
    }

    // Uninstall any existing version first (ignore errors — may not be installed).
    let _ = Command::new("copilot")
        .args(["plugin", "uninstall", "cortado"])
        .output();

    // Install via `copilot plugin install <path>`.
    let output = match Command::new("copilot")
        .args(["plugin", "install"])
        .arg(&plugin_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            return SetupInstallResult {
                success: false,
                error: Some(format!("Failed to run `copilot plugin install`: {e}")),
            }
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return SetupInstallResult {
            success: false,
            error: Some(format!(
                "`copilot plugin install` failed: {}{}",
                stderr.trim(),
                if stdout.trim().is_empty() {
                    String::new()
                } else {
                    format!("\n{}", stdout.trim())
                }
            )),
        };
    }

    // Clean up temp directory.
    let _ = std::fs::remove_dir_all(&tmp_base);

    SetupInstallResult {
        success: true,
        error: None,
    }
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

    let feed = match feed::create_feed(&feed_config) {
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
                name: "Health".into(),
                feed_type: "http-health".into(),
                interval: Some("30s".into()),
                retain: Some("1h".into()),
                notify: Some(false),
                type_specific: [(
                    "url".into(),
                    serde_json::json!("https://example.com/health"),
                )]
                .into_iter()
                .collect(),
                fields: [(
                    "status".into(),
                    FieldOverrideDto {
                        visible: Some(true),
                        label: Some("Health Status".into()),
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
        assert_eq!(parsed[1].name, "Health");
        assert_eq!(
            parsed[1]
                .field_overrides
                .get("status")
                .unwrap()
                .label
                .as_deref(),
            Some("Health Status")
        );
    }

    #[test]
    fn toml_quote_escapes_special_chars() {
        assert_eq!(toml_quote("hello"), "\"hello\"");
        assert_eq!(toml_quote("he\"llo"), "\"he\\\"llo\"");
        assert_eq!(toml_quote("back\\slash"), "\"back\\\\slash\"");
    }

    #[test]
    fn duration_to_string_zero_seconds() {
        assert_eq!(duration_to_string(Duration::from_secs(0)), "0s");
    }

    #[test]
    fn json_value_to_toml_inline_covers_all_types() {
        assert_eq!(
            json_value_to_toml_inline(&serde_json::json!("hello")),
            "\"hello\""
        );
        assert_eq!(json_value_to_toml_inline(&serde_json::json!(42)), "42");
        assert_eq!(json_value_to_toml_inline(&serde_json::json!(1.5)), "1.5");
        assert_eq!(json_value_to_toml_inline(&serde_json::json!(true)), "true");
        assert_eq!(json_value_to_toml_inline(&serde_json::Value::Null), "\"\"");
        assert_eq!(
            json_value_to_toml_inline(&serde_json::json!(["a", "b"])),
            "[\"a\", \"b\"]"
        );
        assert_eq!(
            json_value_to_toml_inline(&serde_json::json!({"key": "val"})),
            "{}"
        );
    }

    #[test]
    fn feed_config_to_dto_preserves_all_fields() {
        use crate::feed::config::{FeedConfig, FieldOverride};

        let mut type_specific = toml::Table::new();
        type_specific.insert(
            "repo".to_string(),
            toml::Value::String("org/repo".to_string()),
        );

        let config = FeedConfig {
            name: "My PRs".to_string(),
            feed_type: "github-pr".to_string(),
            interval: Some(Duration::from_secs(300)),
            retain: Some(Duration::from_secs(7200)),
            notify: Some(false),
            type_specific,
            field_overrides: [(
                "labels".to_string(),
                FieldOverride {
                    visible: Some(false),
                    label: Some("Tags".to_string()),
                },
            )]
            .into_iter()
            .collect(),
        };

        let dto = feed_config_to_dto(&config);
        assert_eq!(dto.name, "My PRs");
        assert_eq!(dto.feed_type, "github-pr");
        assert_eq!(dto.interval.as_deref(), Some("5m"));
        assert_eq!(dto.retain.as_deref(), Some("2h"));
        assert_eq!(dto.notify, Some(false));
        assert_eq!(
            dto.type_specific.get("repo"),
            Some(&serde_json::json!("org/repo"))
        );
        assert_eq!(dto.fields.get("labels").unwrap().visible, Some(false));
        assert_eq!(
            dto.fields.get("labels").unwrap().label.as_deref(),
            Some("Tags")
        );
    }

    #[test]
    fn dto_to_toml_document_omits_none_fields() {
        let feeds = vec![FeedConfigDto {
            name: "Minimal".into(),
            feed_type: "http-health".into(),
            interval: None,
            retain: None,
            notify: None,
            type_specific: [("url".into(), serde_json::json!("https://example.com"))]
                .into_iter()
                .collect(),
            fields: HashMap::new(),
        }];

        let toml_str = dto_to_toml_document(&feeds);
        assert!(!toml_str.contains("interval"));
        assert!(!toml_str.contains("retain"));
        assert!(!toml_str.contains("notify"));
        assert!(toml_str.contains("https://example.com"));
    }

    #[test]
    fn activity_to_test_activity_extracts_status() {
        use crate::feed::{Activity, Field, FieldValue, StatusKind};

        let a = Activity {
            id: "test".to_string(),
            title: "Test PR".to_string(),
            fields: vec![
                Field {
                    name: "review".to_string(),
                    label: "Review".to_string(),
                    value: FieldValue::Status {
                        value: "approved".to_string(),
                        kind: StatusKind::AttentionPositive,
                    },
                },
                Field {
                    name: "labels".to_string(),
                    label: "Labels".to_string(),
                    value: FieldValue::Text {
                        value: "wip".to_string(),
                    },
                },
            ],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        };

        let ta = activity_to_test_activity(&a);
        assert_eq!(ta.title, "Test PR");
        assert_eq!(ta.status.as_deref(), Some("approved"));
    }

    #[test]
    fn activity_to_test_activity_no_status_field() {
        use crate::feed::{Activity, Field, FieldValue};

        let a = Activity {
            id: "test".to_string(),
            title: "Test".to_string(),
            fields: vec![Field {
                name: "output".to_string(),
                label: "Output".to_string(),
                value: FieldValue::Text {
                    value: "hello".to_string(),
                },
            }],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        };

        let ta = activity_to_test_activity(&a);
        assert!(ta.status.is_none());
    }

    // ── config_has_cortado_plugin ────────────────────────────────────

    #[test]
    fn plugin_check_string_entry_with_cortado() {
        let json = r#"{ "plugin": ["github:oribarilan/cortado/plugins/opencode"] }"#;
        assert!(config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_npm_package_name() {
        let json = r#"{ "plugin": ["cortado-opencode"] }"#;
        assert!(config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_tuple_entry() {
        let json = r#"{ "plugin": [["cortado-opencode", { "some": "option" }]] }"#;
        assert!(config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_mixed_plugins_finds_cortado() {
        let json = r#"{ "plugin": ["opencode-helicone", "cortado-opencode", "other-plugin"] }"#;
        assert!(config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_no_cortado_entry() {
        let json = r#"{ "plugin": ["opencode-helicone", "other-plugin"] }"#;
        assert!(!config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_empty_plugin_array() {
        let json = r#"{ "plugin": [] }"#;
        assert!(!config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_no_plugin_key() {
        let json = r#"{ "provider": {} }"#;
        assert!(!config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_malformed_json() {
        assert!(!config_has_cortado_plugin("not valid json"));
    }

    #[test]
    fn plugin_check_empty_string() {
        assert!(!config_has_cortado_plugin(""));
    }

    #[test]
    fn plugin_check_plugin_is_not_array() {
        let json = r#"{ "plugin": "cortado-opencode" }"#;
        assert!(!config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_tuple_without_cortado() {
        let json = r#"{ "plugin": [["other-plugin", {}]] }"#;
        assert!(!config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_numeric_entry_skipped() {
        let json = r#"{ "plugin": [42, "cortado-opencode"] }"#;
        assert!(config_has_cortado_plugin(json));
    }

    #[test]
    fn plugin_check_empty_tuple_skipped() {
        let json = r#"{ "plugin": [[], "cortado-opencode"] }"#;
        assert!(config_has_cortado_plugin(json));
    }

    // ── install / check opencode plugin ─────────────────────────────

    #[test]
    fn install_writes_plugin_file_to_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");

        // Simulate what install_opencode_plugin does.
        std::fs::create_dir_all(&plugins_dir).unwrap();
        let plugin_path = plugins_dir.join(OPENCODE_PLUGIN_FILENAME);
        std::fs::write(&plugin_path, OPENCODE_PLUGIN_SOURCE).unwrap();

        assert!(plugin_path.exists());
        let content = std::fs::read_to_string(&plugin_path).unwrap();
        assert!(content.contains("cortado-opencode"));
        assert!(content.contains("export default CortadoPlugin"));
    }

    #[test]
    fn install_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();
        let plugin_path = plugins_dir.join(OPENCODE_PLUGIN_FILENAME);

        // Write twice — should succeed both times with same content.
        std::fs::write(&plugin_path, OPENCODE_PLUGIN_SOURCE).unwrap();
        std::fs::write(&plugin_path, OPENCODE_PLUGIN_SOURCE).unwrap();

        let content = std::fs::read_to_string(&plugin_path).unwrap();
        assert_eq!(content, OPENCODE_PLUGIN_SOURCE);
    }

    #[test]
    fn embedded_plugin_source_is_valid() {
        // Verify the embedded source is non-empty and contains expected markers.
        assert!(!OPENCODE_PLUGIN_SOURCE.is_empty());
        assert!(OPENCODE_PLUGIN_SOURCE.contains("export default CortadoPlugin"));
        assert!(OPENCODE_PLUGIN_SOURCE.contains("harness"));
        assert!(OPENCODE_PLUGIN_SOURCE.contains("session.status"));
    }

    #[test]
    fn opencode_plugins_dir_returns_path() {
        let dir = opencode_plugins_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with(".config/opencode/plugins"));
    }

    // ── parse_plugin_version ────────────────────────────────────────

    #[test]
    fn parse_version_from_first_line() {
        let source = "// cortado-plugin-version: 2\n// rest of file";
        assert_eq!(parse_plugin_version(source), Some(2));
    }

    #[test]
    fn parse_version_from_later_line() {
        let source = "// comment\n// cortado-plugin-version: 5\ncode";
        assert_eq!(parse_plugin_version(source), Some(5));
    }

    #[test]
    fn parse_version_none_when_missing() {
        let source = "// no version here\nexport default Foo;";
        assert_eq!(parse_plugin_version(source), None);
    }

    #[test]
    fn parse_version_none_for_empty_string() {
        assert_eq!(parse_plugin_version(""), None);
    }

    #[test]
    fn parse_version_ignores_after_line_5() {
        let source = "a\nb\nc\nd\ne\n// cortado-plugin-version: 3";
        assert_eq!(parse_plugin_version(source), None);
    }

    #[test]
    fn parse_version_handles_extra_whitespace() {
        let source = "  // cortado-plugin-version:   7  \n";
        assert_eq!(parse_plugin_version(source), Some(7));
    }

    // ── is_plugin_outdated ──────────────────────────────────────────

    #[test]
    fn outdated_when_disk_older() {
        assert!(is_plugin_outdated(
            "// cortado-plugin-version: 1\n",
            "// cortado-plugin-version: 2\n"
        ));
    }

    #[test]
    fn not_outdated_when_same_version() {
        assert!(!is_plugin_outdated(
            "// cortado-plugin-version: 2\n",
            "// cortado-plugin-version: 2\n"
        ));
    }

    #[test]
    fn not_outdated_when_disk_newer() {
        assert!(!is_plugin_outdated(
            "// cortado-plugin-version: 3\n",
            "// cortado-plugin-version: 2\n"
        ));
    }

    #[test]
    fn outdated_when_no_version_on_disk() {
        assert!(is_plugin_outdated(
            "// no version\nexport default Foo;\n",
            "// cortado-plugin-version: 2\n"
        ));
    }

    #[test]
    fn not_outdated_when_no_version_in_embedded() {
        assert!(!is_plugin_outdated(
            "// cortado-plugin-version: 1\n",
            "// no version\n"
        ));
    }

    #[test]
    fn not_outdated_when_neither_has_version() {
        assert!(!is_plugin_outdated("// old\n", "// new\n"));
    }

    #[test]
    fn embedded_plugin_has_version_header() {
        assert!(
            parse_plugin_version(OPENCODE_PLUGIN_SOURCE).is_some(),
            "Embedded plugin source must have a cortado-plugin-version header"
        );
    }

    // ── copilot plugin ────────────────────────────────────────────

    #[test]
    fn copilot_plugin_dir_returns_path() {
        let dir = copilot_plugin_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.ends_with(".copilot/installed-plugins/_direct/copilot"));
    }

    #[test]
    fn embedded_copilot_hook_script_is_valid() {
        assert!(!COPILOT_HOOK_SCRIPT.is_empty());
        assert!(COPILOT_HOOK_SCRIPT.contains("harness"));
        assert!(COPILOT_HOOK_SCRIPT.contains("copilot"));
    }

    #[test]
    fn embedded_copilot_hook_has_version_header() {
        assert!(
            parse_plugin_version(COPILOT_HOOK_SCRIPT).is_some(),
            "Embedded Copilot hook script must have a cortado-plugin-version header"
        );
    }

    #[test]
    fn embedded_copilot_plugin_json_is_valid() {
        let v: serde_json::Value =
            serde_json::from_str(COPILOT_PLUGIN_JSON).expect("plugin.json must be valid JSON");
        assert_eq!(v["name"], "cortado");
    }

    #[test]
    fn embedded_copilot_hooks_json_is_valid() {
        let v: serde_json::Value =
            serde_json::from_str(COPILOT_HOOKS_JSON).expect("hooks.json must be valid JSON");
        assert_eq!(v["version"], 1);
        assert!(v["hooks"]["sessionStart"].is_array());
        assert!(v["hooks"]["sessionEnd"].is_array());
    }
}
