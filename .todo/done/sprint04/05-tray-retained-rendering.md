---
status: done
---

# Tray rendering for retained activities

## Goal

Render retained activities distinctly and consistently in the native tray menu.

## Acceptance criteria

- [x] Retained activities use hollow dot prefix (`◦`) in activity titles.
- [x] Retained activities render after active activities within each feed section.
- [x] Existing field rendering and Open action behavior continue to work for retained activities.
- [x] Existing error/empty/feed submenu behavior remains intact.
- [x] `just check` passes.

## Notes

- Do not add `(retained)` title suffix in sprint04.

## Relevant files

- `src-tauri/src/tray.rs`
