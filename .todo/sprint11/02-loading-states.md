---
status: done
---

# Loading & refresh animations

## Goal

Add loading/refresh animations to the menubar panel and main screen for all three loading scenarios: startup, manual refresh, and background refresh. Pulled from backlog item `optional-loading-animations.md`.

## Scenarios

- **Startup** — feeds loading for the first time (no cached data). Show a prominent skeleton/shimmer.
- **Manual refresh** — user clicks the refresh button in the footer. Show an inline indicator that refresh is in progress.
- **Background refresh** — automatic interval refresh while existing data is displayed. Very subtle — a small spinner or brief flash on the refresh icon, not a full-screen loader.

## Acceptance criteria

- [ ] Menubar panel shows a shimmer skeleton on initial load (already exists — verify it uses the new animation tokens).
- [ ] Main screen shows a comparable loading skeleton on initial load (currently may be instant or missing).
- [ ] Manual refresh: the refresh button in the menubar panel footer shows a spinning or pulsing state while feeds are loading.
- [ ] Background refresh: a subtle indicator (e.g., refresh icon rotates briefly) signals that data was just refreshed, without disrupting the current view.
- [ ] All loading animations respect `prefers-reduced-motion`.
- [ ] Transition from loading → loaded is smooth (content fades in rather than popping).

## Notes

- The menubar panel already has a shimmer skeleton (`@keyframes shimmer`). Reuse and extend rather than reinvent.
- Main screen: only the list pane shimmers on startup. The detail pane shows its empty-state placeholder ("Select an activity") since its content depends on a selection — shimmering it would imply something is loading there when nothing is.
- Background refresh indicator should be very restrained — the user shouldn't feel like the app is "busy" when it's just doing a routine poll.
