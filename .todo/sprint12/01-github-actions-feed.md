---
status: pending
---

# GitHub Actions feed (`github-actions`)

## Goal

Add a curated `github-actions` feed type that tracks workflow run status for a repository. Uses the `gh` CLI (already a dependency of `github-pr`).

## Config

```toml
[[feed]]
name = "my ci"
type = "github-actions"
repo = "owner/repo"               # Required

# Optional filters
branch = "main"                    # Only runs on this branch
workflow = "ci.yml"                # Only this workflow file
event = "push"                     # Filter by trigger event (push, pull_request, etc.)
user = "@me"                       # Only runs triggered by this user
```

## Auth & preflight

Identical to `github-pr`:
1. `gh --version` -- binary exists
2. `gh auth status` -- authenticated

Share this preflight logic with `github-pr` by extracting a common helper (e.g., `gh_preflight()` in a shared module or a function in `mod.rs`).

## Data source

```sh
gh run list --repo OWNER/REPO --limit 20 --json name,status,conclusion,headBranch,event,url,updatedAt,workflowName,databaseId,number
```

Optional filters passed as CLI flags:
- `--branch BRANCH`
- `--workflow WORKFLOW`
- `--event EVENT`
- `--user USER`

## Provided fields

| Field        | Type   | Label       | Description                              |
|-------------|--------|-------------|------------------------------------------|
| `status`    | status | Status      | Run status (passing, failing, running)   |
| `branch`    | text   | Branch      | Head branch name                         |
| `workflow`  | text   | Workflow    | Workflow name                            |
| `event`     | text   | Event       | Trigger event (push, pull_request, etc)  |

## Status kind mapping

| Condition                                       | Value       | StatusKind        |
|-------------------------------------------------|-------------|-------------------|
| conclusion = failure/timed_out/startup_failure   | `failing`   | AttentionNegative |
| conclusion = cancelled                          | `cancelled` | AttentionNegative |
| status = in_progress                            | `running`   | Running           |
| status = queued / waiting                       | `queued`    | Waiting           |
| conclusion = success                            | `passing`   | Idle              |
| conclusion = skipped / neutral                  | `skipped`   | Idle              |
| fallback                                        | `unknown`   | Idle              |

## Activity identity

Run URL from `gh` JSON output. Fallback: `{repo}/actions/runs/{databaseId}`.

## Activity title

`{workflowName} #{number}` (e.g., `CI #482`).

## Default interval

`120s` (shares GitHub API rate limits with `github-pr`).

## Acceptance criteria

- [ ] `src-tauri/src/feed/github_actions.rs` implements `Feed` trait
- [ ] Config parsing validates `repo` is present
- [ ] Optional filters (`branch`, `workflow`, `event`, `user`) are passed as CLI flags when present
- [ ] Preflight checks shared with `github-pr` via extracted helper
- [ ] All status/conclusion values mapped to StatusKind per table above
- [ ] Field overrides supported
- [ ] Registered in `instantiate_feed()` in `mod.rs`
- [ ] Unit tests: config validation, status mapping (all conclusion/status values), field overrides, preflight
- [ ] `specs/main.md` updated: replace the existing `github-actions` future-feed entry with the final field contract and config example
- [ ] `just check` passes

## Notes

- This is the lowest-effort feed to add. Same CLI, same auth, same JSON parsing pattern as `github-pr`.
- The `gh run list` JSON schema is well-documented and stable.
- Cap results at 20 (consistent with existing feeds).
- Consider whether to also support a "latest per workflow" mode in the future (one activity per workflow file showing only the latest run). Not required for this task.

## Relevant files

- `src-tauri/src/feed/github_actions.rs` -- new file
- `src-tauri/src/feed/mod.rs` -- register feed type
- `src-tauri/src/feed/github_pr.rs` -- extract shared `gh` preflight
- `specs/main.md` -- update config docs
