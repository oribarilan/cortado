---
status: pending
---

# Tray reload applies full feed config

## Goal

Make tray `Reload` re-read `feeds.toml` and apply all feed config changes without app restart.

## Acceptance criteria

- [ ] Reload menu action remains asynchronous/non-blocking in tray handler.
- [ ] Reload re-parses `~/.config/cortado/feeds.toml` on demand.
- [ ] Successful reload applies full config delta:
  - add/remove feeds
  - changed feed names/types (treated as remove/add)
  - interval updates
  - type-specific config updates
  - field override updates
  - `notify` updates
- [ ] Reload success triggers tray refresh from updated cache/runtime.
- [ ] Reload config errors are surfaced clearly and keep last-known-good runtime active.
- [ ] Reload does not regress existing menu behavior (`Open`, `Quit`, feed error rows, empty states).
- [ ] `just check` passes.

## Notes

- This task wires UI event to runtime manager from task 05.
- File watcher/hot reload remains out of scope.

## Relevant files

- `src-tauri/src/tray.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/feed/config.rs`
- `src-tauri/src/feed/runtime.rs`
