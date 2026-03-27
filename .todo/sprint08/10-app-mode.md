---
status: pending
---

# 10 — App Mode & Activation Behavior

## Goal

Make the menubar (tray icon + menubar panel) optional via a setting. Ensure the app is always reachable: launching/reactivating the app opens the main screen, and settings are accessible from the main screen.

## Acceptance Criteria

- [ ] New setting in app settings: `show_menubar` (boolean, default: `true`)
- [ ] When `show_menubar` is `false`: tray icon is hidden, menubar panel is not created/shown
- [ ] When `show_menubar` is `true`: existing tray + menubar panel behavior unchanged
- [ ] Setting is toggleable from the Settings UI (General section)
- [ ] Toggling takes effect immediately (no app restart required)
- [ ] When the app is launched (or re-opened via double-click / Spotlight / `open -a Cortado`), the main screen opens
  - Handle macOS `applicationShouldHandleReopen` / `NSApplicationDelegate` reopen event
  - This works regardless of `show_menubar` setting
- [ ] Main screen footer (or header) includes a "Settings" action that opens the settings window
- [ ] When menubar is off, the global hotkey (⌘+Shift+Space) is the primary way to access the app — ensure it's always registered
- [ ] `ActivationPolicy::Accessory` — verify that macOS reopen events (`RunEvent::Reopen` or equivalent) still fire. Accessory apps don't show in the Dock but should still receive reopen when launched from Spotlight/Finder. Test this early.

### Backend (settings schema)

- [ ] Add `show_menubar` field to `AppSettings` struct in `app_settings.rs` with `#[serde(default)]` defaulting to `true`, so existing `settings.toml` files are handled gracefully
- [ ] Add Tauri command to read/write `show_menubar` (or reuse a generic settings update command if one exists)

## Notes

- The app currently uses `ActivationPolicy::Accessory` — this may need to stay as-is even when menubar is off, since the main screen is still an overlay (not a Dock app). Test that reopen events still fire with Accessory policy.
- The `reopen` event in Tauri: check if `tauri::RunEvent::Reopen` exists in Tauri v2, or if we need to use a macOS-specific hook.
- Tray icon removal at runtime: verify `tray.set_visible(false)` or equivalent works without crashing. Alternatively, just don't create the tray on startup when the setting is off.
- Consider the first-launch experience: if menubar is on by default, the user has both. They can then disable the menubar from settings if they prefer hotkey-only.
