---
status: pending
---

# Sprint 03 — Per-feed notifications + reloadable config

## Theme

Add desktop notifications that are configurable per **Feed**, and make tray **Reload** re-read `~/.config/cortado/feeds.toml` so all feed config changes take effect without app restart.

## Sequencing

```
01-spec-contract ───────────────────────────────┐
                                                 ├──> 04-new-activity-notifier ─┐
02-notification-backend ────────────────┐       │                                │
                                         ├───────┘                                ├──> 07-notification-and-reload-tests
03-feed-notify-config ───────────────────┘                                        │
                                                                                   │
05-reloadable-poller-runtime ───────────────────────> 06-reload-applies-all-feed-config ─┘
```

- Task 01 comes first to align `specs/main.md` with sprint scope before implementation.
- Tasks 02 and 03 can run in parallel after 01.
- Task 04 depends on 02 + 03 (needs notification backend and per-feed notify config).
- Task 05 can run in parallel with 02/03/04 (runtime reload foundation).
- Task 06 depends on 05 (tray reload wiring uses reloadable runtime) and should integrate with 03/04 behavior.
- Task 07 lands last to verify end-to-end notification + reload semantics.

## Cross-task notes

- **Spec alignment first**: current spec says config loads once at startup and notifications are a non-goal; sprint03 must update this contract.
- **Reload semantics**: this sprint adds **manual** reload only (tray action). File-watcher/hot-reload remains backlog.
- **Reload applies all feed config**: add/remove feeds, interval, type-specific fields, field overrides, and notification settings.
- **Failure safety**: reload parse/build failures must keep last-known-good runtime active.
- **Notification scope (MVP)**: notify on newly discovered activities only (by `Activity.id`), not status-change notifications.
- **Notification defaults**: `notify` defaults to `false` when omitted.
- **Noise control**: startup seed and reload seed are notification-silent to prevent flood.
- **Performance**: no synchronous poll-all from tray handlers; keep refresh/reload asynchronous and lightweight.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-spec-contract.md` | Update spec contract for per-feed notifications and explicit manual config reload |
| 02 | `02-notification-backend.md` | Wire desktop notification backend + permissions/capabilities |
| 03 | `03-feed-notify-config.md` | Add/parse per-feed `notify` config and thread through feed runtime |
| 04 | `04-new-activity-notifier.md` | Emit notifications for newly discovered activities on notify-enabled feeds |
| 05 | `05-reloadable-poller-runtime.md` | Add runtime manager that can atomically reload feed config and swap poll loops |
| 06 | `06-reload-applies-all-feed-config.md` | Wire tray Reload to apply full `feeds.toml` changes without restart |
| 07 | `07-notification-and-reload-tests.md` | Add deterministic tests for notification and reload behavior |
