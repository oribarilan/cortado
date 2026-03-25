---
status: done
---

# Add `checks` rollup to `ado-pr` feed

## Goal

Expose a `checks` status field for Azure DevOps PR activities using Azure CLI policy evaluations, with a simple deterministic rollup.

## Acceptance criteria

- [ ] `specs/main.md` is updated to un-defer checks for `ado-pr` and add `checks` to the curated fields table.
- [ ] `ado-pr` adds a `checks` field to `provided_fields()` and activity fields.
- [ ] Implementation fetches per-PR policy states via `az repos pr policy list --id <PR_ID>` using explicit flags (`--organization <org> --detect false --output json`). Note: `--project` is not supported by this command; the PR ID is unique per org.
- [ ] Rollup filters to **CI policies only** (Build and Status type IDs); reviewer/approval policies are excluded since the `review` field covers that.
- [ ] Rollup mapping is implemented exactly:
  - [ ] any `rejected` or `broken` => `failed`
  - [ ] else any `queued` or `running` => `running`
  - [ ] `notApplicable` is ignored in rollup
  - [ ] else => `succeeded`
  - [ ] zero policies or all `notApplicable` => `succeeded`
- [ ] `ado-pr` tray field ordering includes `checks` (alongside review/mergeable/draft), influencing the activity dot color.
- [ ] Calls are bounded by the existing per-feed activity cap (20 PRs max).
- [ ] Unknown/unexpected policy states are mapped to neutral `"<state> (unknown)"` (do not crash poll).
- [ ] Per-PR policy-call failures do not fail the whole feed poll; that PR's checks value becomes neutral unknown while other PRs continue.
- [ ] Bounded concurrency (max 5 in flight) is extracted into a shared module with single-responsibility design and its own tests. Per-call timeout is 30s (same as `AZ_POLL_TIMEOUT`).
- [ ] Unit tests cover failed/running/succeeded rollups, unknown-state behavior, empty policy list, and all-notApplicable edge case.
- [ ] `just check` passes.

## Notes

- Keep this implementation CLI-only (no REST client dependency).
- Keep complexity low: v1 can use one additional policy call per returned PR.
- The bounded-concurrency module should be reusable by other feeds that need per-activity async work.

## Relevant files

- `src-tauri/src/feed/ado_pr.rs`
- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/feed/github_pr.rs` (reference: existing `checks` field pattern)
- `src-tauri/src/process.rs` (`ProcessRunner` abstraction for CLI calls)
- `src-tauri/src/tray.rs`
- `specs/main.md`
