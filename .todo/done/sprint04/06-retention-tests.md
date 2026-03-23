---
status: done
---

# Retention + duration parser test coverage

## Goal

Add deterministic tests for sprint04 duration parsing and retained-activity lifecycle behavior.

## Acceptance criteria

- [x] Config parser tests cover valid duration strings for `interval` and `retain`.
- [x] Config parser tests reject integer interval values and invalid duration strings.
- [x] Runtime tests cover disappear → retained → expired lifecycle.
- [x] Runtime tests cover no-retain behavior and reappearance handling.
- [x] Tray behavior tests (or equivalent unit coverage) validate retained ordering and hollow-dot symbol selection.
- [x] Tests remain isolated (no external CLI/network dependency).
- [x] `just check` passes.

## Notes

- Prefer focused unit tests around parser/runtime boundary logic.
- Tray ordering/symbol behavior is validated through deterministic activity ordering logic and symbol selection in tray rendering code paths.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/src/feed/runtime.rs`
- `src-tauri/src/tray.rs`
