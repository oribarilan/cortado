---
status: done
---

# Wire `ado-pr` into feed registry

## Goal

Make `ado-pr` instantiable from config via the shared feed registry dispatch.

## Acceptance criteria

- [x] `feed/mod.rs` exposes `ado_pr` module.
- [x] Feed instantiation dispatch recognizes `type = "ado-pr"`.
- [x] Unknown feed type behavior remains unchanged.
- [x] Existing feed types (`github-pr`, `shell`) remain unaffected.
- [x] `just check` passes.

## Relevant files

- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/main.rs` (if wiring updates are needed)
