---
status: pending
---

# Verify icons in running app

## Goal

Build and run the app to verify both icons look correct in context.

## Acceptance criteria

- [ ] Tray icon renders correctly in menu bar (light and dark mode)
- [ ] App icon appears correctly in Activity Monitor / About dialog
- [ ] `just check` passes
- [ ] No leftover temp files or unused assets

## Notes

- Run `just dev` and visually inspect.
- Toggle macOS appearance (System Settings > Appearance) to verify tray icon template tinting.
- Check Activity Monitor to see the app icon at small size.
