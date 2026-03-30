---
status: pending
---

# Refactor: terminal strategies module

## Goal

Restructure the existing Ghostty strategy and the waterfall into an extensible `terminals/` module. Adding a new terminal becomes: one file, one function, one line in the registry.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminals/mod.rs` with strategy registry
- [ ] `src-tauri/src/terminal_focus/terminals/ghostty.rs` — moved from `ghostty.rs`
- [ ] Waterfall calls `terminals::try_focus(ctx)` as a single step (iterates internally)
- [ ] Adding a new terminal requires only: new file + add to `TERMINAL_STRATEGIES`
- [ ] Existing behavior unchanged
- [ ] `just check` passes
