---
status: done
---

# Enforce field overrides

## Goal

Apply config-driven field overrides consistently so `[feed.fields.<name>]` visibility and labels affect runtime output.

## Acceptance criteria

- [x] `visible = false` removes matching fields from emitted activity field lists.
- [x] `label = "..."` overrides the rendered label for matching fields.
- [x] Override application order is deterministic:
  - base feed field metadata defaults
  - feed-type explicit config (if present)
  - `[feed.fields.<name>]` override as final source of truth
- [x] `provided_fields` metadata reflects label overrides but is not filtered by `visible`.
- [x] Override behavior is applied consistently for `github-pr` and `shell` feeds.
- [x] Unknown override keys are ignored safely (no panic/no hard error).
- [x] Dead code related to ignored overrides is removed.
- [x] `just check` passes.

## Notes

- Prefer a shared helper to avoid duplicated override logic across feed implementations.

## Relevant files

- `src-tauri/src/feed/mod.rs` (or a new helper module under `src-tauri/src/feed/`)
- `src-tauri/src/feed/github_pr.rs`
- `src-tauri/src/feed/shell.rs`
