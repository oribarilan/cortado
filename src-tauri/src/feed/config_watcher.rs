use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{watch, RwLock};

use crate::app_settings::{self, AppSettings, AppSettingsState};
use crate::feed::config::{self, FeedConfig};

const DEBOUNCE_DELAY: Duration = Duration::from_millis(500);

/// What kind of config change was detected.
#[derive(Debug, Clone, Default)]
pub struct ConfigChangeStatus {
    pub feeds_changed: bool,
    pub settings_changed: bool,
    /// Parse error for the changed file (if invalid TOML).
    pub parse_error: Option<String>,
}

impl ConfigChangeStatus {
    /// Returns true if any change (valid or invalid) was detected.
    #[allow(dead_code)] // Used in tests; available for future callers.
    pub fn has_any_change(&self) -> bool {
        self.feeds_changed || self.settings_changed || self.parse_error.is_some()
    }

    /// Human-readable description of what changed.
    pub fn change_description(&self) -> &'static str {
        match (self.feeds_changed, self.settings_changed) {
            (true, true) => "Config changed",
            (true, false) => "Feed config changed",
            (false, true) => "Settings changed",
            _ => "Config changed",
        }
    }
}

/// Shared state tracking config file changes.
#[derive(Clone)]
pub struct ConfigChangeState {
    inner: Arc<RwLock<ConfigChangeStatus>>,
}

impl ConfigChangeState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConfigChangeStatus::default())),
        }
    }

    pub async fn status(&self) -> ConfigChangeStatus {
        self.inner.read().await.clone()
    }

    async fn update(&self, status: ConfigChangeStatus) {
        *self.inner.write().await = status;
    }
}

/// Spawns a file watcher for feeds.toml and settings.toml.
///
/// On change (debounced 500ms), parses the changed file and compares to the
/// running config. Updates `ConfigChangeState` and bumps the update counter
/// to trigger a UI refresh.
pub fn spawn_config_watcher(
    state: ConfigChangeState,
    startup_feed_configs: Arc<Vec<FeedConfig>>,
    settings_state: AppSettingsState,
    update_tx: watch::Sender<u64>,
) {
    let feeds_path = config::feeds_config_path().ok();
    let settings_path = app_settings::settings_path().ok();

    tauri::async_runtime::spawn(async move {
        config_watch_loop(
            state,
            startup_feed_configs,
            settings_state,
            update_tx,
            feeds_path,
            settings_path,
        )
        .await;
    });
}

async fn config_watch_loop(
    state: ConfigChangeState,
    startup_feed_configs: Arc<Vec<FeedConfig>>,
    settings_state: AppSettingsState,
    update_tx: watch::Sender<u64>,
    feeds_path: Option<PathBuf>,
    settings_path: Option<PathBuf>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    let _watcher = setup_config_watcher(tx, &feeds_path, &settings_path);

    loop {
        // Wait for a file change event.
        if rx.recv().await.is_none() {
            break;
        }

        // Debounce: wait then drain any queued events.
        tokio::time::sleep(DEBOUNCE_DELAY).await;
        while rx.try_recv().is_ok() {}

        // Check both files against running config.
        let new_status = check_config_changes(
            &feeds_path,
            &settings_path,
            &startup_feed_configs,
            &settings_state,
        )
        .await;

        state.update(new_status).await;
        super::runtime::bump_update_counter(&update_tx);
    }
}

async fn check_config_changes(
    feeds_path: &Option<PathBuf>,
    settings_path: &Option<PathBuf>,
    startup_feed_configs: &[FeedConfig],
    settings_state: &AppSettingsState,
) -> ConfigChangeStatus {
    let mut status = ConfigChangeStatus::default();

    if let Some(ref path) = feeds_path {
        match check_feeds_changed(path, startup_feed_configs) {
            Ok(changed) => status.feeds_changed = changed,
            Err(err) => status.parse_error = Some(err),
        }
    }

    if let Some(ref path) = settings_path {
        match check_settings_changed(path, settings_state).await {
            Ok(changed) => status.settings_changed = changed,
            Err(err) => {
                if status.parse_error.is_none() {
                    status.parse_error = Some(err);
                }
            }
        }
    }

    status
}

fn check_feeds_changed(path: &PathBuf, startup_configs: &[FeedConfig]) -> Result<bool, String> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist → equivalent to empty config.
            return Ok(!startup_configs.is_empty());
        }
        Err(e) => return Err(format!("Failed to read feeds config: {e}")),
    };

    let new_configs =
        config::parse_feeds_config_str(&raw).map_err(|e| format!("Invalid feed config: {e}"))?;

    Ok(new_configs != startup_configs)
}

async fn check_settings_changed(
    path: &PathBuf,
    settings_state: &AppSettingsState,
) -> Result<bool, String> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist → equivalent to default settings.
            let current = settings_state.read().await.clone();
            return Ok(current != AppSettings::default());
        }
        Err(e) => return Err(format!("Failed to read settings: {e}")),
    };

    let new_settings: AppSettings =
        toml::from_str(&raw).map_err(|e| format!("Invalid settings: {e}"))?;

    let current = settings_state.read().await.clone();
    Ok(new_settings != current)
}

fn setup_config_watcher(
    tx: tokio::sync::mpsc::Sender<()>,
    feeds_path: &Option<PathBuf>,
    settings_path: &Option<PathBuf>,
) -> Option<RecommendedWatcher> {
    let watcher = RecommendedWatcher::new(
        move |_event: Result<notify::Event, notify::Error>| {
            let _ = tx.blocking_send(());
        },
        Config::default(),
    );

    let mut watcher = match watcher {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[config-watcher] failed to create watcher: {e}");
            return None;
        }
    };

    // Watch the parent directories (not the files directly) because editors
    // like vim delete and recreate files, which would remove the watch.
    let mut watched = std::collections::HashSet::new();
    for path in [feeds_path, settings_path]
        .iter()
        .filter_map(|p| p.as_ref())
    {
        if let Some(parent) = path.parent() {
            if parent.exists() && watched.insert(parent.to_path_buf()) {
                if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                    eprintln!("[config-watcher] failed to watch {}: {e}", parent.display());
                }
            }
        }
    }

    Some(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cortado-test-{name}-{ts}.toml"))
    }

    #[test]
    fn status_has_any_change_default_is_false() {
        let status = ConfigChangeStatus::default();
        assert!(!status.has_any_change());
    }

    #[test]
    fn status_has_any_change_feeds() {
        let status = ConfigChangeStatus {
            feeds_changed: true,
            ..Default::default()
        };
        assert!(status.has_any_change());
    }

    #[test]
    fn status_has_any_change_settings() {
        let status = ConfigChangeStatus {
            settings_changed: true,
            ..Default::default()
        };
        assert!(status.has_any_change());
    }

    #[test]
    fn status_has_any_change_parse_error() {
        let status = ConfigChangeStatus {
            parse_error: Some("bad".to_string()),
            ..Default::default()
        };
        assert!(status.has_any_change());
    }

    #[test]
    fn change_description_feeds_only() {
        let status = ConfigChangeStatus {
            feeds_changed: true,
            ..Default::default()
        };
        assert_eq!(status.change_description(), "Feed config changed");
    }

    #[test]
    fn change_description_settings_only() {
        let status = ConfigChangeStatus {
            settings_changed: true,
            ..Default::default()
        };
        assert_eq!(status.change_description(), "Settings changed");
    }

    #[test]
    fn change_description_both() {
        let status = ConfigChangeStatus {
            feeds_changed: true,
            settings_changed: true,
            ..Default::default()
        };
        assert_eq!(status.change_description(), "Config changed");
    }

    #[test]
    fn feeds_unchanged_when_content_matches() {
        let path = temp_path("feeds-match");
        let raw = r#"[[feed]]
name = "X"
type = "http-health"
url = "https://example.com"
"#;
        fs::write(&path, raw).unwrap();
        let startup = config::parse_feeds_config_str(raw).unwrap();

        let result = check_feeds_changed(&path, &startup);
        assert!(!result.unwrap());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn feeds_changed_when_content_differs() {
        let path = temp_path("feeds-differ");
        let startup_raw = r#"[[feed]]
name = "X"
type = "http-health"
url = "https://example.com"
"#;
        let new_raw = r#"[[feed]]
name = "X"
type = "http-health"
url = "https://changed.example.com"
"#;
        let startup = config::parse_feeds_config_str(startup_raw).unwrap();
        fs::write(&path, new_raw).unwrap();

        let result = check_feeds_changed(&path, &startup);
        assert!(result.unwrap());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn feeds_error_on_invalid_toml() {
        let path = temp_path("feeds-invalid");
        fs::write(&path, "this is not valid [[[ toml").unwrap();
        let startup = vec![];

        let result = check_feeds_changed(&path, &startup);
        assert!(result.is_err());

        let _ = fs::remove_file(&path);
    }

    #[tokio::test]
    async fn settings_unchanged_when_content_matches() {
        let path = temp_path("settings-match");
        let settings = AppSettings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        fs::write(&path, &toml_str).unwrap();

        let state = AppSettingsState::new(settings);
        let result = check_settings_changed(&path, &state).await;
        assert!(!result.unwrap());

        let _ = fs::remove_file(&path);
    }

    #[tokio::test]
    async fn settings_changed_when_content_differs() {
        let path = temp_path("settings-differ");
        let mut modified = AppSettings::default();
        modified.general.theme = "dark".to_string();
        let toml_str = toml::to_string_pretty(&modified).unwrap();
        fs::write(&path, &toml_str).unwrap();

        let state = AppSettingsState::new(AppSettings::default());
        let result = check_settings_changed(&path, &state).await;
        assert!(result.unwrap());

        let _ = fs::remove_file(&path);
    }

    #[tokio::test]
    async fn config_change_state_starts_empty() {
        let state = ConfigChangeState::new();
        let status = state.status().await;
        assert!(!status.has_any_change());
    }

    #[tokio::test]
    async fn config_change_state_update_and_read() {
        let state = ConfigChangeState::new();
        state
            .update(ConfigChangeStatus {
                feeds_changed: true,
                ..Default::default()
            })
            .await;
        let status = state.status().await;
        assert!(status.feeds_changed);
        assert!(!status.settings_changed);
    }
}
