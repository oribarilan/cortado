---
status: done
---

# Show last-refreshed timestamp on activities

## Goal

Every activity shows when its data was last polled, so users can judge data freshness at a glance (e.g., "3m ago", "just now").

## Design decisions

- The timestamp is a `last_refreshed: Option<u64>` field on `FeedSnapshot` (unix ms), set in `build_snapshot_for_feed` in `runtime.rs`. `None` for feeds that have never been polled.
- Frontend receives unix ms and formats it with a `format_relative_time` helper (e.g., "just now", "3m ago", "2h ago").
- **Live ticking**: a React interval timer (every ~30s) re-renders the relative time so it stays fresh between polls.
- A `formatRelativeTime` helper was created in `src/shared/utils.ts` for frontend display.

## Design

- The timestamp belongs on `FeedSnapshot` (per-feed), not per-activity -- all activities in a feed are refreshed together.
- Display location: panel detail pane footer and tray expanded activity detail. Keep it unobtrusive.
- Feeds that have never been polled (pre-seed) show no timestamp.
- The frontend `formatRelativeTime` helper handles live display; the backend Rust `relative_time` in `harness/feed.rs` remains for harness-specific formatting.

## Acceptance criteria

- [ ] Each `FeedSnapshot` carries a `last_refreshed` timestamp (unix ms) set when the feed was last polled
- [ ] The panel detail pane shows a human-readable relative time (e.g., "3m ago", "just now")
- [ ] The tray expanded activity detail also shows the relative time
- [ ] The timestamp updates on each poll cycle without requiring user interaction
- [ ] Feeds that have never been polled show no timestamp (not "unknown" or epoch)
- [ ] Relative time formatting follows existing patterns in the codebase
- [ ] `just check` passes

## Relevant files

- `src-tauri/src/feed/mod.rs` -- `FeedSnapshot`, `Activity` structs
- `src-tauri/src/feed/runtime.rs` -- `build_snapshot_for_feed`, poll loop
- `src/main-screen/MainScreenApp.tsx` -- Panel detail pane
- `src/App.tsx` -- Tray activity detail
- `src/shared/types.ts` -- TypeScript types
- `src/shared/utils.ts` -- Formatting helpers
- `src-tauri/src/feed/harness/feed.rs` -- `relative_time` helper
