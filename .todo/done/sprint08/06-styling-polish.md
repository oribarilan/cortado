---
status: pending
---

# 06 -- Styling & Polish

## Goal

Match the Clean Split showcase visual design. Ensure the panel feels native, cohesive with the menubar panel's design language, and polished in both light and dark mode.

## Acceptance Criteria

- [ ] OKLCH color tokens consistent with `src/styles.css` (panel surface, text levels, status colors)
- [ ] Vibrancy and backdrop-filter blur on the panel surface
- [ ] Dark/light mode following system preference
- [ ] Status dot colors, retained hollow dots, pulse animation for running
- [ ] Focus outline visible only when keyboard-active (same pattern as menubar panel)
- [ ] Smooth but fast transitions (hover: 80-100ms, no expand/collapse animation needed)
- [ ] Feed header styling: uppercase, small, tertiary color
- [ ] Detail pane field grid aligned and readable
- [ ] Footer keyboard hints styled with `kbd` tags
- [ ] Panel border, shadow, and border-radius matching the showcase (~12px radius)
- [ ] No popover arrow -- unlike the menubar panel, the panel is centered, not attached to the tray icon. Use rounded corners only, no `popover` crate.
- [ ] Reduced-motion media query disables animations
- [ ] No horizontal scrollbars anywhere

## Notes

- The panel should feel like it belongs to the same app as the menubar panel, but it's not a copy -- it's wider and split, so spacing/sizing will differ
- Consider extracting shared CSS variables into a common file that both panels import
