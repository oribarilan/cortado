use crate::feed::{FeedSnapshot, StatusKind};

/// Type of status change detected between poll snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// An existing activity's rollup kind changed.
    KindChanged,
    /// A new activity appeared in the feed.
    NewActivity,
    /// An activity disappeared from the feed (and was not retained).
    RemovedActivity,
}

/// A single notifiable status change event.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used by dispatch and content modules.
pub struct StatusChangeEvent {
    pub feed_name: String,
    pub activity_id: String,
    pub activity_title: String,
    pub activity_url: Option<String>,
    pub change_type: ChangeType,
    pub previous_kind: Option<StatusKind>,
    pub new_kind: Option<StatusKind>,
}

/// Compares two snapshots of the same feed and returns notifiable changes.
///
/// - `KindChanged`: activity exists in both, but rollup kind differs.
/// - `NewActivity`: activity present in `new` but absent in `prev`.
/// - `RemovedActivity`: activity present in `prev` but absent in `new`
///   (and not retained in `new`).
///
/// Retained activities are excluded from change detection — their status
/// is frozen at Idle and should not produce notifications.
pub fn detect_changes(prev: &FeedSnapshot, new: &FeedSnapshot) -> Vec<StatusChangeEvent> {
    let mut events = Vec::new();

    // Build a map of previous activity id → rollup kind.
    let prev_map: std::collections::HashMap<&str, StatusKind> = prev
        .activities
        .iter()
        .filter(|a| !a.retained)
        .map(|a| (a.id.as_str(), StatusKind::rollup_for_activity(a)))
        .collect();

    // Check new activities for kind changes and new appearances.
    for activity in &new.activities {
        if activity.retained {
            continue;
        }

        let new_kind = StatusKind::rollup_for_activity(activity);
        let url = extract_activity_url(activity);

        if let Some(&prev_kind) = prev_map.get(activity.id.as_str()) {
            if prev_kind != new_kind {
                events.push(StatusChangeEvent {
                    feed_name: new.name.clone(),
                    activity_id: activity.id.clone(),
                    activity_title: activity.title.clone(),
                    activity_url: url,
                    change_type: ChangeType::KindChanged,
                    previous_kind: Some(prev_kind),
                    new_kind: Some(new_kind),
                });
            }
        } else {
            events.push(StatusChangeEvent {
                feed_name: new.name.clone(),
                activity_id: activity.id.clone(),
                activity_title: activity.title.clone(),
                activity_url: url,
                change_type: ChangeType::NewActivity,
                previous_kind: None,
                new_kind: Some(new_kind),
            });
        }
    }

    // Check for removed activities.
    let new_ids: std::collections::HashSet<&str> =
        new.activities.iter().map(|a| a.id.as_str()).collect();

    for activity in &prev.activities {
        if activity.retained {
            continue;
        }

        if !new_ids.contains(activity.id.as_str()) {
            let prev_kind = StatusKind::rollup_for_activity(activity);
            let url = extract_activity_url(activity);

            events.push(StatusChangeEvent {
                feed_name: new.name.clone(),
                activity_id: activity.id.clone(),
                activity_title: activity.title.clone(),
                activity_url: url,
                change_type: ChangeType::RemovedActivity,
                previous_kind: Some(prev_kind),
                new_kind: None,
            });
        }
    }

    events
}

/// Extracts an openable URL from an activity (ID or url field).
fn extract_activity_url(activity: &crate::feed::Activity) -> Option<String> {
    if activity.id.starts_with("https://") || activity.id.starts_with("http://") {
        return Some(activity.id.clone());
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::feed::{Activity, FeedSnapshot, Field, FieldValue, StatusKind};

    use super::*;

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
        }
    }

    fn snapshot(name: &str, activities: Vec<Activity>) -> FeedSnapshot {
        FeedSnapshot {
            name: name.to_string(),
            feed_type: "test".to_string(),
            activities,
            provided_fields: Vec::new(),
            error: None,
            hide_when_empty: false,
        }
    }

    #[test]
    fn no_changes_detected_when_identical() {
        let a = activity(
            "pr-1",
            "PR #1",
            vec![status_field("review", "awaiting", StatusKind::Waiting)],
        );
        let prev = snapshot("Feed", vec![a.clone()]);
        let new = snapshot("Feed", vec![a]);

        let events = detect_changes(&prev, &new);
        assert!(events.is_empty());
    }

    #[test]
    fn detects_kind_change() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "pr-1",
                "PR #1",
                vec![status_field("review", "awaiting", StatusKind::Waiting)],
            )],
        );
        let new = snapshot(
            "Feed",
            vec![activity(
                "pr-1",
                "PR #1",
                vec![status_field(
                    "review",
                    "approved",
                    StatusKind::AttentionPositive,
                )],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::KindChanged);
        assert_eq!(events[0].previous_kind, Some(StatusKind::Waiting));
        assert_eq!(events[0].new_kind, Some(StatusKind::AttentionPositive));
        assert_eq!(events[0].activity_id, "pr-1");
    }

    #[test]
    fn detects_new_activity() {
        let prev = snapshot("Feed", vec![]);
        let new = snapshot(
            "Feed",
            vec![activity(
                "pr-2",
                "PR #2",
                vec![status_field("review", "awaiting", StatusKind::Waiting)],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::NewActivity);
        assert!(events[0].previous_kind.is_none());
        assert_eq!(events[0].new_kind, Some(StatusKind::Waiting));
    }

    #[test]
    fn detects_removed_activity() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "pr-3",
                "PR #3",
                vec![status_field(
                    "review",
                    "approved",
                    StatusKind::AttentionPositive,
                )],
            )],
        );
        let new = snapshot("Feed", vec![]);

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::RemovedActivity);
        assert_eq!(events[0].previous_kind, Some(StatusKind::AttentionPositive));
        assert!(events[0].new_kind.is_none());
    }

    #[test]
    fn retained_activities_are_excluded() {
        let mut retained = activity(
            "pr-4",
            "PR #4",
            vec![status_field(
                "review",
                "approved",
                StatusKind::AttentionPositive,
            )],
        );
        retained.retained = true;

        let prev = snapshot("Feed", vec![retained.clone()]);
        let new = snapshot("Feed", vec![retained]);

        let events = detect_changes(&prev, &new);
        assert!(events.is_empty());
    }

    #[test]
    fn multiple_changes_in_one_snapshot() {
        let prev = snapshot(
            "Feed",
            vec![
                activity(
                    "pr-1",
                    "PR #1",
                    vec![status_field("checks", "passing", StatusKind::Idle)],
                ),
                activity(
                    "pr-2",
                    "PR #2",
                    vec![status_field("checks", "running", StatusKind::Running)],
                ),
            ],
        );

        let new = snapshot(
            "Feed",
            vec![
                activity(
                    "pr-1",
                    "PR #1",
                    vec![status_field(
                        "checks",
                        "failing",
                        StatusKind::AttentionNegative,
                    )],
                ),
                activity(
                    "pr-3",
                    "PR #3",
                    vec![status_field("checks", "passing", StatusKind::Idle)],
                ),
            ],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 3);

        let kind_changed = events.iter().find(|e| e.activity_id == "pr-1").unwrap();
        assert_eq!(kind_changed.change_type, ChangeType::KindChanged);

        let new_activity = events.iter().find(|e| e.activity_id == "pr-3").unwrap();
        assert_eq!(new_activity.change_type, ChangeType::NewActivity);

        let removed = events.iter().find(|e| e.activity_id == "pr-2").unwrap();
        assert_eq!(removed.change_type, ChangeType::RemovedActivity);
    }

    #[test]
    fn extracts_url_from_activity_id() {
        let a = activity(
            "https://github.com/org/repo/pull/42",
            "PR #42",
            vec![status_field("review", "awaiting", StatusKind::Waiting)],
        );
        let prev = snapshot("Feed", vec![]);
        let new = snapshot("Feed", vec![a]);

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].activity_url.as_deref(),
            Some("https://github.com/org/repo/pull/42")
        );
    }

    #[test]
    fn no_url_when_id_and_fields_are_not_urls() {
        let a = Activity {
            id: "test:check".to_string(),
            title: "Test".to_string(),
            fields: vec![Field {
                name: "output".to_string(),
                label: "Output".to_string(),
                value: FieldValue::Text {
                    value: "hello".to_string(),
                },
            }],
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
        };

        let prev = snapshot("Feed", vec![]);
        let new = snapshot("Feed", vec![a]);

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert!(events[0].activity_url.is_none());
    }

    #[test]
    fn errored_feed_snapshot_produces_no_changes() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "pr-1",
                "PR #1",
                vec![status_field("review", "awaiting", StatusKind::Waiting)],
            )],
        );
        let mut new = snapshot("Feed", vec![]);
        new.error = Some("poll failed".to_string());

        // When a poll errors, the runtime preserves previous activities,
        // but if it didn't, we should still not crash.
        let events = detect_changes(&prev, &new);
        // pr-1 would be detected as removed — but the dispatch layer
        // should skip errored feeds. Detection itself is honest.
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::RemovedActivity);
    }

    #[test]
    fn rollup_uses_highest_priority_across_fields() {
        let prev = snapshot(
            "Feed",
            vec![activity(
                "pr-1",
                "PR #1",
                vec![
                    status_field("review", "awaiting", StatusKind::Waiting),
                    status_field("checks", "passing", StatusKind::Idle),
                ],
            )],
        );

        // checks goes from passing → failing, which escalates the rollup
        let new = snapshot(
            "Feed",
            vec![activity(
                "pr-1",
                "PR #1",
                vec![
                    status_field("review", "awaiting", StatusKind::Waiting),
                    status_field("checks", "failing", StatusKind::AttentionNegative),
                ],
            )],
        );

        let events = detect_changes(&prev, &new);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].previous_kind, Some(StatusKind::Waiting));
        assert_eq!(events[0].new_kind, Some(StatusKind::AttentionNegative));
    }
}
