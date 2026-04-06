---
status: pending
---

# Define and document top-N ranking strategy per feed type

## Goal

Ensure each feed type has an intentional, documented ordering strategy for how it picks the "top N" activities. The ordering itself is not user-configurable, but the UI should communicate what "most recent" means for each feed type.

## Context

Currently, feeds rely on the CLI tool's default ordering:
- `gh pr list`: returns open PRs ordered by creation date (newest first).
- `gh run list`: returns runs ordered by `updatedAt` (newest first).
- `az repos pr list`: returns PRs ordered by creation date (newest first).
- `http-health`: only one activity per feed (no ordering needed).
- Harness feeds (`copilot-session`, `opencode-session`): ordered by file modification time.

This ordering happens to be reasonable, but it's implicit -- there's no explicit sort in the feed code, and the behavior isn't documented.

## Acceptance criteria

- [ ] Each feed type's ranking strategy is documented in code comments (what field determines ordering, what ordering is assumed from the CLI).
- [ ] If a feed type's results aren't guaranteed to come pre-sorted from the CLI, add an explicit sort before truncation.
- [ ] For feeds where "recency" could mean different things (e.g., creation date vs last update), the choice is intentional and documented.
- [ ] If any feed type's implicit ordering is found to be wrong or inconsistent, fix it.
- [ ] Consider adding a subtle description in the settings UI or feed tooltip indicating the ordering (e.g., "Shows the N most recently updated PRs").

## Related files

- `src-tauri/src/feed/github_pr.rs` -- PR ordering
- `src-tauri/src/feed/github_actions.rs` -- run ordering
- `src-tauri/src/feed/ado_pr.rs` -- PR ordering
- `src-tauri/src/feed/http_health.rs` -- single activity, no ordering
- `src-tauri/src/feed/harness/` -- harness feed ordering

## Notes

- This task is mostly an audit + documentation pass. Code changes are only needed if a feed's ordering is found to be wrong.
- Consider whether `updatedAt` vs `createdAt` is the right recency metric for PRs. A PR created 6 months ago but updated today might be more relevant than one created yesterday with no activity.
