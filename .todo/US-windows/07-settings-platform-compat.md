---
status: pending
---

# Task: Settings UI platform compatibility

## Goal

Make the Settings window show platform-appropriate options: hide macOS-only settings on Windows, show Windows-specific settings where applicable, and ensure all system integration links point to the correct OS settings.

## Acceptance criteria

- [ ] Ghostty tab switching section is hidden on Windows (Ghostty's AppleScript API is macOS-only). Handled naturally by the new modular Terminals tab design (see `US-windows-prereq/01-terminals-settings-tab`).
- [ ] Accessibility permission section is hidden on Windows (macOS `AXIsProcessTrusted` concept doesn't apply on Windows). Handled naturally by the new modular Terminals tab design.
- [ ] "Configure in System Settings" notification link: macOS opens `x-apple.systempreferences:` (unchanged); Windows opens `ms-settings:notifications`
- [ ] Accessibility settings link: macOS opens `x-apple.systempreferences:...Privacy_Accessibility` (unchanged); Windows omits the link entirely (Windows doesn't gate automation on accessibility permissions)
- [ ] "Launch at Login" toggle works on both platforms (backend handles platform-specific autostart -- `MacosLauncher::LaunchAgent` on macOS, registry-based on Windows via `tauri-plugin-autostart`)
- [ ] `open_settings_file` / `reveal_settings_file` commands work on Windows (covered by task 03 abstraction)
- [ ] `open_config_file` / `reveal_config_file` commands work on Windows (covered by task 03 abstraction)
- [ ] Config file path shown in Settings UI uses backend-provided path (not hardcoded `~/.config/cortado/`)
- [ ] `FocusCaps` response from backend includes `platform_supported` field (from task 05) for conditional rendering
- [ ] Both platforms build cleanly

## Notes

- `tauri-plugin-autostart` supports Windows via registry run keys. The `MacosLauncher::LaunchAgent` variant is macOS-specific -- the plugin auto-selects the correct strategy per platform. Verify this in the plugin docs/source.
- Some settings sections could show a "Not available on Windows" note rather than being completely hidden. Ask user for preference if unclear.
- The Settings window itself uses standard Tauri decorations (`decorations: true`) which work on both platforms -- no platform-specific window code needed.
- The platform detection utility from task 06 (`isMacOS()`) is used here for conditional rendering.

## Dependencies

- Task 03 (platform shell commands -- for file open/reveal)
- Task 05 (terminal focus platform -- for `FocusCaps.platform_supported` field)
- Task 06 (frontend platform compat -- for `isMacOS()`/`isWindows()` utility)
- **`US-windows-prereq/01-terminals-settings-tab`** (prerequisite -- the Terminals tab redesign must land first)

## Related files

- `src/settings/SettingsApp.tsx` (full file -- multiple platform-specific sections)
- `src-tauri/src/command.rs` (`open_notification_settings`, `check_focus_caps`)
- `src-tauri/src/app_settings.rs` (`open_settings_file`, `reveal_settings_file`)
- `src-tauri/src/settings_config.rs` (`open_config_file`, `reveal_config_file`)
