---
status: pending
---

# Real GitHub PR polling via gh CLI

## Goal

Replace the GitHub PR stub with live polling through `gh` CLI so activities reflect actual open PR state for the configured repository.

## Acceptance criteria

- [ ] `GithubPrFeed::poll()` calls `gh pr list` with explicit repo and JSON fields.
- [ ] Command invocation is deterministic and single-shot per poll:
  - `gh pr list --repo <owner/repo> --state open --limit 100 --json number,title,url,isDraft,labels,mergeable,reviewDecision,statusCheckRollup`
- [ ] Command shape uses one request per poll, including at least:
  - `number`, `title`, `url`, `isDraft`, `labels`, `mergeable`, `reviewDecision`, `statusCheckRollup`
- [ ] JSON output is parsed into typed Rust structures and mapped into `Activity` rows.
- [ ] Feed fields are populated from real data:
  - `review` (status)
  - `checks` (status)
  - `mergeable` (status)
  - `draft` (status)
  - `labels` (text)
- [ ] Mapping rules are deterministic:
  - `reviewDecision`: `APPROVED`→success `approved`; `CHANGES_REQUESTED`→warning `changes requested`; `REVIEW_REQUIRED`→pending `awaiting`; unknown/null→neutral `unknown`
  - `mergeable`: `MERGEABLE`→success `yes`; `CONFLICTING`→error `no`; `UNKNOWN`→pending `unknown`; `null`→neutral `unknown`
  - `draft`: `isDraft=true`→pending `yes`; `isDraft=false`→neutral `no`
  - `labels`: comma-separated label names sorted lexicographically; empty label set emits empty string
  - `checks` from `statusCheckRollup` in same response only (no per-PR follow-up calls):
    - any failing/error/cancelled conclusion present → error `failing`
    - else any pending/in_progress/queued/waiting state present → pending `pending`
    - else at least one successful/skipped/neutral conclusion and no pending/error → success `passing`
    - else (no check data) → neutral `unknown`
- [ ] Checks status is derived from available CLI output in the same poll request (no per-PR N+1 calls in this sprint).
- [ ] Missing `gh` binary yields a clear actionable error message.
- [ ] Auth/API failures surface stderr/context in feed error output.
- [ ] Empty PR set returns success with zero activities (not an error).
- [ ] `just check` passes.

## Notes

- This sprint should prioritize reliability and low overhead: one `gh` process + one API request per feed poll.
- Keep any extra/derived fields optional for future sprints unless required by spec.

## Relevant files

- `src-tauri/src/feed/github_pr.rs`
