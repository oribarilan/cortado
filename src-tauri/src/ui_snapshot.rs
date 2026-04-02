use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::feed::{config::ConfigChangeTracker, FeedSnapshot, FeedSnapshotCache};

const CONFIG_WARNING_FEED_NAME: &str = "Configuration";
const CONFIG_WARNING_FEED_TYPE: &str = "app";
const CONFIG_WARNING_MESSAGE: &str = "Config file changed. Restart Cortado to apply updates.";

pub async fn list_for_ui(app_handle: &AppHandle) -> Result<Vec<FeedSnapshot>, String> {
    let cache = app_handle
        .try_state::<FeedSnapshotCache>()
        .ok_or_else(|| "feed snapshot cache state is missing".to_string())?
        .inner()
        .clone();

    let mut snapshots = cache.list().await;
    inject_config_warning_snapshot(app_handle, &mut snapshots).await?;

    Ok(snapshots)
}

pub async fn refresh_config_change_state(app_handle: &AppHandle) -> Result<bool, String> {
    let tracker = app_handle
        .try_state::<Arc<ConfigChangeTracker>>()
        .ok_or_else(|| "config change tracker state is missing".to_string())?
        .inner()
        .clone();

    tracker
        .refresh()
        .await
        .map_err(|err| format!("config change refresh failed: {err}"))
}

async fn inject_config_warning_snapshot(
    app_handle: &AppHandle,
    snapshots: &mut Vec<FeedSnapshot>,
) -> Result<(), String> {
    snapshots.retain(|snapshot| {
        !(snapshot.name == CONFIG_WARNING_FEED_NAME
            && snapshot.feed_type == CONFIG_WARNING_FEED_TYPE)
    });

    let tracker = app_handle
        .try_state::<Arc<ConfigChangeTracker>>()
        .ok_or_else(|| "config change tracker state is missing".to_string())?
        .inner()
        .clone();

    let has_changed = tracker.has_changed().await;

    if has_changed {
        snapshots.insert(
            0,
            FeedSnapshot {
                name: CONFIG_WARNING_FEED_NAME.to_string(),
                feed_type: CONFIG_WARNING_FEED_TYPE.to_string(),
                activities: Vec::new(),
                provided_fields: Vec::new(),
                error: Some(CONFIG_WARNING_MESSAGE.to_string()),
                hide_when_empty: false,
            },
        );
    }

    Ok(())
}
