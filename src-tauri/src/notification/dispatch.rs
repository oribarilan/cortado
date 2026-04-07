use tauri::AppHandle;

use crate::app_settings::{AppSettingsState, DeliveryPreset, FeedNotifyOverride, NotificationMode};
use crate::feed::{FeedSnapshot, StatusKind};

use super::change_detection::{self, ChangeType, StatusChangeEvent};
use super::content;

/// Processes a feed poll result and dispatches OS notifications if warranted.
///
/// This is the main entry point called from the poll loop after a snapshot
/// is built but before it replaces the cached version.
///
/// The pipeline:
/// 1. Detect changes between previous and new snapshot.
/// 2. Resolve per-feed notification override (Off / Global / Mode).
/// 3. Filter by global `enabled` toggle (read live from shared state).
/// 4. Filter by effective notification mode.
/// 5. Batch by delivery preset.
/// 6. Send via tauri-plugin-notification.
pub async fn process_feed_update(
    app_handle: &AppHandle,
    settings_state: &AppSettingsState,
    prev: &FeedSnapshot,
    new: &FeedSnapshot,
    feed_override: &FeedNotifyOverride,
) {
    // Skip errored polls -- they don't represent real status changes.
    if new.error.is_some() {
        return;
    }

    // Per-feed override: Off means skip entirely.
    if matches!(feed_override, FeedNotifyOverride::Off) {
        return;
    }

    // Read settings live (master toggle is immediate).
    let settings = settings_state.read().await;

    if !settings.notifications.enabled {
        return;
    }

    // Resolve effective mode: per-feed override or global.
    let effective_mode = match feed_override {
        FeedNotifyOverride::Off => unreachable!(),
        FeedNotifyOverride::Global => &settings.notifications.mode,
        FeedNotifyOverride::Mode(m) => m,
    };

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
            ChangeType::KindChanged => matches_mode(effective_mode, event),
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
pub(crate) fn matches_mode(mode: &NotificationMode, event: &StatusChangeEvent) -> bool {
    match mode {
        NotificationMode::WorthKnowing => event.new_kind.is_some_and(|k| {
            matches!(
                k,
                StatusKind::Idle | StatusKind::AttentionPositive | StatusKind::AttentionNegative
            )
        }),
        NotificationMode::NeedAttention => event.new_kind.is_some_and(|k| {
            matches!(
                k,
                StatusKind::AttentionPositive | StatusKind::AttentionNegative
            )
        }),
        NotificationMode::All => true,
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
    fn mode_worth_knowing_passes_idle() {
        let event = kind_changed_event(StatusKind::Running, StatusKind::Idle);
        assert!(matches_mode(&NotificationMode::WorthKnowing, &event));
    }

    #[test]
    fn mode_worth_knowing_passes_attention_positive() {
        let event = kind_changed_event(StatusKind::Waiting, StatusKind::AttentionPositive);
        assert!(matches_mode(&NotificationMode::WorthKnowing, &event));
    }

    #[test]
    fn mode_worth_knowing_passes_attention_negative() {
        let event = kind_changed_event(StatusKind::Idle, StatusKind::AttentionNegative);
        assert!(matches_mode(&NotificationMode::WorthKnowing, &event));
    }

    #[test]
    fn mode_worth_knowing_skips_running() {
        let event = kind_changed_event(StatusKind::Idle, StatusKind::Running);
        assert!(!matches_mode(&NotificationMode::WorthKnowing, &event));
    }

    #[test]
    fn mode_worth_knowing_skips_waiting() {
        let event = kind_changed_event(StatusKind::Idle, StatusKind::Waiting);
        assert!(!matches_mode(&NotificationMode::WorthKnowing, &event));
    }

    #[test]
    fn mode_worth_knowing_returns_false_when_new_kind_is_none() {
        let event = StatusChangeEvent {
            feed_name: "Feed".to_string(),
            activity_id: "pr-1".to_string(),
            activity_title: "PR #1".to_string(),
            activity_url: None,
            change_type: ChangeType::KindChanged,
            previous_kind: Some(StatusKind::Idle),
            new_kind: None,
        };
        assert!(!matches_mode(&NotificationMode::WorthKnowing, &event));
    }

    #[test]
    fn mode_need_attention_passes_attention_kinds() {
        let pos = kind_changed_event(StatusKind::Waiting, StatusKind::AttentionPositive);
        assert!(matches_mode(&NotificationMode::NeedAttention, &pos));

        let neg = kind_changed_event(StatusKind::Idle, StatusKind::AttentionNegative);
        assert!(matches_mode(&NotificationMode::NeedAttention, &neg));
    }

    #[test]
    fn mode_need_attention_skips_non_attention_kinds() {
        let idle = kind_changed_event(StatusKind::Running, StatusKind::Idle);
        assert!(!matches_mode(&NotificationMode::NeedAttention, &idle));

        let running = kind_changed_event(StatusKind::Idle, StatusKind::Running);
        assert!(!matches_mode(&NotificationMode::NeedAttention, &running));

        let waiting = kind_changed_event(StatusKind::Idle, StatusKind::Waiting);
        assert!(!matches_mode(&NotificationMode::NeedAttention, &waiting));
    }

    #[test]
    fn mode_need_attention_returns_false_when_new_kind_is_none() {
        let event = StatusChangeEvent {
            feed_name: "Feed".to_string(),
            activity_id: "pr-1".to_string(),
            activity_title: "PR #1".to_string(),
            activity_url: None,
            change_type: ChangeType::KindChanged,
            previous_kind: Some(StatusKind::Idle),
            new_kind: None,
        };
        assert!(!matches_mode(&NotificationMode::NeedAttention, &event));
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
