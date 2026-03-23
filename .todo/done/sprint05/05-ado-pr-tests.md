---
status: done
---

# `ado-pr` unit test coverage

## Goal

Add deterministic tests for Azure DevOps feed behavior using mocked process runners.

## Acceptance criteria

- [x] Config validation tests cover required keys and invalid values.
- [x] Dependency checks tests cover:
  - [x] missing `az`
  - [x] missing `azure-devops` extension
  - [x] unauthenticated (needs `az login`)
- [x] Poll mapping tests cover stable conversion to fields: `review`, `mergeable`, `draft`, `labels`.
- [x] Empty PR set test returns empty activities without error.
- [x] Tests are isolated (no real Azure/API/CLI dependency).
- [x] `just check` passes.

## Relevant files

- `src-tauri/src/feed/ado_pr.rs`
- `src-tauri/src/feed/dependency.rs`
