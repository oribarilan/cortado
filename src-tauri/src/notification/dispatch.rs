use tauri::AppHandle;

use crate::app_settings::{AppSettingsState, DeliveryPreset, NotificationMode};
use crate::feed::FeedSnapshot;

use super::change_detection::{self, ChangeType, StatusChangeEvent};
use super::content;

/// Processes a feed poll result and dispatches OS notifications if warranted.
///
/// This is the main entry point called from the poll loop after a snapshot
/// is built but before it replaces the cached version.
///
/// The pipeline:
/// 1. Detect changes between previous and new snapshot.
/// 2. Filter by per-feed `notify` toggle.
/// 3. Filter by global `enabled` toggle (read live from shared state).
/// 4. Filter by notification mode.
/// 5. Batch by delivery preset.
/// 6. Send via tauri-plugin-notification.
pub async fn process_feed_update(
    app_handle: &AppHandle,
    settings_state: &AppSettingsState,
    prev: &FeedSnapshot,
    new: &FeedSnapshot,
    feed_notify_enabled: bool,
) {
    // Skip errored polls — they don't represent real status changes.
    if new.error.is_some() {
        return;
    }

    // Per-feed toggle: skip feeds with notify=false.
    if !feed_notify_enabled {
        return;
    }

    // Read settings live (master toggle is immediate).
    let settings = settings_state.read().await;

    if !settings.notifications.enabled {
        return;
    }

    let all_changes = change_detection::detect_changes(prev, new);
    if all_changes.is_empty() {
        return;
    }

    // Filter by mode and activity event toggles.
    let filtered: Vec<StatusChangeEvent> = all_changes
        .into_iter()
        .filter(|event| match &event.change_type {
            ChangeType::NewActivity => settings.notifications.notify_new_activities,
            ChangeType::RemovedActivity => settings.notifications.notify_removed_activities,
            ChangeType::KindChanged => matches_mode(&settings.notifications.mode, event),
        })
        .collect();

    if filtered.is_empty() {
        return;
    }

    // Drop the settings lock before sending notifications.
    let delivery = settings.notifications.delivery;
    drop(settings);

    match delivery {
        DeliveryPreset::Immediate => {
            for event in &filtered {
                let formatted = content::format_single(event);
                send_notification(app_handle, &formatted);
            }
        }
        DeliveryPreset::Grouped => {
            let formatted = content::format_grouped(&new.name, &filtered);
            send_notification(app_handle, &formatted);
        }
    }
}

/// Checks whether a kind-change event passes the notification mode filter.
fn matches_mode(mode: &NotificationMode, event: &StatusChangeEvent) -> bool {
    match mode {
        NotificationMode::All => true,
        NotificationMode::EscalationOnly => match (event.previous_kind, event.new_kind) {
            (Some(prev), Some(new)) => new.priority() > prev.priority(),
            _ => false,
        },
        NotificationMode::SpecificKinds { kinds } => {
            event.new_kind.is_some_and(|k| kinds.contains(&k))
        }
    }
}

fn send_notification(app_handle: &AppHandle, notification: &content::FormattedNotification) {
    use tauri_plugin_notification::NotificationExt;

    let result = app_handle
        .notification()
        .builder()
        .title(&notification.title)
        .body(&notification.body)
        .show();

    if let Err(err) = result {
        eprintln!("failed to send notification: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_settings::NotificationSettings;
    use crate::feed::StatusKind;

    fn kind_changed_event(prev: StatusKind, new: StatusKind) -> StatusChangeEvent {
        StatusChangeEvent {
            feed_name: "Feed".to_string(),
            activity_id: "pr-1".to_string(),
            activity_title: "PR #1".to_string(),
            activity_url: None,
            change_type: ChangeType::KindChanged,
            previous_kind: Some(prev),
            new_kind: Some(new),
        }
    }

    #[test]
    fn mode_all_passes_everything() {
        let event = kind_changed_event(StatusKind::Idle, StatusKind::Running);
        assert!(matches_mode(&NotificationMode::All, &event));
    }

    #[test]
    fn mode_escalation_only_passes_higher_priority() {
        let escalation = kind_changed_event(StatusKind::Idle, StatusKind::AttentionNegative);
        assert!(matches_mode(&NotificationMode::EscalationOnly, &escalation));

        let de_escalation = kind_changed_event(StatusKind::AttentionNegative, StatusKind::Idle);
        assert!(!matches_mode(
            &NotificationMode::EscalationOnly,
            &de_escalation
        ));
    }

    #[test]
    fn mode_escalation_same_priority_does_not_pass() {
        let same = kind_changed_event(StatusKind::Waiting, StatusKind::Waiting);
        assert!(!matches_mode(&NotificationMode::EscalationOnly, &same));
    }

    #[test]
    fn mode_specific_kinds_filters_by_new_kind() {
        let mode = NotificationMode::SpecificKinds {
            kinds: vec![StatusKind::AttentionNegative],
        };

        let matching = kind_changed_event(StatusKind::Idle, StatusKind::AttentionNegative);
        assert!(matches_mode(&mode, &matching));

        let not_matching = kind_changed_event(StatusKind::Idle, StatusKind::Running);
        assert!(!matches_mode(&mode, &not_matching));
    }

    #[test]
    fn change_detection_filters_new_and_removed_by_toggle() {
        let settings = NotificationSettings {
            enabled: true,
            mode: NotificationMode::All,
            delivery: DeliveryPreset::Immediate,
            notify_new_activities: false,
            notify_removed_activities: false,
        };

        let new_event = StatusChangeEvent {
            feed_name: "Feed".to_string(),
            activity_id: "pr-1".to_string(),
            activity_title: "New PR".to_string(),
            activity_url: None,
            change_type: ChangeType::NewActivity,
            previous_kind: None,
            new_kind: Some(StatusKind::Idle),
        };

        let removed_event = StatusChangeEvent {
            feed_name: "Feed".to_string(),
            activity_id: "pr-2".to_string(),
            activity_title: "Old PR".to_string(),
            activity_url: None,
            change_type: ChangeType::RemovedActivity,
            previous_kind: Some(StatusKind::Idle),
            new_kind: None,
        };

        // Simulate the filtering logic from process_feed_update
        let events = vec![new_event, removed_event];
        let filtered: Vec<_> = events
            .into_iter()
            .filter(|event| match &event.change_type {
                ChangeType::NewActivity => settings.notify_new_activities,
                ChangeType::RemovedActivity => settings.notify_removed_activities,
                ChangeType::KindChanged => matches_mode(&settings.mode, event),
            })
            .collect();

        assert!(filtered.is_empty());
    }
}
