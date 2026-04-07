---
status: done
---

# Backend: per-feed notification mode override

## Goal

Extend per-feed `notify` from a bool to support mode overrides, and wire the dispatch pipeline to resolve the effective mode per feed.

## Acceptance criteria

- [ ] `FeedNotifyOverride` enum: `Off`, `Global`, `Mode(NotificationMode)` (or equivalent)
- [ ] Config parsing: `notify = false` → Off, `notify = true` / absent → Global, `notify = "worth_knowing"` (etc.) → Mode
- [ ] Sibling field `notify_kinds` added: when `notify = "specific_kinds"`, `notify_kinds = ["attention", "idle"]` etc. carries the kinds list
- [ ] `FeedConfigDto` updated from `Option<bool>` to support the new type (round-trips correctly between frontend and backend)
- [ ] `feed_notify_map` in `NotificationContext` changes from `HashMap<String, bool>` to `HashMap<String, FeedNotifyOverride>`
- [ ] `process_feed_update` resolves effective mode: Off → skip, Global → global_mode, Mode(m) → m
- [ ] `matches_mode()` receives the resolved effective mode
- [ ] Tests for config parsing: all three variants (bool true, bool false, string mode name)
- [ ] Tests for dispatch resolution: effective mode correctly applied
- [ ] `just check` passes

## Notes

- Per-feed notify override changes require an app restart (no hot-reload of `feed_notify_map`).

## Related files

- `src-tauri/src/feed/config.rs` -- `FeedConfig`, `notify` field, TOML parsing
- `src-tauri/src/settings_config.rs` -- `FeedConfigDto`
- `src-tauri/src/notification/dispatch.rs` -- `NotificationContext`, `process_feed_update`
- `src-tauri/src/notification/runtime.rs` -- `NotificationContext` struct, `feed_notify_map` storage
- `src-tauri/src/main.rs` (lines 54-57) -- `feed_notify_map` construction
