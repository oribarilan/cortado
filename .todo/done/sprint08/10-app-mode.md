---
status: pending
---

# 10 — App Mode & Activation Behavior

## Goal

Make the menubar (tray icon + menubar panel) optional via a setting. Ensure the app is always reachable: launching/reactivating the app opens the panel, and settings are accessible from the panel.

## Acceptance Criteria

- [ ] New setting in app settings: `general.show_menubar` (boolean, default: `true`)
- [ ] When `general.show_menubar` is `false`: tray icon is hidden, menubar panel is not created/shown
- [ ] When `general.show_menubar` is `true`: existing tray + menubar panel behavior unchanged
- [ ] Setting is toggleable from the Settings UI (General section)
- [ ] Toggling takes effect immediately (no app restart required)
- [ ] When the app is launched (or re-opened via double-click / Spotlight / `open -a Cortado`), the panel opens
  - Handle macOS `applicationShouldHandleReopen` / `NSApplicationDelegate` reopen event
  - This works regardless of `general.show_menubar` setting
- [ ] Panel footer (or header) includes a "Settings" action that opens the settings window
- [ ] When menubar is off, the global hotkey (⌘+Shift+Space) is the primary way to access the app — ensure it's always registered
- [ ] `ActivationPolicy::Accessory` — verify that macOS reopen events (`RunEvent::Reopen` or equivalent) still fire. Accessory apps don't show in the Dock but should still receive reopen when launched from Spotlight/Finder. Test this early.

### Backend (settings schema)

- [ ] Add `show_menubar` field to `AppSettings` struct in `app_settings.rs` with `#[serde(default)]` defaulting to `true`, so existing `settings.toml` files are handled gracefully
- [ ] Add Tauri command to read/write `show_menubar` (or reuse a generic settings update command if one exists)

## Notes

### Reopen event (researched)

- **`RunEvent::Reopen { has_visible_windows: bool }` exists in Tauri v2** (confirmed in `tauri 2.10.3`, which is the resolved version in `Cargo.lock`).
- It maps from macOS's `applicationShouldHandleReopen` via `tauri-runtime-wry`.
- Tauri does NOT special-case it away for `ActivationPolicy::Accessory` — the event should still fire when the app is re-launched from Spotlight/Finder/`open -a` while already running.
- **The app does NOT currently handle any `RunEvent` variants** — `main.rs` calls `.run(tauri::generate_context!())` with no event callback. Task 10 needs to switch to `.run(|_app, event| { match event { ... } })` to handle `Reopen`.
- Since Accessory apps have no Dock icon, the primary reopen trigger is Spotlight / Finder / `open -a`. This should be tested early in the task.

### Other notes

- Tray icon removal at runtime: verify `tray.set_visible(false)` or equivalent works without crashing. Alternatively, just don't create the tray on startup when the setting is off.
- Consider the first-launch experience: if menubar is on by default, the user has both. They can then disable the menubar from settings if they prefer hotkey-only.
