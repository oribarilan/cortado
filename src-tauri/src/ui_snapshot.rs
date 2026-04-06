use tauri::{AppHandle, Manager};

use crate::feed::config_watcher::ConfigChangeState;
use crate::feed::connectivity::ConnectivityManager;
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

    // Mark network feeds as disconnected when offline.
    if let Some(cm) = app_handle.try_state::<std::sync::Arc<ConnectivityManager>>() {
        if cm.is_offline() {
            mark_disconnected_feeds(&mut snapshots);
        }
    }

    Ok(snapshots)
}

/// Marks all network-type feed snapshots as disconnected.
///
/// Separated from `list_for_ui` so the logic is testable without an `AppHandle`.
fn mark_disconnected_feeds(snapshots: &mut [FeedSnapshot]) {
    for snapshot in snapshots {
        if crate::feed::is_network_feed_type(&snapshot.feed_type) {
            snapshot.is_disconnected = true;
        }
    }
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
        // Invalid config -- show as an activity with error details, no restart action.
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
                last_refreshed: None,
                is_disconnected: false,
            },
        );
    } else if status.feeds_changed || status.settings_changed {
        // Valid config that differs from running -- prompt restart.
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
                last_refreshed: None,
                is_disconnected: false,
            },
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::feed::FeedSnapshot;

    use super::mark_disconnected_feeds;

    fn snapshot(name: &str, feed_type: &str) -> FeedSnapshot {
        FeedSnapshot {
            name: name.to_string(),
            feed_type: feed_type.to_string(),
            activities: Vec::new(),
            provided_fields: Vec::new(),
            error: None,
            hide_when_empty: false,
            last_refreshed: None,
            is_disconnected: false,
        }
    }

    #[test]
    fn marks_network_feeds_as_disconnected() {
        let mut snapshots = vec![
            snapshot("My PRs", "github-pr"),
            snapshot("CI", "github-actions"),
            snapshot("Copilot", "copilot-session"),
            snapshot("Health", "http-health"),
        ];

        mark_disconnected_feeds(&mut snapshots);

        assert!(snapshots[0].is_disconnected); // github-pr
        assert!(snapshots[1].is_disconnected); // github-actions
        assert!(!snapshots[2].is_disconnected); // copilot-session (local)
        assert!(snapshots[3].is_disconnected); // http-health
    }

    #[test]
    fn empty_snapshots_is_no_op() {
        let mut snapshots: Vec<FeedSnapshot> = Vec::new();
        mark_disconnected_feeds(&mut snapshots);
        assert!(snapshots.is_empty());
    }

    #[test]
    fn only_local_feeds_remain_connected() {
        let mut snapshots = vec![
            snapshot("Copilot", "copilot-session"),
            snapshot("OpenCode", "opencode-session"),
        ];

        mark_disconnected_feeds(&mut snapshots);

        assert!(!snapshots[0].is_disconnected);
        assert!(!snapshots[1].is_disconnected);
    }
}
