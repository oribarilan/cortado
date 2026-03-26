---
status: pending
---

# 06 — Notification dispatch

## Goal

Wire the status change detection engine (task 05) to actual OS notifications via `tauri-plugin-notification` (task 02). Apply all configuration filters: notification mode, delivery preset, per-feed toggle.

## Dispatch pipeline

```
Poll result
  → detect_changes() (task 05)
  → filter by per-feed notify toggle
  → filter by global enabled (checked immediately — master toggle is live)
  → filter by notification mode (All / EscalationOnly / SpecificKinds)
  → batch by delivery preset (Immediate / Grouped)
  → send via tauri-plugin-notification
  → register click action (open activity URL)
```

## Acceptance criteria

- [ ] Changes are filtered by `feed.notify` toggle (skip feeds with `notify = false`)
- [ ] Changes are filtered by global `enabled` toggle
- [ ] `NotificationMode::All` passes all changes
- [ ] `NotificationMode::EscalationOnly` only passes changes where new kind > previous kind in priority
- [ ] `NotificationMode::SpecificKinds` only passes changes where new kind is in the configured set
- [ ] `DeliveryPreset::Immediate` sends one notification per change event
- [ ] `DeliveryPreset::Grouped` batches changes per feed within a poll cycle
- [ ] Clicking a notification opens the activity's URL (using `Activity.id` for PR feeds)
- [ ] Notifications are suppressed during startup seed (`seed_startup_best_effort`)
- [ ] Notification permission denied is handled gracefully (no crash, log warning)
- [ ] `just check` passes

## Notes

- For `Grouped` mode (the default and simplest): one notification per feed per poll. This is the natural boundary since each feed polls independently.
- `Digest` mode is deferred to backlog — do not implement here.
- The master `enabled` toggle must be checked live (not cached at poll start) so toggling it off takes effect immediately.
- For click actions, `tauri-plugin-notification` supports action types. Check the plugin docs for how to register a URL-open action on macOS.
- Consider a `NotificationDispatcher` struct that holds the `AppHandle`, `NotificationSettings`, and manages the digest buffer.

## Relevant files

- Task 02 output — notification plugin setup
- Task 03 output — notification config types
- Task 05 output — `detect_changes()` function
- `src-tauri/src/feed/runtime.rs` — poll loop integration point
- `src-tauri/src/main.rs` — app handle access
