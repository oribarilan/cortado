# Notifications show generic status kind instead of feed-specific status value

## Context

When a status change triggers a notification, the notification body shows the generic `StatusKind::human_name()` label (e.g., "in progress", "ready to go", "needs attention") instead of the feed-specific status value (e.g., "working", "approved", "failing").

For example, when an opencode harness session starts working, the panel shows "working" but the notification says "in progress". Similarly, a GitHub PR approval shows "approved" in the panel but "ready to go" in the notification.

**Value delivered**: Notifications match what the user sees in the panel/tray, making them immediately actionable without having to mentally translate generic labels.

## Related Files

- `src-tauri/src/notification/change_detection.rs` — `StatusChangeEvent` struct (lines 17-25) and `detect_changes()` — event only carries `StatusKind`, not the status value string
- `src-tauri/src/notification/content.rs` — `format_single()` (lines 12-33) — composes notification body using `StatusKind::human_name()` for `KindChanged` events
- `src-tauri/src/feed/mod.rs` — `StatusKind::human_name()` (lines 123-131) and `FieldValue::Status { value, kind }` (line 139)
- `src-tauri/src/notification/content.rs` (lines 73-247) — unit tests
- `src-tauri/src/notification/integration_tests.rs` — integration tests

## Dependencies

- None

## Acceptance Criteria

- [ ] `StatusChangeEvent` carries the new (and optionally previous) status value string alongside the status kind
- [ ] `detect_changes()` populates the status value from the activity's status field
- [ ] `format_single()` uses the status value in the notification body, falling back to `StatusKind::human_name()` only when no value is available
- [ ] All existing tests in `notification/content.rs` and `notification/integration_tests.rs` are updated and pass
- [ ] `just check` passes cleanly

## Verification

- **Automated**: Run `just check` — all tests pass, no warnings
- **Ad-hoc**: Trigger a status change for a harness feed (e.g., opencode starts working) and confirm the notification says "working", not "in progress"

## Notes

- The root cause is that `StatusChangeEvent` was designed to only track kind transitions, not the underlying status value. The fix is to thread the value through from `detect_changes()` to `format_single()`.
- `StatusKind::human_name()` should remain as the fallback for edge cases where a value is unavailable, but the feed-specific value should always be preferred.
