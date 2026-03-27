---
status: pending
---

# 12 — Close Panel on Footer Action

## Goal

When "Open App" or "Settings" is clicked in the menubar panel footer, close the panel before performing the action. This prevents the panel from lingering behind the main screen or settings window.

## Acceptance Criteria

- [ ] Clicking "Open App" in the menubar panel footer closes the panel, then opens the main screen
- [ ] Clicking "Settings" in the menubar panel footer closes the panel, then opens the settings window
- [ ] Other footer actions (Refresh feeds, Quit) remain unchanged

## Notes

- The panel can be hidden via `invoke("hide_menubar_panel")` or by emitting `menubar_panel_did_resign_key`. Simplest approach: add a `hide_menubar_panel` Tauri command (similar to `hide_main_screen_panel`) and call it from the frontend before invoking the action. Alternatively, hide the panel from Rust inside the `open_main_screen` and `open_settings` commands using the panel's `order_out`.
