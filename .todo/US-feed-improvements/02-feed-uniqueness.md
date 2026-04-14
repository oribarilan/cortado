---
status: pending
---

# Feed uniqueness / deduplication

## Goal

Some feed types return multiple activities that represent different runs of the same logical item. For example, GitHub Actions returns the last 20 workflow runs, but if a repo has one workflow file, you see 20 historical runs of the same workflow -- only the latest matters.

Add a uniqueness mechanism so feeds can deduplicate activities by a grouping key, keeping only the most recent activity per group.

## Design

### Grouping key

Each feed type that supports deduplication defines a **grouping key** derivable from the activity. For GitHub Actions, the natural grouping key is the workflow name (the `name` field from `gh run list`, which corresponds to the workflow YAML file).

### Where to deduplicate

**Option A: Backend (in the feed's `poll()` method)**
Each feed type deduplicates its own activity list before returning. This is the simplest approach -- GitHub Actions would group runs by `name` and keep only the first (most recent) per group.

**Option B: Backend (in the feed trait / registry layer)**
Add an optional `unique_by` concept to the `Feed` trait or `FeedConfig`, so the registry can deduplicate after polling. More generic but potentially over-engineered for the current need.

**Recommendation**: Option A. Each feed knows its own semantics best. GitHub Actions deduplicates by workflow name. Other feeds can add their own logic when needed.

### Implementation for GitHub Actions

In `github_actions.rs`, after mapping runs to activities:

```rust
// Keep only the latest run per workflow name
let mut seen = HashSet::new();
let activities: Vec<Activity> = runs
    .into_iter()
    .filter(|run| seen.insert(run.name.clone()))
    .map(|run| map_run_to_activity(run, &self.repo, &self.config_overrides))
    .take(MAX_ACTIVITIES_PER_FEED)
    .collect();
```

Since `gh run list` returns runs in reverse chronological order, the first occurrence of each workflow name is already the latest run.

### Config option

Consider an optional `unique: true` config field per feed that enables dedup. Default could be `true` for GitHub Actions (since historical runs are rarely useful) or we can leave it always-on for this feed type and add config later if users want the full history.

## Acceptance criteria

- [ ] GitHub Actions feed returns only the latest run per workflow name by default
- [ ] If a repo has 5 workflows, the feed shows exactly 5 activities (one per workflow)
- [ ] If a repo has 1 workflow, the feed shows 1 activity (the latest run)
- [ ] Deduplication preserves the most recent run (not oldest)
- [ ] Consider: should this be configurable (e.g., `unique: true/false` in feed config)?

## Related files

- `src-tauri/src/feed/github_actions.rs` -- GitHub Actions feed implementation
- `src-tauri/src/feed/mod.rs` -- Feed trait, if adding trait-level support
- `src-tauri/src/feed/config.rs` -- FeedConfig, if adding config option

## Notes

- Other feeds that might benefit from uniqueness in the future: any feed showing historical runs or events. ADO PRs and GitHub PRs are already unique per PR, so they don't need this.
- The `--workflow` filter already narrows to a single workflow file, but users often want to monitor all workflows in a repo without seeing duplicate history.
