---
status: pending
---

# Reloadable poller runtime manager

## Goal

Introduce runtime control that can replace active feed polling state atomically when config is reloaded.

## Acceptance criteria

- [ ] A runtime manager/state type exists to own current feed registry, cache, and poller loop handles.
- [ ] Runtime supports an atomic "swap to new config" operation.
- [ ] Old poll loops are cancelled/stopped during swap to avoid duplicate polling.
- [ ] New runtime is seeded best-effort and then starts steady-state polling.
- [ ] Swap operation preserves read-path consistency (commands/tray see a coherent snapshot set).
- [ ] Config/build failure leaves prior runtime active (no partial teardown).
- [ ] `list_feeds` and tray refresh continue using cache (no synchronous poll-all).
- [ ] `just check` passes.

## Notes

- Keep this task backend/runtime-focused; tray reload wiring happens in task 06.
- Maintain existing feed-level error semantics during and after runtime swaps.

## Relevant files

- `src-tauri/src/main.rs`
- `src-tauri/src/feed/runtime.rs`
- `src-tauri/src/command.rs`
- `src-tauri/src/feed/mod.rs`
