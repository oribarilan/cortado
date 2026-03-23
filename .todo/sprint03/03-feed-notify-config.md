---
status: pending
---

# Per-feed notification config parsing

## Goal

Add a per-feed `notify` setting to feed config parsing and runtime feed construction so notification behavior is configurable per feed.

## Acceptance criteria

- [ ] Feed config supports `notify = true|false` on each `[[feed]]` entry.
- [ ] `notify` is optional and defaults to `false` when omitted.
- [ ] Invalid `notify` types produce a clear config validation error.
- [ ] Parsed notify config is available to runtime/feed instances without relying on ad-hoc TOML access.
- [ ] Existing config keys (`name`, `type`, `interval`, type-specific fields, field overrides) keep current behavior.
- [ ] Config parser tests include notify present/omitted/invalid cases.
- [ ] `just check` passes.

## Notes

- Keep config shape intentionally simple in sprint03; richer notification policy can come later.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/feed/github_pr.rs`
- `src-tauri/src/feed/shell.rs`
