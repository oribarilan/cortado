---
status: done
---

# Auto-open panel on app launch

## Goal

Make the panel (main-screen window) automatically open when the app launches, so users immediately see Cortado's UI instead of just a menubar icon.

## Context

Currently, no window auto-shows on startup. The panel only opens via global hotkey (Cmd+Shift+Space), tray "Open App", or app reopen event. New users have no visual cue that the app is running beyond the menubar icon.

## Acceptance criteria

- [ ] Panel (main-screen) opens automatically on app launch
- [ ] Subsequent launches also auto-open the panel
- [ ] The hotkey and tray "Open App" continue to work as toggle (if panel is already visible, they hide it)
- [ ] No regressions to existing panel show/hide behavior

## Notes

- The auto-open should happen after the app setup completes (tray created, feeds started), likely at the end of the `setup()` closure in `main.rs`.
- Call `main_screen::toggle_main_screen_panel` or equivalent — reuse the existing show logic, don't add a new path.
- This is a general behavior change, not specific to the empty state. It benefits all users.

## Relevant files

- `src-tauri/src/main.rs` — app setup, where to add auto-open
- `src-tauri/src/main_screen.rs` — `toggle_main_screen_panel` / show logic
