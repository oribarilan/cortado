---
status: done
---

# Config change tracker (restart-required model)

## Goal

Detect runtime changes to `~/.config/cortado/feeds.toml` without applying them live so Cortado can prompt for restart.

## Acceptance criteria

- [x] A small tracker type fingerprints the config file state at startup.
- [x] Tracker can report whether config file changed since startup.
- [x] Missing-file ↔ present-file transitions are detected as changes.
- [x] Detection uses lightweight metadata checks (mtime/size), not full file parsing on every check.
- [x] Detection failures are surfaced to logs but do not crash polling/runtime.
- [x] `just check` passes.

## Notes

- Runtime config apply behavior remains startup-only in this sprint.
- This task intentionally avoids runtime feed-registry swaps.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/src/main.rs`
