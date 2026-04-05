# Task: Show last-refreshed timestamp on activities

## Context
Activities show their current status but give no indication of *when* the data was last fetched. Users have no way to know if they're looking at fresh data or a stale snapshot from minutes ago. Adding a "last refreshed" indicator builds trust in the displayed information.

**Value delivered**: Every activity shows when its data was last polled, so users can judge data freshness at a glance.

## Related Files
- `src-tauri/src/feed/mod.rs` — `FeedSnapshot`, `Activity` structs
- `src-tauri/src/feed/runtime.rs` — `build_snapshot_for_feed`, poll loop
- `src/main-screen/MainScreenApp.tsx` — Panel detail pane
- `src/App.tsx` — Tray activity detail
- `src/shared/types.ts` — TypeScript types
- `src/shared/utils.ts` — Formatting helpers

## Dependencies
- None

## Acceptance Criteria
- [ ] Each `FeedSnapshot` carries a `last_refreshed` timestamp (unix ms) set when the feed was last polled
- [ ] The panel detail pane shows a human-readable relative time (e.g., "3m ago", "just now") for the selected activity
- [ ] The tray expanded activity detail also shows the relative time
- [ ] The timestamp updates on each poll cycle without requiring user interaction
- [ ] Feeds that have never been polled (pre-seed) show no timestamp (not "unknown" or epoch)
- [ ] Relative time formatting follows existing patterns in the codebase (see `relative_time` in `harness/feed.rs`)

## Scope Estimate
Small

## Notes
- The timestamp belongs on `FeedSnapshot` (per-feed), not per-activity — all activities in a feed are refreshed together.
- Consider reusing the `relative_time` helper already in `harness/feed.rs` or extracting it to a shared location.
- Display location: in the panel, the detail pane footer or a subtle line below the fields. In the tray, below the expanded fields. Keep it unobtrusive.
