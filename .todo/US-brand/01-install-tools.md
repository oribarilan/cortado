---
status: pending
---

# Install icon tools

## Goal

Ensure `rsvg-convert` (from librsvg) is installed so we can convert SVGs to PNGs at exact pixel sizes.

## Acceptance criteria

- [ ] `rsvg-convert --version` works
- [ ] Can convert a test SVG to PNG

## Notes

Install via Homebrew: `brew install librsvg`. This is a well-maintained GNOME project — the standard SVG rasterizer on Linux, also works great on macOS.

`tauri icon` (already available via `pnpm exec tauri icon`) handles the rest of the pipeline.
