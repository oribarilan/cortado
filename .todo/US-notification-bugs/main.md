# US-notification-bugs

## Goal

Fix bugs and improve formatting in notification content so that notifications display accurate, feed-specific information with clean, readable formatting.

## Definition of Done

- [ ] Notifications show feed-specific status values (e.g., "working", "approved", "failing") instead of generic status kind labels (e.g., "in progress", "ready to go", "needs attention")
- [ ] Notification formatting uses arrow separator, updated new/removed labels, and grouped notifications show the latest change
- [ ] All unit tests in `notification/content.rs` and `notification/integration_tests.rs` updated and passing
- [ ] `just check` passes cleanly

## Task Priority

1. `01-status-value-in-notifications.md` -- fix the status value bug (data plumbing)
2. `02-notification-formatting.md` -- rework formatting (presentation)

Task 2 depends on task 1 (uses the status value that task 1 threads through).

## Cross-Cutting Concerns

- The generic `StatusKind::human_name()` labels should remain available as a fallback when no status value is present, but should never be preferred over the feed-specific value.
- This US is separate from `US-notification-model` which deals with notification modes and per-feed overrides, not notification content.
- Use plain Unicode symbols (e.g., `→`), not emoji.
