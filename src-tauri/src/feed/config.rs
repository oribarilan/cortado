use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use toml::{Table, Value};

const CONFIG_DIR: &str = ".config/cortado";
const CONFIG_FILE: &str = "feeds.toml";

/// Optional display overrides for a specific field.
#[derive(Debug, Clone)]
pub struct FieldOverride {
    pub visible: Option<bool>,
    pub label: Option<String>,
}

/// Parsed feed configuration entry from `feeds.toml`.
#[derive(Debug, Clone)]
pub struct FeedConfig {
    pub name: String,
    pub feed_type: String,
    pub interval: Option<u64>,
    pub type_specific: Table,
    pub field_overrides: HashMap<String, FieldOverride>,
}

/// Returns the canonical config file path (`~/.config/cortado/feeds.toml`).
pub fn feeds_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("could not resolve home directory"))?;

    Ok(home_dir.join(CONFIG_DIR).join(CONFIG_FILE))
}

/// Loads feed configuration entries from the user config file.
///
/// If the file does not exist, this returns an empty list.
pub fn load_feeds_config() -> Result<Vec<FeedConfig>> {
    let config_path = feeds_config_path()?;

    load_feeds_config_from_path(&config_path)
}

fn load_feeds_config_from_path(config_path: &Path) -> Result<Vec<FeedConfig>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("failed reading {}", config_path.display()))?;

    parse_feeds_config_toml(&raw)
}

fn parse_feeds_config_toml(raw: &str) -> Result<Vec<FeedConfig>> {
    let parsed = raw
        .parse::<Value>()
        .context("invalid TOML in feeds config. expected [[feed]] entries")?;

    let root = parsed
        .as_table()
        .ok_or_else(|| anyhow!("feeds config root must be a TOML table"))?;

    let Some(feed_value) = root.get("feed") else {
        return Ok(Vec::new());
    };

    let feed_array = feed_value
        .as_array()
        .ok_or_else(|| anyhow!("`feed` must be an array of tables (`[[feed]]`)"))?;

    let mut configs = Vec::with_capacity(feed_array.len());
    let mut seen_names = HashSet::new();

    for (index, feed_entry) in feed_array.iter().enumerate() {
        let feed_table = feed_entry
            .as_table()
            .ok_or_else(|| anyhow!("feed entry at index {} must be a table (`[[feed]]`)", index))?;

        let name = required_string(feed_table, "name", index)?;
        let feed_type = required_string(feed_table, "type", index)?;

        if !seen_names.insert(name.to_string()) {
            bail!("duplicate feed name `{name}` in config");
        }

        let interval = optional_positive_integer(feed_table, "interval", index)?;
        let field_overrides = parse_field_overrides(feed_table, index)?;

        let mut type_specific = feed_table.clone();
        type_specific.remove("name");
        type_specific.remove("type");
        type_specific.remove("interval");
        type_specific.remove("fields");

        configs.push(FeedConfig {
            name: name.to_string(),
            feed_type: feed_type.to_string(),
            interval,
            type_specific,
            field_overrides,
        });
    }

    Ok(configs)
}

fn parse_field_overrides(
    feed_table: &Table,
    feed_index: usize,
) -> Result<HashMap<String, FieldOverride>> {
    let Some(fields_value) = feed_table.get("fields") else {
        return Ok(HashMap::new());
    };

    let fields_table = fields_value
        .as_table()
        .ok_or_else(|| anyhow!("feed[{feed_index}].fields must be a table"))?;

    let mut overrides = HashMap::new();

    for (field_name, field_value) in fields_table {
        let override_table = field_value.as_table().ok_or_else(|| {
            anyhow!(
                "feed[{feed_index}].fields.{field_name} must be a table with optional `visible` and `label`"
            )
        })?;

        let visible = match override_table.get("visible") {
            Some(value) => Some(value.as_bool().ok_or_else(|| {
                anyhow!("feed[{feed_index}].fields.{field_name}.visible must be a boolean")
            })?),
            None => None,
        };

        let label = match override_table.get("label") {
            Some(value) => Some(
                value
                    .as_str()
                    .ok_or_else(|| {
                        anyhow!("feed[{feed_index}].fields.{field_name}.label must be a string")
                    })?
                    .to_string(),
            ),
            None => None,
        };

        overrides.insert(field_name.clone(), FieldOverride { visible, label });
    }

    Ok(overrides)
}

fn required_string<'a>(table: &'a Table, key: &str, feed_index: usize) -> Result<&'a str> {
    table
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("feed[{feed_index}] missing required string field `{key}`"))
}

fn optional_positive_integer(table: &Table, key: &str, feed_index: usize) -> Result<Option<u64>> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let parsed = value
        .as_integer()
        .ok_or_else(|| anyhow!("feed[{feed_index}].{key} must be an integer"))?;

    if parsed <= 0 {
        bail!("feed[{feed_index}].{key} must be greater than zero");
    }

    Ok(Some(parsed as u64))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{load_feeds_config_from_path, parse_feeds_config_toml};

    #[test]
    fn parse_valid_config_with_overrides() {
        let raw = r#"
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "personal/cortado"
interval = 60

[feed.fields.labels]
visible = false
label = "Tags"

[[feed]]
name = "Disk usage"
type = "shell"
command = "df -h /"
"#;

        let configs = parse_feeds_config_toml(raw).expect("valid config should parse");
        assert_eq!(configs.len(), 2);

        let github = &configs[0];
        assert_eq!(github.name, "My PRs");
        assert_eq!(github.feed_type, "github-pr");
        assert_eq!(github.interval, Some(60));
        assert_eq!(
            github
                .type_specific
                .get("repo")
                .and_then(|value| value.as_str()),
            Some("personal/cortado")
        );

        let labels_override = github
            .field_overrides
            .get("labels")
            .expect("labels override should exist");
        assert_eq!(labels_override.visible, Some(false));
        assert_eq!(labels_override.label.as_deref(), Some("Tags"));

        let shell = &configs[1];
        assert_eq!(shell.name, "Disk usage");
        assert_eq!(shell.feed_type, "shell");
        assert_eq!(shell.interval, None);
    }

    #[test]
    fn parse_errors_on_missing_required_keys() {
        let missing_name = r#"
[[feed]]
type = "shell"
command = "echo hi"
"#;

        let error = match parse_feeds_config_toml(missing_name) {
            Ok(_) => panic!("missing name should fail"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("missing required string field `name`"));

        let missing_type = r#"
[[feed]]
name = "No type"
command = "echo hi"
"#;

        let error = match parse_feeds_config_toml(missing_type) {
            Ok(_) => panic!("missing type should fail"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("missing required string field `type`"));
    }

    #[test]
    fn parse_errors_on_invalid_interval() {
        let non_integer_interval = r#"
[[feed]]
name = "Bad interval"
type = "shell"
command = "echo hi"
interval = "fast"
"#;

        let error = match parse_feeds_config_toml(non_integer_interval) {
            Ok(_) => panic!("non-integer interval should fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("interval must be an integer"));

        let non_positive_interval = r#"
[[feed]]
name = "Zero interval"
type = "shell"
command = "echo hi"
interval = 0
"#;

        let error = match parse_feeds_config_toml(non_positive_interval) {
            Ok(_) => panic!("non-positive interval should fail"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("interval must be greater than zero"));
    }

    #[test]
    fn parse_errors_on_duplicate_feed_names() {
        let raw = r#"
[[feed]]
name = "Dup"
type = "shell"
command = "echo hi"

[[feed]]
name = "Dup"
type = "github-pr"
repo = "personal/cortado"
"#;

        let error = match parse_feeds_config_toml(raw) {
            Ok(_) => panic!("duplicate names should fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("duplicate feed name `Dup`"));
    }

    #[test]
    fn load_from_missing_file_returns_empty_list() {
        let mut path = std::env::temp_dir();
        path.push(unique_missing_filename());

        if path.exists() {
            let _ = fs::remove_file(&path);
        }

        let configs = load_feeds_config_from_path(&path).expect("missing file should be ok");
        assert!(configs.is_empty());
    }

    fn unique_missing_filename() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();

        PathBuf::from(format!("cortado-missing-{now}.toml"))
    }
}
