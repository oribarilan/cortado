---
status: done
---

# 06 — Frontend: Theme + Text Size Wiring

## Goal

Create a shared hook that applies the theme and text-size settings to the DOM, and wire it into all three windows.

## Acceptance Criteria

- [ ] `src/shared/useAppearance.ts` hook exists
- [ ] On mount: invokes `get_settings`, sets `document.documentElement.dataset.theme` and `document.documentElement.dataset.textSize`
- [ ] Listens for Tauri `appearance-changed` event and updates data attributes live
- [ ] Cleans up event listener on unmount
- [ ] Hook is called from `App.tsx`, `MainScreenApp.tsx`, and `SettingsApp.tsx`
- [ ] Theme and text size switch instantly across all open windows when changed
- [ ] Default behavior (no settings file) matches current appearance (system theme, medium text)

## Notes

- The hook should be lightweight — it only reads settings on mount and listens for events. No polling.
- `SettingsApp.tsx` already reads settings on mount — the hook should not duplicate that call. Consider whether the hook can share the existing settings load, or whether a second lightweight call is acceptable.
- The `data-theme` attribute drives the CSS token mechanism built in task 01. `data-text-size` drives the root font-size scaling.
