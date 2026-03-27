use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

const CONFIG_DIR: &str = ".config/cortado";
const SETTINGS_FILE: &str = "settings.toml";

/// Global app settings persisted in `~/.config/cortado/settings.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub notifications: NotificationSettings,
}

/// Notification preferences within global settings.
///
/// Populated fully in task 03; for now carries the master toggle only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    pub enabled: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Thread-safe handle to live app settings.
///
/// The master notification toggle is read from this on every poll cycle,
/// so toggling it takes effect immediately without restart.
#[derive(Clone)]
pub struct AppSettingsState {
    inner: Arc<RwLock<AppSettings>>,
}

impl AppSettingsState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            inner: Arc::new(RwLock::new(settings)),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, AppSettings> {
        self.inner.read().await
    }

    pub async fn update(&self, settings: AppSettings) {
        *self.inner.write().await = settings;
    }
}

/// Returns the canonical settings file path (`~/.config/cortado/settings.toml`).
pub fn settings_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("could not resolve home directory"))?;
    Ok(home_dir.join(CONFIG_DIR).join(SETTINGS_FILE))
}

/// Loads settings from `~/.config/cortado/settings.toml`, using defaults if absent.
pub fn load_settings() -> Result<AppSettings> {
    let path = settings_path()?;
    load_settings_from_path(&path)
}

fn load_settings_from_path(path: &Path) -> Result<AppSettings> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let raw =
        fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))?;

    let settings: AppSettings =
        toml::from_str(&raw).with_context(|| "failed parsing settings.toml")?;

    Ok(settings)
}

/// Persists settings to `~/.config/cortado/settings.toml` with backup.
pub fn save_settings_to_file(settings: &AppSettings) -> Result<()> {
    let path = settings_path()?;
    save_settings_to_path(settings, &path)
}

fn save_settings_to_path(settings: &AppSettings, path: &Path) -> Result<()> {
    let toml_str =
        toml::to_string_pretty(settings).context("failed serializing settings to TOML")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating config directory: {}", parent.display()))?;
    }

    // Backup existing file before overwrite.
    if path.exists() {
        let backup_path = path.with_extension("toml.bak");
        fs::copy(path, &backup_path)
            .with_context(|| format!("failed backing up {}", path.display()))?;
    }

    fs::write(path, toml_str).with_context(|| format!("failed writing {}", path.display()))?;

    Ok(())
}

/// Tauri command: read current app settings.
#[tauri::command]
pub async fn get_settings(
    state: tauri::State<'_, AppSettingsState>,
) -> Result<AppSettings, String> {
    let settings = state.read().await.clone();
    Ok(settings)
}

/// Tauri command: save app settings (persists to file and updates live state).
#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: tauri::State<'_, AppSettingsState>,
) -> Result<(), String> {
    save_settings_to_file(&settings).map_err(|e| e.to_string())?;
    state.update(settings).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn temp_settings_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("cortado-test-settings-{name}-{ts}.toml"));
        path
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let path = temp_settings_path("missing");
        if path.exists() {
            let _ = fs::remove_file(&path);
        }

        let settings = load_settings_from_path(&path).expect("should return defaults");
        assert!(settings.notifications.enabled);
    }

    #[test]
    fn round_trip_preserves_values() {
        let path = temp_settings_path("roundtrip");

        let settings = AppSettings {
            notifications: NotificationSettings { enabled: false },
        };

        save_settings_to_path(&settings, &path).expect("save should succeed");
        let loaded = load_settings_from_path(&path).expect("load should succeed");

        assert!(!loaded.notifications.enabled);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn backup_created_on_overwrite() {
        let path = temp_settings_path("backup");

        let original = AppSettings::default();
        save_settings_to_path(&original, &path).expect("first save");

        let updated = AppSettings {
            notifications: NotificationSettings { enabled: false },
        };
        save_settings_to_path(&updated, &path).expect("second save");

        let backup_path = path.with_extension("toml.bak");
        assert!(backup_path.exists(), "backup file should exist");

        // Backup should contain the original (enabled=true)
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("enabled = true"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup_path);
    }

    #[test]
    fn missing_notifications_section_uses_defaults() {
        let path = temp_settings_path("partial");
        fs::write(&path, "# empty settings file\n").expect("write empty file");

        let settings = load_settings_from_path(&path).expect("should use defaults");
        assert!(settings.notifications.enabled);

        let _ = fs::remove_file(&path);
    }

    #[tokio::test]
    async fn state_update_is_visible_immediately() {
        let state = AppSettingsState::new(AppSettings::default());
        assert!(state.read().await.notifications.enabled);

        let mut updated = state.read().await.clone();
        updated.notifications.enabled = false;
        state.update(updated).await;

        assert!(!state.read().await.notifications.enabled);
    }
}
