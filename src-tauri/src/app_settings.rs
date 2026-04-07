use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::feed::StatusKind;

const SETTINGS_FILE: &str = "settings.toml";

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_text_size() -> String {
    "m".to_string()
}

fn default_global_hotkey() -> String {
    "super+shift+space".to_string()
}

/// Global app settings persisted in `~/.config/cortado/settings.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub panel: PanelSettings,
    pub notifications: NotificationSettings,
    pub focus: FocusSettings,
}

/// General preferences under `[general]` in settings.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GeneralSettings {
    /// Theme preference: `"system"`, `"light"`, or `"dark"`.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Text size preference: `"xs"`, `"s"`, `"m"`, `"l"`, or `"xl"`.
    #[serde(default = "default_text_size")]
    pub text_size: String,
    #[serde(default = "default_true")]
    pub show_menubar: bool,
    /// Global hotkey to toggle the panel. Empty string = disabled.
    /// Format: Tauri shortcut string, e.g. `"super+shift+space"`.
    #[serde(default = "default_global_hotkey")]
    pub global_hotkey: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            text_size: default_text_size(),
            show_menubar: true,
            global_hotkey: default_global_hotkey(),
        }
    }
}

/// Main screen (panel) display preferences under `[panel]` in settings.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PanelSettings {
    /// Show the "Needs Attention" priority section at the top of the activity list.
    #[serde(default = "default_true")]
    pub show_priority_section: bool,
    /// Show feeds that have no activities.
    #[serde(default)]
    pub show_empty_feeds: bool,
}

impl Default for PanelSettings {
    fn default() -> Self {
        Self {
            show_priority_section: true,
            show_empty_feeds: false,
        }
    }
}

/// Which status changes should trigger a notification.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum NotificationMode {
    /// Notify on completion and attention: Idle, AttentionPositive, AttentionNegative.
    /// Skips transient in-progress states (Running, Waiting).
    #[default]
    WorthKnowing,
    /// Only when it's my turn: AttentionPositive, AttentionNegative.
    NeedAttention,
    /// Any rollup kind change fires a notification.
    All,
    /// Only when the new kind is in the configured set.
    SpecificKinds {
        #[serde(default)]
        kinds: Vec<StatusKind>,
    },
}

/// How notifications are batched before delivery.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryPreset {
    /// One notification per activity change.
    Immediate,
    /// At most one notification per feed per poll cycle.
    #[default]
    Grouped,
}

/// Per-feed notification override resolved from config.
///
/// Determines whether a feed uses the global notification mode, overrides it
/// with a feed-specific mode, or disables notifications entirely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedNotifyOverride {
    /// Notifications disabled for this feed (`notify = false`).
    Off,
    /// Use the global notification mode (`notify = true` or absent).
    Global,
    /// Override with a specific notification mode (`notify = "worth_knowing"` etc.).
    Mode(NotificationMode),
}

/// Notification preferences within global settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NotificationSettings {
    pub enabled: bool,
    #[serde(flatten)]
    pub mode: NotificationMode,
    pub delivery: DeliveryPreset,
    pub notify_new_activities: bool,
    pub notify_removed_activities: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: NotificationMode::default(),
            delivery: DeliveryPreset::default(),
            notify_new_activities: true,
            notify_removed_activities: false,
        }
    }
}

/// Focus settings under `[focus]` in settings.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FocusSettings {
    /// Whether to use tmux pane switching when tmux is detected.
    pub tmux_enabled: bool,
    /// Whether to attempt the accessibility strategy (user opt-in).
    /// Even if true, requires OS-level permission to actually work.
    pub accessibility_enabled: bool,
}

impl Default for FocusSettings {
    fn default() -> Self {
        Self {
            tmux_enabled: true,
            accessibility_enabled: false,
        }
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

/// Returns the canonical settings file path.
pub fn settings_path() -> Result<PathBuf> {
    Ok(crate::app_env::config_dir().join(SETTINGS_FILE))
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

/// Appearance payload emitted to all windows when theme or text size changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearancePayload {
    pub theme: String,
    pub text_size: String,
}

/// Tauri command: save app settings (persists to file and updates live state).
#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: tauri::State<'_, AppSettingsState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    save_settings_to_file(&settings).map_err(|e| e.to_string())?;

    let payload = AppearancePayload {
        theme: settings.general.theme.clone(),
        text_size: settings.general.text_size.clone(),
    };

    state.update(settings).await;

    // Notify all windows so they can update data-theme / data-text-size attributes.
    if let Err(err) = app_handle.emit("appearance-changed", &payload) {
        eprintln!("failed emitting appearance-changed event: {err}");
    }

    Ok(())
}

/// Tauri command: return the app settings file path.
#[tauri::command]
pub fn get_settings_path() -> Result<String, String> {
    let path = settings_path().map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

/// Tauri command: open the settings file in the default editor.
#[tauri::command]
pub fn open_settings_file() -> Result<(), String> {
    let path = settings_path().map_err(|e| e.to_string())?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed creating config directory: {e}"))?;
        }
        fs::write(&path, "").map_err(|e| format!("failed creating settings file: {e}"))?;
    }

    Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("failed to open settings file: {e}"))?;

    Ok(())
}

/// Tauri command: reveal the settings file in Finder.
#[tauri::command]
pub fn reveal_settings_file() -> Result<(), String> {
    let path = settings_path().map_err(|e| e.to_string())?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed creating config directory: {e}"))?;
        }
        fs::write(&path, "").map_err(|e| format!("failed creating settings file: {e}"))?;
    }

    Command::new("open")
        .arg("-R")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("failed to reveal settings file: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::feed::StatusKind;

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
        assert_eq!(settings.notifications.mode, NotificationMode::WorthKnowing);
        assert_eq!(settings.notifications.delivery, DeliveryPreset::Grouped);
        assert!(settings.notifications.notify_new_activities);
        assert!(!settings.notifications.notify_removed_activities);
    }

    #[test]
    fn round_trip_preserves_all_values() {
        let path = temp_settings_path("roundtrip");

        let settings = AppSettings {
            notifications: NotificationSettings {
                enabled: false,
                mode: NotificationMode::NeedAttention,
                delivery: DeliveryPreset::Immediate,
                notify_new_activities: false,
                notify_removed_activities: true,
            },
            ..AppSettings::default()
        };

        save_settings_to_path(&settings, &path).expect("save should succeed");
        let loaded = load_settings_from_path(&path).expect("load should succeed");

        assert!(!loaded.notifications.enabled);
        assert_eq!(loaded.notifications.mode, NotificationMode::NeedAttention);
        assert_eq!(loaded.notifications.delivery, DeliveryPreset::Immediate);
        assert!(!loaded.notifications.notify_new_activities);
        assert!(loaded.notifications.notify_removed_activities);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn round_trip_specific_kinds() {
        let path = temp_settings_path("specific");

        let settings = AppSettings {
            notifications: NotificationSettings {
                enabled: true,
                mode: NotificationMode::SpecificKinds {
                    kinds: vec![StatusKind::AttentionNegative, StatusKind::AttentionPositive],
                },
                delivery: DeliveryPreset::Grouped,
                notify_new_activities: true,
                notify_removed_activities: false,
            },
            ..AppSettings::default()
        };

        save_settings_to_path(&settings, &path).expect("save should succeed");
        let loaded = load_settings_from_path(&path).expect("load should succeed");

        match &loaded.notifications.mode {
            NotificationMode::SpecificKinds { kinds } => {
                assert_eq!(kinds.len(), 2);
                assert!(kinds.contains(&StatusKind::AttentionNegative));
                assert!(kinds.contains(&StatusKind::AttentionPositive));
            }
            other => panic!("expected SpecificKinds, got {:?}", other),
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn backup_created_on_overwrite() {
        let path = temp_settings_path("backup");

        let original = AppSettings::default();
        save_settings_to_path(&original, &path).expect("first save");

        let updated = AppSettings {
            notifications: NotificationSettings {
                enabled: false,
                ..NotificationSettings::default()
            },
            ..AppSettings::default()
        };
        save_settings_to_path(&updated, &path).expect("second save");

        let backup_path = path.with_extension("toml.bak");
        assert!(backup_path.exists(), "backup file should exist");

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
        assert_eq!(settings.notifications.mode, NotificationMode::WorthKnowing);

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

    #[test]
    fn round_trip_general_section() {
        let path = temp_settings_path("general");

        let settings = AppSettings {
            general: GeneralSettings {
                theme: "dark".to_string(),
                text_size: "l".to_string(),
                show_menubar: false,
                global_hotkey: "super+alt+KeyK".to_string(),
            },
            panel: PanelSettings {
                show_priority_section: false,
                show_empty_feeds: false,
            },
            ..AppSettings::default()
        };

        save_settings_to_path(&settings, &path).expect("save should succeed");
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.contains("[general]"), "should have [general] section");
        assert!(raw.contains("[panel]"), "should have [panel] section");

        let loaded = load_settings_from_path(&path).expect("load should succeed");
        assert_eq!(loaded.general.theme, "dark");
        assert_eq!(loaded.general.text_size, "l");
        assert!(!loaded.general.show_menubar);
        assert_eq!(loaded.general.global_hotkey.as_str(), "super+alt+KeyK");
        assert!(!loaded.panel.show_priority_section);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn global_hotkey_none_round_trips() {
        let path = temp_settings_path("hotkey-none");

        let settings = AppSettings {
            general: GeneralSettings {
                global_hotkey: String::new(),
                ..GeneralSettings::default()
            },
            ..AppSettings::default()
        };

        save_settings_to_path(&settings, &path).expect("save should succeed");
        let loaded = load_settings_from_path(&path).expect("load should succeed");
        assert!(loaded.general.global_hotkey.is_empty());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_global_hotkey_uses_default() {
        let path = temp_settings_path("hotkey-default");
        fs::write(&path, "[general]\ntheme = \"dark\"\n").expect("write");

        let loaded = load_settings_from_path(&path).expect("should use default");
        assert_eq!(loaded.general.global_hotkey.as_str(), "super+shift+space");

        let _ = fs::remove_file(&path);
    }
}
