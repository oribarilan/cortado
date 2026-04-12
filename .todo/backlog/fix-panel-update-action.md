---
status: done
---

# Install-update action not clickable in the panel

## Problem

The "install update" action (for both app updates and plugin updates) works correctly in the tray but is not clickable in the panel. When a `cortado-update` activity is selected in the panel's detail pane, no action button is rendered -- the user cannot install the update from the panel.

## Root cause

The panel's `DetailPane` in `src/main-screen/MainScreenApp.tsx` only handles two action cases: restart and open. It has no branch for update activities. When a cortado-update activity is selected:

1. `supportsRestart(activity)` -> `false` (activity.action is `null`)
2. `canOpen` -> `false` (no URL or focus field)
3. Result: falls through to `null`, no button rendered.

The tray (`src/App.tsx:362-381`) works because it explicitly checks `supportsUpdate(feed)` before the restart/open branches and renders dedicated install buttons with their own state and handlers.

## Fix

Port the tray's update-action handling to the panel's `DetailPane`:

1. Import `supportsUpdate` and `isPluginUpdate` from `src/shared/utils.ts`.
2. Add `installing` / `pluginInstalling` state to the detail pane.
3. Add `installUpdate` and `installPluginUpdate` handlers (calling `invoke("install_update")` and `invoke("install_opencode_plugin")`).
4. Add the update button branch in the detail pane's action area, before the restart/open checks.
5. Add the update case to `openFocusedActivity` for keyboard support (Enter key).

## Relevant files

- `src/main-screen/MainScreenApp.tsx` -- panel detail pane (needs changes)
- `src/App.tsx:362-381` -- tray update buttons (reference implementation)
- `src/shared/utils.ts` -- `supportsUpdate`, `isPluginUpdate`, `supportsRestart`
- `src-tauri/src/feed/cortado_update.rs` -- backend update feed
- `src-tauri/src/command.rs:282` -- `install_update` Tauri command
