---
status: pending
---

# Show last-refreshed timestamp on activities

## Goal

Every activity shows when its data was last polled, so users can judge data freshness at a glance (e.g., "3m ago", "just now").

## Design

- The timestamp belongs on `FeedSnapshot` (per-feed), not per-activity — all activities in a feed are refreshed together.
- Display location: panel detail pane footer and tray expanded activity detail. Keep it unobtrusive.
- Feeds that have never been polled (pre-seed) show no timestamp.
- Consider reusing the `relative_time` helper already in `harness/feed.rs` or extracting it to a shared location.

## Acceptance criteria

- [ ] Each `FeedSnapshot` carries a `last_refreshed` timestamp (unix ms) set when the feed was last polled
- [ ] The panel detail pane shows a human-readable relative time (e.g., "3m ago", "just now")
- [ ] The tray expanded activity detail also shows the relative time
- [ ] The timestamp updates on each poll cycle without requiring user interaction
- [ ] Feeds that have never been polled show no timestamp (not "unknown" or epoch)
- [ ] Relative time formatting follows existing patterns in the codebase
- [ ] `just check` passes

## Relevant files

- `src-tauri/src/feed/mod.rs` — `FeedSnapshot`, `Activity` structs
- `src-tauri/src/feed/runtime.rs` — `build_snapshot_for_feed`, poll loop
- `src/main-screen/MainScreenApp.tsx` — Panel detail pane
- `src/App.tsx` — Tray activity detail
- `src/shared/types.ts` — TypeScript types
- `src/shared/utils.ts` — Formatting helpers
- `src-tauri/src/feed/harness/feed.rs` — `relative_time` helper
