---
status: done
---

# Sprint 03 — Per-feed notifications + config change notice

## Theme

Add desktop notifications that are configurable per **Feed**, and detect runtime config changes so the tray can prompt for restart to apply updated `~/.config/cortado/feeds.toml`.

## Sequencing

```
01-spec-contract ───────────────────────────────┐
                                                 ├──> 04-new-activity-notifier ─┐
02-notification-backend ────────────────┐       │                                │
                                         ├───────┘                                ├──> 07-notification-and-reload-tests
03-feed-notify-config ───────────────────┘                                        │
                                                                                   │
05-config-change-tracker ───────────────────────────> 06-tray-config-change-warning ───────┘
```

- Task 01 comes first to align `specs/main.md` with sprint scope before implementation.
- Tasks 02 and 03 can run in parallel after 01.
- Task 04 depends on 02 + 03 (needs notification backend and per-feed notify config).
- Task 05 is completed and moved to `.todo/done/sprint03/`.
- Task 06 is completed and moved to `.todo/done/sprint03/`.
- Task 07 lands last to verify end-to-end notification + config-change warning semantics.

## Cross-task notes

- **Spec alignment first**: current spec says config loads once at startup and notifications are a non-goal; sprint03 must update this contract.
- **Config apply model**: config still applies on startup only. Runtime changes are detected and surfaced as restart-required warnings.
- **No runtime swap**: this sprint does not hot-reload feed runtime or re-read config into active polling loops.
- **Failure safety**: config-change detection failures should be logged; app continues polling with current runtime.
- **Notification scope (MVP)**: notify on newly discovered activities only (by `Activity.id`), not status-change notifications.
- **Notification defaults**: `notify` defaults to `false` when omitted.
- **Noise control**: startup seed is notification-silent to prevent flood.
- **Performance**: no synchronous poll-all from tray handlers; keep refresh/config-check paths asynchronous and lightweight.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `.todo/done/sprint03/01-spec-contract.md` | Deferred — carry over notification-specific spec items |
| 02 | `.todo/done/sprint03/02-notification-backend.md` | Deferred — notification backend wiring |
| 03 | `.todo/done/sprint03/03-feed-notify-config.md` | Deferred — per-feed `notify` config parsing |
| 04 | `.todo/done/sprint03/04-new-activity-notifier.md` | Deferred — new-activity notification dispatch |
| 05 | `.todo/done/sprint03/05-config-change-tracker.md` | ✅ Done — Track `feeds.toml` changes during runtime without applying them live |
| 06 | `.todo/done/sprint03/06-tray-config-change-warning.md` | ✅ Done — Surface persistent tray warning when config changes and restart is required |
| 07 | `.todo/done/sprint03/07-notification-and-reload-tests.md` | Deferred — notification-focused tests and remaining coverage |

## Outcome

- Completed in this sprint:
  - 05 `config-change-tracker`
  - 06 `tray-config-change-warning`
- Deferred to a future sprint:
  - 01 `spec-contract` (notification-specific spec items)
  - 02 `notification-backend`
  - 03 `feed-notify-config`
  - 04 `new-activity-notifier`
  - 07 `notification-and-reload-tests` (notification-focused checks)
