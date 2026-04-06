---
status: done
---

# Animation tokens & reduced-motion foundation

## Goal

Establish shared CSS custom properties for animation durations and easings, and normalize existing animations to use them. This gives all three windows (menubar panel, panel, settings) a consistent motion language.

## Acceptance criteria

- [ ] A shared set of CSS custom properties defined on `:root` in each CSS file (or a shared import):
  - `--duration-fast` (~100ms) -- hover states, micro-interactions
  - `--duration-normal` (~180ms) -- most transitions (expand, crossfade, section switch)
  - `--duration-slow` (~280ms) -- larger reveals, modals
  - `--ease-out` -- default easing for entrances
  - `--ease-in-out` -- default easing for state changes
- [ ] Existing hardcoded transition durations in `styles.css`, `main-screen.css`, and `settings.css` are replaced with the new tokens.
- [ ] `prefers-reduced-motion: reduce` block updated to reset all duration tokens to `0ms`, so every animation built on tokens automatically respects the preference.
- [ ] No visual regression -- existing animations look and feel the same after the swap.

## Notes

- The three CSS files currently define their own transition values independently. Unify them.
- Don't add a shared CSS file import unless the build already supports it -- duplicating the `:root` block in each file is fine for now.
- Consider a `--duration-none: 0ms` token for explicitly non-animated states.
