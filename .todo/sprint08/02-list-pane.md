---
status: pending
---

# 02 — List Pane

## Goal

Build the left-side activity list for the main screen. Feed-grouped sections with compact rows (status dot + title). Full keyboard navigation with visible focus tracking.

## Acceptance Criteria

- [ ] Left pane (~220px) shows feed sections with uppercase feed-name headers
- [ ] Each activity row: status dot (colored by derived status kind) + title (truncated with ellipsis)
- [ ] Retained activities show hollow dot
- [ ] Arrow keys (↑/↓) move focus through rows, skipping feed headers
- [ ] Focused row has visible outline (blue focus ring, keyboard-active only)
- [ ] Enter on a focused row opens the activity URL via `open_activity` command
- [ ] First activity is focused when the panel opens
- [ ] Empty state: when no feeds are configured, show a message similar to the menubar panel ("No feeds configured…"). When feeds exist but have zero activities, show "No activities" per feed.
- [ ] Activities without an openable URL: Enter does nothing (same as current `supportsOpen` logic)
- [ ] Data sourced from `list_feeds` command on mount and `feeds-updated` event
- [ ] State resets when panel is shown (via a `main_screen_panel_will_show` event or similar)
- [ ] Footer bar with keyboard hint legend (↑↓ navigate · ↵ open · esc close) and a small gear icon to open Settings
- [ ] Gear icon opens settings window via `open_settings` command
- [ ] ⌘, also opens settings (standard macOS convention)
- [ ] ⌘Q quits the app when the main screen is focused

## Notes

- Share TypeScript types (`FeedSnapshot`, `Activity`, `Field`, `StatusKind`, etc.) with the menubar panel — consider extracting to a shared module under `src/shared/` or `src/types.ts`
- Utility functions like `deriveActivityKind`, `supportsOpen`, `kindPriority` should also be shared
- The list doesn't show status chips — that's the detail pane's job. Keep rows minimal.
