---
status: done
---

# Restyle the priority section header

## Goal

The "Needs Attention" header in the panel's priority section feels out-of-place compared to the regular feed section headers. Redesign it to feel more cohesive while still signaling urgency.

## Chosen direction

**B -- Subtle Left Border**: 2px left border in `var(--status-attention-negative)`; header text stays `var(--text-tertiary)` like regular feed headers. Remove the ⚑ icon. Change text to "ATTENTION".

Showcase with all alternatives: `showcase-priority-header.html`

## Related files

- `src/main-screen/MainScreenApp.tsx:458` -- renders `<header className="ms-feed-header ms-priority-header">Needs Attention</header>`
- `src/main-screen/main-screen.css:320-322` -- `.ms-priority-header` override (currently just sets color to attention-negative)
- `src/main-screen/main-screen.css:92-100` -- `.ms-feed-header` base styles (uppercase, tertiary, semibold, xs)
- `src/shared/tokens.css` -- design tokens

## Acceptance criteria

- [x] User picks a direction from the showcase -- **B (Subtle Left Border)**
- [x] Priority header is restyled to match the chosen direction
- [x] Header still feels distinct enough to convey urgency, but consistent with overall list pane style
- [x] Both dark and light themes look correct
- [x] `just check` passes

## Notes

- The current header uses the ⚑ flag character and full attention-negative color. Regular feed headers are uppercase, tertiary-colored, and have no icons.
- Whatever direction is chosen, make sure the separator below the priority section still works visually.
