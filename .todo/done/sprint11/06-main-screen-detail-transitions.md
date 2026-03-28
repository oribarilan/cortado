---
status: done
---

# Panel detail pane & priority section transitions

## Goal

Add subtle content transitions to the panel: a crossfade in the detail pane when the selected activity changes, and an enter/exit animation for the priority ("Needs Attention") section.

## Acceptance criteria

### Detail pane crossfade
- [ ] When the user changes selection (keyboard or mouse), the detail pane content crossfades to the new activity's details.
- [ ] The crossfade is fast (`--duration-fast` to `--duration-normal`) — keyboard navigation should never feel laggy.
- [ ] If the user is arrowing rapidly through the list, the animation should not queue up or stutter — debounce or cancel in-flight transitions.
- [ ] Respects `prefers-reduced-motion`.

### Priority section enter/exit
- [ ] When the "Needs Attention" section first appears (e.g., after a refresh adds an `AttentionNegative` activity), it animates in — slide down + fade, or height expand.
- [ ] When it disappears (no more attention items), it animates out smoothly.
- [ ] Respects `prefers-reduced-motion`.

## Notes

- The detail pane crossfade is the highest-risk animation in the sprint — if it feels sluggish during fast keyboard navigation, it's worse than no animation. Test carefully with rapid up/down arrow.
- Consider using a CSS transition on `opacity` keyed to the activity ID, rather than React unmount/remount cycles.
- The priority section animation can reuse the `grid-template-rows` expand/collapse pattern already proven in the menubar panel.
