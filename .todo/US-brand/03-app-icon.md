---
status: pending
---

# Design and generate app icon

## Goal

Create a full-color app icon with the cortado coffee theme.

## Acceptance criteria

- [ ] `src-tauri/icons/app-icon.svg` exists with the source design
- [ ] `src-tauri/icons/app-icon.png` is 1024x1024, RGBA
- [ ] Design uses warm coffee tones and shows the layered cortado look
- [ ] Icon looks good at all sizes (1024 down to 32px)

## Notes

- The app icon can be much richer than the tray icon -- gradients, shadows, depth.
- Design centered in the square canvas. macOS applies its own squircle mask.
- Consider the cortado's distinctive look: a short glass with a visible layer of dark espresso sitting on top of lighter steamed milk.
- Warm palette: espresso browns, cream/milk whites, maybe a subtle glass highlight.
- Test readability at small sizes -- the 32x32 version must still be identifiable.
