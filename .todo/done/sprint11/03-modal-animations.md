---
status: done
---

# Modal entrance & exit animations

## Goal

Animate the entrance and exit of modal overlays in Settings (reset-defaults confirmation, delete-feed confirmation) so they feel deliberate rather than instant.

## Acceptance criteria

- [ ] Modal backdrop fades in on open (`opacity: 0 → 1`) using `--duration-normal`.
- [ ] Modal dialog scales up slightly on entrance (e.g., `scale(0.97) → scale(1)`) combined with opacity fade, using `--ease-out`.
- [ ] On close, the reverse animation plays (fade out + scale down) before the modal is removed from the DOM or hidden.
- [ ] Both the reset-defaults modal and the delete-feed confirmation use the same animation pattern.
- [ ] Animations respect `prefers-reduced-motion` (instant show/hide when reduced motion is preferred).
- [ ] Keyboard dismissal (Escape) and button dismissal both trigger the exit animation.

## Notes

- Currently modals appear/disappear instantly via conditional rendering.
- The exit animation requires either a brief delay before unmounting (e.g., `onAnimationEnd` callback) or keeping the element mounted with a hidden state. Choose whichever is simpler.
- Keep the scale subtle -- `0.97` to `1.0` is enough. Big zooms feel dated.
