---
status: pending
---

# GitHub PR feed stub

## Goal

Implement a `GithubPrFeed` struct that satisfies the `Feed` trait. Returns hardcoded activities (no real GitHub API calls). Proves the trait works for a multi-activity, field-rich feed type.

## Acceptance criteria

- [ ] `src-tauri/src/feed/github_pr.rs` exists with `GithubPrFeed` struct
- [ ] Implements `Feed` trait with hardcoded poll data
- [ ] `provided_fields()` returns: review (status), checks (status), mergeable (status), draft (status), labels (text)
- [ ] `poll()` returns 2-3 fake PR activities with realistic field values
- [ ] Can be constructed from a `FeedConfig` (takes name, repo from config)
- [ ] `just check` passes

## Notes

- This is a stub. No HTTP client, no GitHub token, no real API calls.
- The hardcoded data should look realistic so the UI can be developed against it.
- The constructor should extract `repo` from the config's type-specific table and return an error if it's missing.
- No `user` field — the real implementation (sprint 02) will use `gh` CLI which handles auth/identity.
- No `token` or auth fields — `gh` CLI handles all GitHub authentication.

## Relevant files

- `src-tauri/src/feed/github_pr.rs` — new file
- `src-tauri/src/feed/mod.rs` — add `pub mod github_pr;`
