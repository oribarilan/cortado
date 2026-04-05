---
status: pending
---

# Task: Autostart, activation policy, and app lifecycle

## Goal

Make app startup behavior (no dock icon / no taskbar button, autostart/launch-at-login, single instance, app reopen, close-to-tray) work correctly on Windows.

## Acceptance criteria

- [ ] `ActivationPolicy::Accessory` (no Dock icon) is macOS-gated (`#[cfg(target_os = "macos")]`). This is a macOS-only Tauri API — it won't compile on Windows without gating.
- [ ] Windows tray-only behavior: when all windows are hidden, no taskbar button appears (use `set_skip_taskbar(true)` on windows or hide them fully)
- [ ] `MacosLauncher::LaunchAgent` in autostart plugin init is macOS-gated; verify `tauri-plugin-autostart` auto-selects Windows registry strategy (or explicitly configure it)
- [ ] Single instance (`tauri-plugin-single-instance`): works on both platforms. On Windows, the second-instance handler should show/focus the existing panel (currently the handler is empty — it should open the main screen panel)
- [ ] `RunEvent::Reopen` handling (`main.rs:214`): gated behind `#[cfg(target_os = "macos")]` (this event is macOS-only — fires on dock icon click). Windows equivalent behavior is handled by single-instance plugin.
- [ ] App reopen behavior unified: macOS uses `Reopen` event, Windows uses single-instance callback — both show the main screen panel
- [ ] Global hotkey (`tauri-plugin-global-shortcut`): verify it works on Windows (uses `RegisterHotKey` Win32 API — should work out of the box)
- [ ] Windows close-to-tray: intercept `close_requested` event on Windows so the close button (`X`) hides the window instead of quitting. This matches macOS behavior where closing a panel doesn't quit the app. macOS doesn't need this because NSPanels don't have close buttons.
- [ ] Both targets compile cleanly

## Notes

- `ActivationPolicy::Accessory` is behind `#[cfg(target_os = "macos")]` in Tauri's own API — attempting to use it on Windows is a compile error, not just a no-op.
- The single-instance plugin works differently per platform: macOS uses bundle ID, Windows uses a named mutex. Current handler is empty (`|_app, _argv, _cwd| {}`) — on Windows this should call `show_main_screen_panel()` or equivalent.
- `tauri-plugin-autostart` v2: the `MacosLauncher` enum is macOS-specific. On Windows, the plugin uses the Registry `Run` key automatically. The init call may need to be platform-conditional:
  ```rust
  #[cfg(target_os = "macos")]
  let autostart = tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None);
  #[cfg(target_os = "windows")]
  let autostart = tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None); // LaunchAgent ignored on Windows
  ```
  Verify whether the `MacosLauncher` parameter is simply ignored on non-macOS or causes an error.
- Global shortcut plugin uses platform-native APIs (macOS: Carbon hotkeys; Windows: `RegisterHotKey`). Cross-platform by design.

## Dependencies

- Task 01 (Cargo deps — plugins must be available)
- Task 02 (platform window management — for panel show/toggle in single-instance handler)

## Related files

- `src-tauri/src/main.rs:88-90` (single-instance handler — currently empty)
- `src-tauri/src/main.rs:92-95` (autostart init with `MacosLauncher`)
- `src-tauri/src/main.rs:148` (`set_activation_policy`)
- `src-tauri/src/main.rs:156-177` (global shortcut registration)
- `src-tauri/src/main.rs:214` (`RunEvent::Reopen` handler)
