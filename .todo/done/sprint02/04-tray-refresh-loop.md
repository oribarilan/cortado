---
status: done
---

# Tray menu refresh from background poller

## Goal

Keep the native tray menu in sync with background-polled feed snapshots so users see updates without manual refresh actions.

## Acceptance criteria

- [x] Tray menu refresh path consumes poller cache snapshots instead of triggering full synchronous `poll_all()`.
- [x] Tray UI reflects new snapshots after background polls (automatic refresh loop or equivalent update trigger).
- [x] Manual `Refresh feeds` action remains available and triggers an immediate async one-shot repoll before refreshing menu state.
- [x] Manual refresh does not regress error rendering, empty states, or feed submenu structure.
- [x] Startup tray render uses seeded snapshot data when available.
- [x] `just check` passes.

## Notes

- Keep this task strictly tray/backend integration; no panel UI work in sprint02.

## Relevant files

- `src-tauri/src/tray.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/command.rs`
