---
status: done
---

# Implement Azure DevOps PR feed (`ado-pr`)

## Goal

Implement `ado-pr` feed polling using Azure CLI and map results to Cortado activities/fields.

## Acceptance criteria

- [x] New feed implementation exists at `src-tauri/src/feed/ado_pr.rs` and implements `Feed` trait.
- [x] Polling uses `az repos pr list` with explicit `--org`, `--project`, `--repository`, `--status active`, `--detect false`, JSON output.
- [x] Dependency preflight checks:
  - [x] `az` CLI availability
  - [x] `azure-devops` extension presence
  - [x] authenticated state via `az login`
- [x] User-facing error messages are clear/actionable for missing CLI, missing extension, unauthenticated state.
- [x] Fields mapped deterministically: `review`, `mergeable`, `draft`, `labels`.
- [x] Activity IDs and open URLs are stable across polls.
- [x] `just check` passes.

## Notes

- Do not add PAT auth handling in sprint05.
- Keep checks/build-policy details out of v1 field mapping.

## Relevant files

- `src-tauri/src/feed/ado_pr.rs`
- `src-tauri/src/feed/process.rs`
- `src-tauri/src/feed/dependency.rs`
