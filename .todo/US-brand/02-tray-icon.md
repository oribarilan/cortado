---
status: pending
---

# Design and generate tray icon

## Goal

Create a coffee-themed menubar icon that works as a macOS template image.

## Acceptance criteria

- [ ] `src-tauri/icons/tray.svg` exists with the source design
- [ ] `src-tauri/icons/tray.png` is 22x22, monochrome black on transparent
- [ ] Icon renders correctly in both light and dark menu bars (template tinting)
- [ ] Shape is recognizable at 22px — reads as a cortado glass

## Notes

- Design as SVG at a larger canvas (e.g., 44x44 or 88x88) for clean authoring, then export at 22x22.
- Use only black (#000000) fill with alpha transparency. No grays, no colors — template images are binary (opaque vs transparent). Partial alpha is allowed for anti-aliasing.
- The glass silhouette should be simple: a short, wide-mouthed glass shape with a slight taper. Maybe a subtle horizontal line suggesting the espresso/milk layer boundary.
- Test by replacing the existing `tray.png` and running `just dev`.
