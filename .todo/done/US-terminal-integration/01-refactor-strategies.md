---
status: done
---

# Refactor: terminal strategies module

## Goal

Restructure the existing Ghostty strategy and the waterfall into an extensible `terminals/` module. Adding a new terminal becomes: one file, one function, one line in the registry.

## Current state

- Ghostty strategy lives in `src-tauri/src/terminal_focus/ghostty.rs` (top-level)
- Waterfall in `mod.rs` lists strategies inline with mixed concerns (tmux, ghostty, stubs, app_activation)
- `escape_applescript()` is in `mod.rs` — will be shared by Terminal.app and iTerm2 strategies

## What to do

1. Create `src-tauri/src/terminal_focus/terminals/mod.rs` with a `try_focus(ctx) -> FocusResult` that iterates registered strategies
2. Move `ghostty.rs` into `terminals/ghostty.rs`
3. Replace the inline strategy entries in the waterfall with a single `terminals::try_focus` call
4. Move `escape_applescript()` somewhere shared (keep in parent `mod.rs` as `pub(crate)` — already done)

## Key constraint

The TTY resolution helper (`ps -p <pid> -o tty=`) will be shared by Terminal.app and iTerm2. Add it to `terminals/mod.rs` or a shared utils module during this refactor so both can use it.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminals/mod.rs` with strategy registry
- [ ] `src-tauri/src/terminal_focus/terminals/ghostty.rs` — moved from `ghostty.rs`
- [ ] Waterfall calls `terminals::try_focus(ctx)` as a single step (iterates internally)
- [ ] TTY resolution helper available for Terminal.app/iTerm2 strategies
- [ ] Adding a new terminal requires only: new file + add to registry
- [ ] Existing behavior unchanged — same tests pass
- [ ] `just check` passes
