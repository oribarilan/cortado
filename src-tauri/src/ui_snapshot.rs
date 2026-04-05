use tauri::{AppHandle, Manager};

use crate::feed::config_watcher::ConfigChangeState;
use crate::feed::{Activity, FeedAction, FeedSnapshot, Field, FieldValue, StatusKind};

const CONFIG_FEED_NAME: &str = "Cortado Config";
const CONFIG_FEED_TYPE: &str = "app";

pub async fn list_for_ui(app_handle: &AppHandle) -> Result<Vec<FeedSnapshot>, String> {
    let cache = app_handle
        .try_state::<crate::feed::FeedSnapshotCache>()
        .ok_or_else(|| "feed snapshot cache state is missing".to_string())?
        .inner()
        .clone();

    let mut snapshots = cache.list().await;
    inject_config_snapshot(app_handle, &mut snapshots).await?;

    Ok(snapshots)
}

async fn inject_config_snapshot(
    app_handle: &AppHandle,
    snapshots: &mut Vec<FeedSnapshot>,
) -> Result<(), String> {
    // Remove any stale config snapshot from a previous cycle.
    snapshots.retain(|snapshot| {
        !(snapshot.name == CONFIG_FEED_NAME && snapshot.feed_type == CONFIG_FEED_TYPE)
    });

    let state = app_handle
        .try_state::<ConfigChangeState>()
        .ok_or_else(|| "config change state is missing".to_string())?
        .inner()
        .clone();

    let status = state.status().await;

    if let Some(ref parse_error) = status.parse_error {
        // Invalid config — show as an activity with error details, no restart action.
        let activity = Activity {
            id: "config-error".to_string(),
            title: "Config error".to_string(),
            fields: vec![
                Field {
                    name: "status".to_string(),
                    label: "Status".to_string(),
                    value: FieldValue::Status {
                        value: "Invalid config".to_string(),
                        kind: StatusKind::AttentionNegative,
                    },
                },
                Field {
                    name: "error".to_string(),
                    label: "Error".to_string(),
                    value: FieldValue::Text {
                        value: parse_error.clone(),
                    },
                },
            ],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        };

        snapshots.insert(
            0,
            FeedSnapshot {
                name: CONFIG_FEED_NAME.to_string(),
                feed_type: CONFIG_FEED_TYPE.to_string(),
                activities: vec![activity],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
            },
        );
    } else if status.feeds_changed || status.settings_changed {
        // Valid config that differs from running — prompt restart.
        let description = status.change_description();
        let activity = Activity {
            id: "config-change".to_string(),
            title: description.to_string(),
            fields: vec![Field {
                name: "status".to_string(),
                label: "Status".to_string(),
                value: FieldValue::Status {
                    value: "Restart to apply".to_string(),
                    kind: StatusKind::AttentionPositive,
                },
            }],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: Some(FeedAction::RestartApp),
        };

        snapshots.insert(
            0,
            FeedSnapshot {
                name: CONFIG_FEED_NAME.to_string(),
                feed_type: CONFIG_FEED_TYPE.to_string(),
                activities: vec![activity],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
            },
        );
    }

    Ok(())
}
