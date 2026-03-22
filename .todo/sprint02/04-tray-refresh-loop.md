---
status: pending
---

# Tray menu refresh from background poller

## Goal

Keep the native tray menu in sync with background-polled feed snapshots so users see updates without manual reloads.

## Acceptance criteria

- [ ] Tray menu refresh path consumes poller cache snapshots instead of triggering full synchronous `poll_all()`.
- [ ] Tray UI reflects new snapshots after background polls (automatic refresh loop or equivalent update trigger).
- [ ] Manual `Reload` action remains available and refreshes menu from current cache/state (no direct synchronous repoll in tray event handler).
- [ ] Reload does not regress error rendering, empty states, or feed submenu structure.
- [ ] Startup tray render uses seeded snapshot data when available.
- [ ] `just check` passes.

## Notes

- Keep this task strictly tray/backend integration; no panel UI work in sprint02.

## Relevant files

- `src-tauri/src/tray.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/command.rs`
