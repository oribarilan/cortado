use super::change_detection::StatusChangeEvent;

/// Formatted notification ready for OS delivery.
#[allow(dead_code)] // `url` reserved for click-to-open action.
pub struct FormattedNotification {
    pub title: String,
    pub body: String,
    pub url: Option<String>,
}

/// Formats a single status change event into a notification.
pub fn format_single(event: &StatusChangeEvent) -> FormattedNotification {
    use super::change_detection::ChangeType;
    use crate::feed::StatusKind;

    let body = match &event.change_type {
        ChangeType::NewActivity => format!("New: {}", truncate(&event.activity_title, 80)),
        ChangeType::RemovedActivity => format!("Gone: {}", truncate(&event.activity_title, 80)),
        ChangeType::KindChanged => {
            let kind_name = event
                .new_kind
                .map(StatusKind::human_name)
                .unwrap_or("unknown");
            format!("{} — {}", truncate(&event.activity_title, 60), kind_name)
        }
    };

    FormattedNotification {
        title: event.feed_name.clone(),
        body,
        url: event.activity_url.clone(),
    }
}

/// Formats multiple change events from one feed into a single grouped notification.
pub fn format_grouped(feed_name: &str, events: &[StatusChangeEvent]) -> FormattedNotification {
    if events.len() == 1 {
        return format_single(&events[0]);
    }

    let preview: Vec<&str> = events
        .iter()
        .take(3)
        .map(|e| e.activity_title.as_str())
        .collect();

    let body = format!(
        "{} activities changed\n{}",
        events.len(),
        preview.join(", ")
    );

    FormattedNotification {
        title: feed_name.to_string(),
        body: truncate(&body, 150).to_string(),
        url: None,
    }
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }

    // Find last char boundary before max_len - 1 to leave room for "…"
    let mut end = max_len.saturating_sub(1);
    while !s.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::super::change_detection::{ChangeType, StatusChangeEvent};
    use super::*;
    use crate::feed::StatusKind;

    fn event(
        feed: &str,
        id: &str,
        title: &str,
        change_type: ChangeType,
        prev: Option<StatusKind>,
        new: Option<StatusKind>,
    ) -> StatusChangeEvent {
        StatusChangeEvent {
            feed_name: feed.to_string(),
            activity_id: id.to_string(),
            activity_title: title.to_string(),
            activity_url: Some(format!("https://example.com/{id}")),
            change_type,
            previous_kind: prev,
            new_kind: new,
        }
    }

    #[test]
    fn format_single_kind_changed() {
        let e = event(
            "My PRs",
            "pr-1",
            "Add notifications",
            ChangeType::KindChanged,
            Some(StatusKind::Waiting),
            Some(StatusKind::AttentionNegative),
        );
        let n = format_single(&e);
        assert_eq!(n.title, "My PRs");
        assert!(n.body.contains("Add notifications"));
        assert!(n.body.contains("needs attention"));
    }

    #[test]
    fn format_single_new_activity() {
        let e = event(
            "My PRs",
            "pr-2",
            "Fix bug",
            ChangeType::NewActivity,
            None,
            Some(StatusKind::Idle),
        );
        let n = format_single(&e);
        assert!(n.body.starts_with("New: "));
        assert!(n.body.contains("Fix bug"));
    }

    #[test]
    fn format_single_removed_activity() {
        let e = event(
            "My PRs",
            "pr-3",
            "Old PR",
            ChangeType::RemovedActivity,
            Some(StatusKind::Idle),
            None,
        );
        let n = format_single(&e);
        assert!(n.body.starts_with("Gone: "));
    }

    #[test]
    fn format_grouped_single_delegates_to_format_single() {
        let e = event(
            "My PRs",
            "pr-1",
            "Solo change",
            ChangeType::KindChanged,
            Some(StatusKind::Idle),
            Some(StatusKind::Running),
        );
        let n = format_grouped("My PRs", &[e]);
        // Single event should still produce a useful notification
        assert_eq!(n.title, "My PRs");
        assert!(n.body.contains("Solo change"));
    }

    #[test]
    fn format_grouped_multiple_shows_count_and_preview() {
        let events = vec![
            event(
                "Feed",
                "1",
                "PR Alpha",
                ChangeType::KindChanged,
                Some(StatusKind::Idle),
                Some(StatusKind::Running),
            ),
            event(
                "Feed",
                "2",
                "PR Beta",
                ChangeType::NewActivity,
                None,
                Some(StatusKind::Waiting),
            ),
            event(
                "Feed",
                "3",
                "PR Gamma",
                ChangeType::RemovedActivity,
                Some(StatusKind::Idle),
                None,
            ),
        ];
        let n = format_grouped("Feed", &events);
        assert_eq!(n.title, "Feed");
        assert!(n.body.contains("3 activities changed"));
        assert!(n.body.contains("PR Alpha"));
    }

    #[test]
    fn truncation_handles_long_titles() {
        let long_title = "A".repeat(200);
        let e = event(
            "Feed",
            "pr-1",
            &long_title,
            ChangeType::NewActivity,
            None,
            Some(StatusKind::Idle),
        );
        let n = format_single(&e);
        assert!(n.body.len() < 200);
        assert!(n.body.starts_with("New: "));
    }

    #[test]
    fn truncation_preserves_valid_utf8_at_boundary() {
        // Multi-byte character: each is 3 bytes in UTF-8
        let multibyte_title = "\u{2603}".repeat(50); // snowman × 50 = 150 bytes
        let e = event(
            "Feed",
            "pr-1",
            &multibyte_title,
            ChangeType::KindChanged,
            Some(StatusKind::Idle),
            Some(StatusKind::Running),
        );
        let n = format_single(&e);
        // Should not panic and should produce valid UTF-8
        assert!(n.body.is_char_boundary(n.body.len()));
    }

    #[test]
    fn format_grouped_empty_events_still_works() {
        // Edge case: empty events slice
        let n = format_grouped("Feed", &[]);
        assert_eq!(n.title, "Feed");
        assert!(n.body.contains("0 activities changed"));
    }

    #[test]
    fn format_single_url_is_preserved() {
        let e = event(
            "Feed",
            "pr-1",
            "Test PR",
            ChangeType::KindChanged,
            Some(StatusKind::Idle),
            Some(StatusKind::Running),
        );
        let n = format_single(&e);
        assert_eq!(n.url.as_deref(), Some("https://example.com/pr-1"));
    }
}
