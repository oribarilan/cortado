---
status: pending
---

# Sprint 07 — Notifications

## Theme

Add OS-level notifications to Cortado so users are alerted when activity statuses change. The notification system is configurable at two levels: global preferences (in `settings.toml`) and per-feed toggles (in `feeds.toml`).

## Key design decisions

- **Trigger**: Activity rollup kind change (the derived status dot shifts). New/removed activities are also notifiable (configurable).
- **Opt-out model**: Notifications are on by default per feed. Users disable with `notify = false`.
- **Notification mode**: All changes, escalation-only, or specific destination kinds.
- **Delivery presets**: Immediate (per-activity) or Grouped (per-feed per-poll, default). Digest mode deferred to backlog.
- **Settings reload**: Master toggle takes effect immediately; all other settings apply on next poll cycle.
- **Config persistence**: Global settings in `~/.config/cortado/settings.toml` (new); per-feed toggle in `feeds.toml`.
- **Channel**: macOS Notification Center via `tauri-plugin-notification` (replaces unused `system-notification` crate).
- **Click action**: Opens the activity's URL (PR link for GH/ADO feeds).
- **Startup**: Suppress notifications during initial seed to avoid flood.

## Task sequencing

Tasks are mostly sequential — each builds on prior work:

1. **settings.toml infrastructure** — foundation for global preferences (no deps)
2. **tauri-plugin-notification setup** — install plugin, verify permissions (no deps)
3. **notification config types** — Rust types for notification settings (depends on 1)
4. **per-feed notify toggle** — `notify` on FeedConfig (depends on 3 for full integration, but structurally independent)
5. **status change detection** — diff engine comparing snapshots (depends on 3 for config, but core logic is independent)
6. **notification dispatch** — wire detection → OS notifications (depends on 2, 3, 4, 5)
7. **notification content formatting** — title/body templates (depends on 6)
8. **settings UI — notifications tab** — full UI for notification preferences (depends on 3; can parallel with 5-7)
9. **spec update** — update specs/main.md to document notifications (depends on design being finalized)
10. **integration testing & edge cases** — comprehensive testing (depends on all above)

Tasks 1 and 2 can run in parallel. Task 4 is structurally independent from 3 but benefits from having types defined. Task 8 (UI) can be developed in parallel with 5-7 once types exist.

## Scope boundaries

**In scope:**
- `settings.toml` config system (global app preferences)
- `tauri-plugin-notification` integration
- Notification mode, delivery presets (Immediate + Grouped), new/removed activity toggles
- Per-feed `notify` toggle
- Status change detection (activity rollup diff)
- Notification content design (single, grouped, digest)
- Click-to-open-URL action
- Settings UI notifications tab
- Spec update (remove non-goal, add notifications section)

**Out of scope:**
- Tray icon rollup (stays in backlog)
- Digest delivery preset (deferred to backlog as `optional-notification-digest`)
- Notification scheduling / DND / quiet hours
- Notification history / log
- Sound customization (defer to macOS system settings)
- Cross-platform notification support (macOS only, consistent with Phase 1)
