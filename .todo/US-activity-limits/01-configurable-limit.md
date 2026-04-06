---
status: pending
---

# Configurable activity limit per feed

## Goal

Replace the hard-coded `MAX_ACTIVITIES_PER_FEED = 20` with a per-feed configurable limit, exposed in both the settings UI and the TOML config.

## Context

Every feed type has `const MAX_ACTIVITIES_PER_FEED: usize = 20` and uses it in two places:
- As the `--limit` / `--top` argument to the CLI query (so the API only returns N results).
- As a post-fetch `.take(N)` / `.truncate(N)` safety cap.

There's also a runtime-level truncation in `feed/runtime.rs:304`.

## Acceptance criteria

- [ ] `FeedConfig` supports an optional `limit` field (positive integer).
- [ ] Each feed type reads `limit` from config and uses it for both the CLI query limit and the post-fetch truncation.
- [ ] If `limit` is not set, the current default (20) is preserved.
- [ ] The runtime-level cap in `feed/runtime.rs` respects the per-feed limit rather than its own hard-coded constant.
- [ ] Frontend: the `limit` field is exposed in the settings form as a common field for all feed types (number input).
- [ ] Validation: limit must be a positive integer with reasonable bounds (e.g., 1-100).
- [ ] Existing tests pass; new tests cover config parsing of the `limit` field.

## Related files

- `src-tauri/src/feed/config.rs` -- `FeedConfig` struct
- `src-tauri/src/feed/github_pr.rs` -- `MAX_ACTIVITIES_PER_FEED`, `--limit` arg, `.take()`
- `src-tauri/src/feed/github_actions.rs` -- same pattern
- `src-tauri/src/feed/ado_pr.rs` -- same pattern, uses `--top` instead of `--limit`
- `src-tauri/src/feed/runtime.rs:304` -- runtime-level truncation
- `src/shared/feedTypes.ts` -- feed form field definitions
- `src/settings/SettingsApp.tsx` -- settings form rendering
