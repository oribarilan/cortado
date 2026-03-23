---
status: done
---

# `ado-pr` config parsing and validation

## Goal

Add strict config validation for Azure DevOps PR feeds.

## Acceptance criteria

- [x] `ado-pr` requires non-empty string keys: `org`, `project`, `repo`.
- [x] Missing or empty required keys produce clear validation errors.
- [x] `interval` / `retain` shared duration behavior remains unchanged for `ado-pr`.
- [x] Config parser/constructor tests cover valid and invalid `ado-pr` configs.
- [x] `just check` passes.

## Relevant files

- `src-tauri/src/feed/ado_pr.rs`
- `src-tauri/src/feed/config.rs`
