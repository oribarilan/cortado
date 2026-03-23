---
status: done
---

# Feed system unit tests

## Goal

Add focused Rust unit tests for feed parsing/mapping/override behavior and cache error semantics so sprint02 behavior is stable.

## Acceptance criteria

- [x] Config parser tests cover: valid config, missing required keys, invalid interval, duplicate names, missing file behavior.
- [x] Shell feed tests cover: required `command`, supported `field_type` parsing, output mapping edge cases.
- [x] GitHub feed tests cover: required `repo`, JSON mapping for review/checks/mergeable/draft/labels.
- [x] Field override tests verify hide + relabel behavior and unknown override tolerance.
- [x] Background cache behavior tests verify stale activities are retained when a subsequent poll fails.
- [x] Dependency-check tests verify stable errors for missing `gh` and unauthenticated `gh` states (using mocked command outputs).
- [x] Tests avoid external side effects (no real `gh` calls, network, or shell commands in unit tests).
- [x] `just check` passes.

## Notes

- Keep tests isolated and deterministic; use fixture JSON strings and small helper constructors.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/src/feed/shell.rs`
- `src-tauri/src/feed/github_pr.rs`
- `src-tauri/src/feed/mod.rs`
