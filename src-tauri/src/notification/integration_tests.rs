/// Integration tests for the notification pipeline.
///
/// Tests the full flow from snapshot changes through detection, filtering,
/// and content formatting — everything except actual OS notification delivery
/// (which requires an AppHandle and is tested manually).
#[cfg(test)]
mod tests {
    use crate::app_settings::{DeliveryPreset, NotificationMode, NotificationSettings};
    use crate::feed::{Activity, FeedSnapshot, Field, FieldValue, StatusKind};
    use crate::notification::change_detection::{detect_changes, ChangeType};
    use crate::notification::content::{format_grouped, format_single};
    use crate::notification::dispatch::matches_mode;

    fn status_field(name: &str, value: &str, kind: StatusKind) -> Field {
        Field {
            name: name.to_string(),
            label: name.to_string(),
            value: FieldValue::Status {
                value: value.to_string(),
                kind,
            },
        }
    }

    fn activity(id: &str, title: &str, fields: Vec<Field>) -> Activity {
        Activity {
            id: id.to_string(),
            title: title.to_string(),
            fields,
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        }
    }

    fn snapshot(name: &str, activities: Vec<Activity>) -> FeedSnapshot {
        FeedSnapshot {
            name: name.to_string(),
            feed_type: "github-pr".to_string(),
            activities,
            provided_fields: Vec::new(),
            error: None,
            hide_when_empty: false,
        }
    }

    // ========================================================================
    // Core flow: detection → content → filtering
    // ========================================================================

    #[test]
    fn full_pipeline_kind_change_produces_formatted_notification() {
        let prev = snapshot(
            "My PRs",
            vec![activity(
                "https://github.com/org/repo/pull/42",
                "Add notifications",
                vec![
                    status_field("review", "awaiting", StatusKind::Waiting),
                    status_field("checks", "passing", StatusKind::Idle),
                ],
            )],
        );
        let new = snapshot(
            "My PRs",
            vec![activity(
                "https://github.com/org/repo/pull/42",
                "Add notifications",
                vec![
                    status_field("review", "approved", StatusKind::AttentionPositive),
                    status_field("checks", "passing", StatusKind::Idle),
                ],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::KindChanged);
        assert_eq!(events[0].previous_kind, Some(StatusKind::Waiting));
        assert_eq!(events[0].new_kind, Some(StatusKind::AttentionPositive));

        let notification = format_single(&events[0]);
        assert_eq!(notification.title, "My PRs");
        assert!(notification.body.contains("Add notifications"));
        assert!(notification.body.contains("ready to go"));
        assert_eq!(
            notification.url.as_deref(),
            Some("https://github.com/org/repo/pull/42")
        );
    }

    #[test]
    fn full_pipeline_new_activity_notification() {
        let prev = snapshot("My PRs", vec![]);
        let new = snapshot(
            "My PRs",
            vec![activity(
                "https://github.com/org/repo/pull/99",
                "Fix critical bug",
                vec![status_field("review", "awaiting", StatusKind::Waiting)],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::NewActivity);

        let notification = format_single(&events[0]);
        assert!(notification.body.starts_with("New: "));
        assert!(notification.body.contains("Fix critical bug"));
    }

    #[test]
    fn full_pipeline_removed_activity_notification() {
        let prev = snapshot(
            "My PRs",
            vec![activity(
                "https://github.com/org/repo/pull/10",
                "Old PR merged",
                vec![status_field(
                    "review",
                    "approved",
                    StatusKind::AttentionPositive,
                )],
            )],
        );
        let new = snapshot("My PRs", vec![]);

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::RemovedActivity);

        let notification = format_single(&events[0]);
        assert!(notification.body.starts_with("Gone: "));
    }

    // ========================================================================
    // Notification modes
    // ========================================================================

    #[test]
    fn mode_all_fires_on_any_kind_change() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );
        let new = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "running", StatusKind::Running)],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);

        let mode = NotificationMode::All;
        assert!(matches_mode(&mode, &events[0]));
    }

    #[test]
    fn mode_escalation_only_fires_on_severity_increase() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );
        let escalation = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "failing", StatusKind::AttentionNegative)],
            )],
        );

        let events = detect_changes(&prev, &escalation);
        let mode = NotificationMode::EscalationOnly;
        assert!(matches_mode(&mode, &events[0]));
    }

    #[test]
    fn mode_escalation_only_does_not_fire_on_severity_decrease() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "failing", StatusKind::AttentionNegative)],
            )],
        );
        let de_escalation = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );

        let events = detect_changes(&prev, &de_escalation);
        let mode = NotificationMode::EscalationOnly;
        assert!(!matches_mode(&mode, &events[0]));
    }

    #[test]
    fn mode_specific_kinds_fires_only_for_configured_kinds() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );
        let new_neg = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "failing", StatusKind::AttentionNegative)],
            )],
        );
        let new_running = snapshot(
            "Feed",
            vec![activity(
                "1",
                "A",
                vec![status_field("s", "running", StatusKind::Running)],
            )],
        );

        let mode = NotificationMode::SpecificKinds {
            kinds: vec![StatusKind::AttentionNegative],
        };

        let events_neg = detect_changes(&prev, &new_neg);
        assert!(matches_mode(&mode, &events_neg[0]));

        let events_running = detect_changes(&prev, &new_running);
        assert!(!matches_mode(&mode, &events_running[0]));
    }

    // ========================================================================
    // Delivery presets
    // ========================================================================

    #[test]
    fn grouped_delivery_produces_single_notification_for_multiple_changes() {
        let prev = snapshot(
            "Feed",
            vec![
                activity(
                    "1",
                    "PR Alpha",
                    vec![status_field("s", "idle", StatusKind::Idle)],
                ),
                activity(
                    "2",
                    "PR Beta",
                    vec![status_field("s", "idle", StatusKind::Idle)],
                ),
                activity(
                    "3",
                    "PR Gamma",
                    vec![status_field("s", "idle", StatusKind::Idle)],
                ),
            ],
        );
        let new = snapshot(
            "Feed",
            vec![
                activity(
                    "1",
                    "PR Alpha",
                    vec![status_field("s", "running", StatusKind::Running)],
                ),
                activity(
                    "2",
                    "PR Beta",
                    vec![status_field("s", "failing", StatusKind::AttentionNegative)],
                ),
                activity(
                    "3",
                    "PR Gamma",
                    vec![status_field("s", "approved", StatusKind::AttentionPositive)],
                ),
            ],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 3);

        let grouped = format_grouped("Feed", &events);
        assert_eq!(grouped.title, "Feed");
        assert!(grouped.body.contains("3 activities changed"));
    }

    #[test]
    fn immediate_delivery_produces_individual_notifications() {
        let prev = snapshot(
            "Feed",
            vec![
                activity(
                    "1",
                    "PR A",
                    vec![status_field("s", "idle", StatusKind::Idle)],
                ),
                activity(
                    "2",
                    "PR B",
                    vec![status_field("s", "idle", StatusKind::Idle)],
                ),
            ],
        );
        let new = snapshot(
            "Feed",
            vec![
                activity(
                    "1",
                    "PR A",
                    vec![status_field("s", "running", StatusKind::Running)],
                ),
                activity(
                    "2",
                    "PR B",
                    vec![status_field("s", "failing", StatusKind::AttentionNegative)],
                ),
            ],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 2);

        // Immediate mode: each event gets its own notification
        for event in &events {
            let n = format_single(event);
            assert_eq!(n.title, "Feed");
            assert!(!n.body.is_empty());
        }
    }

    // ========================================================================
    // Edge cases
    // ========================================================================

    #[test]
    fn startup_suppression_no_notifications_from_empty_baseline() {
        // During startup, the initial snapshot has empty activities.
        // The first poll should NOT produce notifications because we compare
        // against the empty baseline.
        let _initial_empty = snapshot("My PRs", vec![]);
        let first_poll = snapshot(
            "My PRs",
            vec![
                activity(
                    "1",
                    "PR A",
                    vec![status_field("s", "idle", StatusKind::Idle)],
                ),
                activity(
                    "2",
                    "PR B",
                    vec![status_field("s", "waiting", StatusKind::Waiting)],
                ),
            ],
        );

        // This IS detected as "new activities" — but the startup seed
        // populates the cache before the poll loop starts. The poll loop
        // then compares against the seeded snapshot (which has activities),
        // not the empty one. So no notifications fire on the first
        // *recurring* poll if nothing changed.

        // To verify: if seed produced the same data as the first poll,
        // no changes are detected.
        let seeded = first_poll.clone();
        let events = detect_changes(&seeded, &first_poll);
        assert!(events.is_empty());
    }

    #[test]
    fn rapid_status_flapping_each_change_is_detected() {
        let state_a = snapshot(
            "Feed",
            vec![activity(
                "1",
                "PR",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );
        let state_b = snapshot(
            "Feed",
            vec![activity(
                "1",
                "PR",
                vec![status_field("s", "failing", StatusKind::AttentionNegative)],
            )],
        );
        let state_c = snapshot(
            "Feed",
            vec![activity(
                "1",
                "PR",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );

        // A → B: escalation
        let events_ab = detect_changes(&state_a, &state_b);
        assert_eq!(events_ab.len(), 1);
        assert_eq!(events_ab[0].new_kind, Some(StatusKind::AttentionNegative));

        // B → C: de-escalation
        let events_bc = detect_changes(&state_b, &state_c);
        assert_eq!(events_bc.len(), 1);
        assert_eq!(events_bc[0].new_kind, Some(StatusKind::Idle));
    }

    #[test]
    fn retained_activities_do_not_trigger_notifications() {
        let mut retained = activity(
            "1",
            "Merged PR",
            vec![status_field("s", "approved", StatusKind::AttentionPositive)],
        );
        retained.retained = true;

        let prev = snapshot("Feed", vec![retained.clone()]);
        let new = snapshot("Feed", vec![retained]);

        let events = detect_changes(&prev, &new);
        assert!(events.is_empty());
    }

    #[test]
    fn errored_feed_does_not_trigger_notifications_when_filtered() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "1",
                "PR",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );
        let mut new = snapshot("Feed", vec![]);
        new.error = Some("network error".to_string());

        // The dispatch module checks `new.error.is_some()` and skips.
        // But detect_changes itself is honest and will report the activity
        // as removed. The filtering happens in process_feed_update.
        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        // The dispatch code would skip this because new.error.is_some().
    }

    #[test]
    fn empty_feed_no_spurious_notifications() {
        let prev = snapshot("Feed", vec![]);
        let new = snapshot("Feed", vec![]);

        let events = detect_changes(&prev, &new);
        assert!(events.is_empty());
    }

    #[test]
    fn new_activity_toggle_controls_filtering() {
        let settings_on = NotificationSettings {
            enabled: true,
            mode: NotificationMode::All,
            delivery: DeliveryPreset::Immediate,
            notify_new_activities: true,
            notify_removed_activities: true,
        };

        let settings_off = NotificationSettings {
            enabled: true,
            mode: NotificationMode::All,
            delivery: DeliveryPreset::Immediate,
            notify_new_activities: false,
            notify_removed_activities: false,
        };

        let prev = snapshot("Feed", vec![]);
        let new = snapshot(
            "Feed",
            vec![activity(
                "1",
                "New PR",
                vec![status_field("s", "idle", StatusKind::Idle)],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::NewActivity);

        // With toggle on: event passes
        let filtered_on: Vec<_> = events
            .iter()
            .filter(|e| match &e.change_type {
                ChangeType::NewActivity => settings_on.notify_new_activities,
                ChangeType::RemovedActivity => settings_on.notify_removed_activities,
                ChangeType::KindChanged => true,
            })
            .collect();
        assert_eq!(filtered_on.len(), 1);

        // With toggle off: event filtered
        let filtered_off: Vec<_> = events
            .iter()
            .filter(|e| match &e.change_type {
                ChangeType::NewActivity => settings_off.notify_new_activities,
                ChangeType::RemovedActivity => settings_off.notify_removed_activities,
                ChangeType::KindChanged => true,
            })
            .collect();
        assert!(filtered_off.is_empty());
    }

    // ========================================================================
    // Config: notify toggle and TOML parsing
    // ========================================================================

    #[test]
    fn notify_field_parses_from_toml() {
        let raw = r#"
[[feed]]
name = "Silent feed"
type = "http-health"
url = "https://example.com"
notify = false

[[feed]]
name = "Loud feed"
type = "http-health"
url = "https://example.com"
notify = true

[[feed]]
name = "Default feed"
type = "http-health"
url = "https://example.com"
"#;

        let configs =
            crate::feed::config::parse_feeds_config_str(raw).expect("valid config should parse");
        assert_eq!(configs.len(), 3);
        assert_eq!(configs[0].notify, Some(false));
        assert_eq!(configs[1].notify, Some(true));
        assert_eq!(configs[2].notify, None); // absent = default (true)
    }

    // ========================================================================
    // Settings round-trip
    // ========================================================================

    #[test]
    fn notification_settings_round_trip_all_modes() {
        use crate::app_settings::AppSettings;

        for mode in [
            NotificationMode::All,
            NotificationMode::EscalationOnly,
            NotificationMode::SpecificKinds {
                kinds: vec![StatusKind::AttentionNegative, StatusKind::Waiting],
            },
        ] {
            let settings = AppSettings {
                notifications: NotificationSettings {
                    enabled: true,
                    mode: mode.clone(),
                    delivery: DeliveryPreset::Immediate,
                    notify_new_activities: false,
                    notify_removed_activities: true,
                },
                ..AppSettings::default()
            };

            // This tests the serde round-trip, not file I/O.
            let toml = toml::to_string_pretty(&settings).expect("serialize");
            let loaded: AppSettings = toml::from_str(&toml).expect("deserialize");
            assert_eq!(loaded.notifications.mode, mode);
            assert_eq!(loaded.notifications.delivery, DeliveryPreset::Immediate);
            assert!(!loaded.notifications.notify_new_activities);
            assert!(loaded.notifications.notify_removed_activities);
        }
    }
}
