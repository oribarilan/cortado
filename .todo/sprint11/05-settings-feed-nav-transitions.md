---
status: pending
---

# Settings feed list/edit navigation transitions

## Goal

Animate the transition between the feed list view and the feed edit form in Settings, so navigating into and out of a feed editor feels like a smooth drill-down rather than an abrupt replacement.

## Acceptance criteria

- [ ] Clicking a feed card (or "Add Feed") transitions from the feed list to the edit form with a slide or crossfade.
- [ ] Clicking the breadcrumb back link transitions from the edit form back to the feed list with a reverse animation.
- [ ] Transition duration uses `--duration-normal`.
- [ ] The breadcrumb text updates immediately — only the content area animates.
- [ ] Respects `prefers-reduced-motion` (instant swap when reduced motion is preferred).

## Notes

- A horizontal slide (list slides left, form slides in from right; reverse on back) would feel natural for drill-down navigation. But a simple crossfade is also acceptable if the slide is tricky to implement without layout issues.
- The feed list and edit form have different heights — make sure the transition handles this gracefully (e.g., animate to the new height, or use a min-height so the container doesn't collapse).
- This is conceptually similar to task 04 (section transitions) but the drill-down metaphor may warrant a different animation direction.
