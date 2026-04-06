---
status: done
---

# Frontend update

## Goal

Replace all Bean/Watch terminology in the frontend with Feed/Activity. Fetch data from the `list_feeds` Tauri command instead of using hardcoded starter data.

## Acceptance criteria

- [ ] TypeScript types renamed: `Bean` → removed, `Watch` → removed. New types: `FeedSnapshot` (with optional `error`), `Activity`, `Field`, `FieldValue`
- [ ] Frontend types match the Rust `FeedSnapshot`, `Activity`, `Field`, `FieldValue` serialization
- [ ] `App.tsx` calls `invoke("list_feeds")` and renders the result
- [ ] CSS class names updated: `bean-list` → `feed-list`, `bean-card` → `feed-card`, `bean-name` → `feed-name`, etc.
- [ ] `watch-row` → `activity-row` (or similar) with field rendering
- [ ] Empty state: if no feeds, show a helpful message pointing to the config file
- [ ] Config error state: if a feed has a config error, show the error message in the feed card instead of activities
- [ ] Poll error state: if a feed fails to poll, show a feed-level error status
- [ ] `just check` passes (tsc + clippy)

## Notes

- Remove the `starterBeans` hardcoded data entirely.
- The invoke call should happen on component mount (existing `useEffect` pattern).
- Field rendering should handle all `FieldValue` variants: text, status (with severity styling), number, url (as link).
- Status severity → CSS: success (green), warning (amber), error (red), pending (blue/gray), neutral (default).
- Keep the existing panel layout (max-width, padding, etc.) -- just update the data model.

## Relevant files

- `src/App.tsx` -- types, data fetching, rendering
- `src/styles.css` -- class name updates, field/status styling
