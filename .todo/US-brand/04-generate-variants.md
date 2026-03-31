---
status: pending
---

# Generate all platform icon variants

## Goal

Use `tauri icon` to generate all required icon sizes and formats from the app icon source.

## Acceptance criteria

- [ ] `pnpm exec tauri icon src-tauri/icons/app-icon.png` runs successfully
- [ ] All generated files in `src-tauri/icons/` are updated (icns, ico, PNGs, iOS, Android)
- [ ] `tauri.conf.json` bundle.icon paths still match the generated files
- [ ] `just check` passes

## Notes

- `tauri icon` overwrites existing files in `src-tauri/icons/` — that's intentional.
- It does NOT touch `tray.png` — that's a separate runtime asset.
- Verify the generated `icon.png` (512x512) looks good as a downscaled version.
- The Square*.png files are Windows Store assets — they'll be generated but aren't critical for macOS.
