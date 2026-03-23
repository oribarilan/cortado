---
status: done
---

# Per-feed retain duration plumbing

## Goal

Add optional per-feed `retain` duration config and expose it to runtime lifecycle logic.

## Acceptance criteria

- [x] Feed config supports optional `retain` duration string.
- [x] Omitted `retain` resolves to no retention.
- [x] Invalid `retain` values produce clear config validation errors.
- [x] Feed trait/runtime plumbing exposes retention duration per feed without feed-specific hacks.
- [x] Existing feed behavior remains unchanged when `retain` is omitted.
- [x] `just check` passes.

## Notes

- Keep primitive generic for all feeds, not github-only.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/feed/github_pr.rs`
- `src-tauri/src/feed/shell.rs`
