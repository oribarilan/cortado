---
status: pending
---

# Restyle the priority section header

## Goal

The "Needs Attention" header in the panel's priority section feels out-of-place compared to the regular feed section headers. Redesign it to feel more cohesive while still signaling urgency.

## Showcase

A showcase with 5 alternatives (dark + light themes) is at:

`showcase-priority-header.html`

Open in a browser, pick a direction, then implement.

## Related files

- `src/main-screen/MainScreenApp.tsx:458` -- renders `<header className="ms-feed-header ms-priority-header">Needs Attention</header>`
- `src/main-screen/main-screen.css:320-322` -- `.ms-priority-header` override (currently just sets color to attention-negative)
- `src/main-screen/main-screen.css:92-100` -- `.ms-feed-header` base styles (uppercase, tertiary, semibold, xs)
- `src/shared/tokens.css` -- design tokens

## Acceptance criteria

- [ ] User picks a direction from the showcase
- [ ] Priority header is restyled to match the chosen direction
- [ ] Header still feels distinct enough to convey urgency, but consistent with overall list pane style
- [ ] Both dark and light themes look correct
- [ ] `just check` passes

## Notes

- The current header uses the ⚑ flag character and full attention-negative color. Regular feed headers are uppercase, tertiary-colored, and have no icons.
- Whatever direction is chosen, make sure the separator below the priority section still works visually.
