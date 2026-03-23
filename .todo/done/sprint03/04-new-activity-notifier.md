---
status: deferred
---

# Notify on newly discovered activities

## Goal

Emit desktop notifications for newly discovered activities on feeds that have `notify = true`.

## Acceptance criteria

- [ ] Notification trigger is based on activity identity diff (`Activity.id`) between baseline and latest snapshot for a feed.
- [ ] Only newly discovered activities trigger notifications in sprint03 (no status-change alerts).
- [ ] Feeds with `notify = false` never dispatch notifications.
- [ ] Initial startup seed does not emit notifications.
- [ ] Reload seed/rehydration path does not emit notification floods.
- [ ] Poll errors do not emit notifications and do not break the poll loop.
- [ ] Notification message includes enough context to identify feed + activity.
- [ ] `just check` passes.

## Notes

- Keep diffing O(n) per feed poll; avoid expensive cross-feed scans.
- Dedupe behavior should be deterministic across repeated unchanged polls.

## Relevant files

- `src-tauri/src/feed/runtime.rs`
- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/feed/` (notification helper from task 02)
