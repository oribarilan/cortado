use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
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

    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("failed reading {}", config_path.display()))?;

    let parsed = raw.parse::<Value>().with_context(|| {
        format!(
            "invalid TOML in {}. expected [[feed]] entries",
            config_path.display()
        )
    })?;

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
