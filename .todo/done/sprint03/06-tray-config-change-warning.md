---
status: done
---

# Tray warning for changed config (restart required)

## Goal

Surface a persistent tray-visible warning when `feeds.toml` changes at runtime, instructing users to restart Cortado to apply updates.

## Acceptance criteria

- [x] Tray/menu refresh path checks config-change state asynchronously.
- [x] When changed, tray shows a persistent warning item/feed-level error with restart guidance.
- [x] Warning remains visible until app restart (or tracker baseline reset in future work).
- [x] Existing menu behavior is preserved (`Refresh feeds`, `Open`, `Quit`, feed errors, empty states).
- [x] Polling continues normally with current runtime while warning is displayed.
- [x] `just check` passes.

## Notes

- This sprint intentionally does not hot-reload or re-parse config into active runtime.
- Warning UX is intentionally loud and persistent.

## Relevant files

- `src-tauri/src/tray.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/feed/config.rs`
