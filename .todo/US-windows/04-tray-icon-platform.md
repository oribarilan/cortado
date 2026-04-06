---
status: pending
---

# Task: Platform-aware tray icon

## Goal

Make the tray icon rendering work correctly on both macOS and Windows, accounting for platform differences in template images, dark mode detection, and icon compositing.

## Acceptance criteria

- [ ] Tray icon uses macOS template mode on macOS (unchanged behavior)
- [ ] Tray icon uses appropriate rendering on Windows (no template mode -- Windows uses colored icons directly)
- [ ] Dark mode detection for icon tinting uses the cross-platform `is_dark_mode()` from task 03
- [ ] Status dot compositing works on both platforms (RGBA pixel manipulation is platform-independent, but the menubar ring color logic may differ)
- [ ] Windows tray icon uses `.ico` format (already present in `icons/icon.ico` -- verify sizes are correct: 16x16, 32x32, 48x48)
- [ ] Tray click behavior: macOS positions menubar panel below icon via `monitor::get_monitor_with_cursor()` (unchanged); Windows shows panel near system tray or shows main screen panel (depends on task 02 decision)
- [ ] `icon_as_template(true/false)` calls are macOS-gated (`#[cfg(target_os = "macos")]`)
- [ ] `panel.rs` tray click handler uses platform-dispatched panel toggle from task 02 (no direct `fns::toggle_menubar_panel()` call)
- [ ] Both targets compile cleanly

## Notes

- Windows system tray icons are typically 16x16 or 32x32. The existing `icon.ico` may need size verification.
- Windows has no equivalent of macOS "template mode" -- icons are displayed as-is. The tray icon needs to be pre-tinted for the current theme (or use a theme-neutral icon).
- The menubar ring color compositing (drawing a ring in the menubar background color around the status dot) is a macOS visual detail. On Windows, a simpler colored dot on a transparent background may suffice.
- `panel.rs` currently calls `monitor::get_monitor_with_cursor()` (macOS-only crate from tauri-toolkit) in the tray click handler. On Windows, use Tauri's built-in monitor APIs or just center the panel.

## Dependencies

- Task 02 (platform window management -- tray click calls panel toggle)
- Task 03 (platform shell commands -- dark mode detection)

## Related files

- `src-tauri/src/tray_icon.rs` (full file -- template mode, compositing, dark mode detection)
- `src-tauri/src/panel.rs` (tray creation, click handler, `icon_as_template`, `monitor::get_monitor_with_cursor()`)
- `src-tauri/icons/` (icon assets)
