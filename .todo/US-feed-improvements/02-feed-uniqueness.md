---
status: done
---

# GitHub Actions: deduplicate by workflow name

## Goal

GitHub Actions feed returns the last N runs across all workflows, which means a repo with one workflow file shows N historical runs of the same workflow. Only the latest run per workflow matters. Deduplicate by workflow name, always on (no config toggle).

## Design

In `github_actions.rs`, after fetching runs from `gh run list` (which returns runs in reverse chronological order), filter to keep only the first occurrence of each workflow name before mapping to activities:

```rust
let mut seen = HashSet::new();
let activities: Vec<Activity> = runs
    .into_iter()
    .filter(|run| seen.insert(run.name.clone()))
    .map(|run| map_run_to_activity(run, &self.repo, &self.config_overrides))
    .take(MAX_ACTIVITIES_PER_FEED)
    .collect();
```

Since `gh run list` returns runs newest-first, the first occurrence of each workflow name is already the latest run. No frontend changes needed.

## Acceptance criteria

- [ ] GitHub Actions feed returns only the latest run per workflow name
- [ ] If a repo has 5 workflows, the feed shows 5 activities (one per workflow)
- [ ] If a repo has 1 workflow, the feed shows 1 activity (the latest run)
- [ ] Deduplication preserves the most recent run (not oldest)

## Related files

- `src-tauri/src/feed/github_actions.rs` -- `poll()` method, `GhWorkflowRun` struct

## Notes

- Other feeds (GitHub PRs, ADO PRs) are already unique per PR, no dedup needed.
- If a user needs full history for a single workflow, they can use the `--workflow` filter.
