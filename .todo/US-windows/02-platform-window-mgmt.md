---
status: pending
---

# Task: Platform window management

## Goal

Gate the macOS NSPanel window management behind `cfg(target_os = "macos")` and provide Windows-native window management so panels (menubar popup + main screen) work on both platforms.

**This is the highest-risk task in the story.** The entire UX identity of Cortado is the floating panel behavior. Budget for iteration — this will likely take 2-3x longer than other tasks.

## Architecture

Use cfg-dispatched module pairs, **not** a trait abstraction (there are exactly two platforms, they share no logic, a trait adds indirection nobody benefits from):

```
src-tauri/src/
  fns.rs                  → rename to panel_macos.rs, gate with cfg(macos)
  panel_windows.rs        → new, cfg(windows), Tauri WebviewWindow-based
  panel.rs                → thin dispatcher: each pub fn is a #[cfg] picking the right module

  main_screen.rs          → rename to main_screen_macos.rs, gate with cfg(macos)
  main_screen_windows.rs  → new, cfg(windows)
  main_screen.rs          → thin dispatcher
```

## Acceptance criteria

- [ ] `fns.rs` renamed to `panel_macos.rs` and gated behind `#[cfg(target_os = "macos")]` — zero changes to its contents
- [ ] `panel_windows.rs` created: uses standard Tauri `WebviewWindow` APIs (`set_always_on_top(true)`, `set_decorations(false)`, `set_skip_taskbar(true)`, show/hide/center)
- [ ] `panel.rs` is a thin dispatcher — each public function delegates to the correct platform module via `#[cfg]`
- [ ] `main_screen.rs` follows the same pattern (macOS/Windows/dispatcher)
- [ ] `command.rs` Tauri commands (`init_panel`, `init_main_screen_panel`, `hide_menubar_panel`, `hide_all_panels`) call through dispatchers, no direct macOS imports
- [ ] `tauri_nspanel::ManagerExt` usage (~8 call sites in command.rs, fns.rs, main_screen.rs) is macOS-gated. Windows uses `app.get_webview_window("main")` instead.
- [ ] Windows vibrancy: use Tauri v2's `WebviewWindow::set_effects()` for Mica/Acrylic on Windows 11, solid background fallback for Windows 10
- [ ] Panel auto-hide on focus loss: macOS uses NSWorkspace notifications (unchanged); Windows uses Tauri's `WindowEvent::Focused(false)` event
- [ ] macOS behavior is byte-for-byte identical to current implementation
- [ ] `transparent: true` in `tauri.conf.json` for `main` and `main-screen` windows: verify WebView2 handles this correctly on Windows (WebView2 transparency requires specific setup — Tauri v2 should handle it internally, but test explicitly)
- [ ] Both targets compile cleanly

## Notes

### Platform differences to accept (not fight)

- **Menubar panel positioning**: macOS positions below tray icon with popover arrow. On Windows, anchor near the system tray area or center on screen. Don't try to replicate the NSPanel popover — it'll feel wrong on Windows.
- **PopoverView** (arrow/border chrome) is macOS-only. Windows panel uses a simple bordered rectangle with shadow.
- **Non-activating panel**: macOS NSPanels don't steal focus from other apps. On Windows, `always_on_top` windows DO steal focus. Mitigate with `set_focus()` management but accept this is a platform difference.
- **NSWorkspace notifications** (`NSWorkspaceDidActivateApplicationNotification`, `NSWorkspaceActiveSpaceDidChangeNotification`): no Windows equivalent. Use `WindowEvent::Focused(false)` for auto-hide.
- **`panel_delegate!(SpotlightPanelDelegate)` with `window_did_resign_key`**: Cocoa concept. On Windows, `WindowEvent::Focused(false)` serves the same purpose.
- **Tauri v2 `set_effects()` API**: supports `Effect::Mica`, `Effect::Acrylic`, `Effect::Tabbed` on Windows 11. Use this instead of the `window-vibrancy` crate.

### Architectural decision

Consider whether Windows should even have a separate menubar popup panel, or whether the main screen panel alone is sufficient. The menubar popup is a macOS idiom (click tray icon → attached popover). On Windows, tray apps typically show a single main window. It may be simpler and more native to just show the main screen panel on tray click on Windows.

## Related files

- `src-tauri/src/fns.rs` (menubar panel — full file is macOS, ~305 lines)
- `src-tauri/src/main_screen.rs` (main screen panel — full file is macOS, ~241 lines)
- `src-tauri/src/command.rs` (Tauri commands calling into panel code, `tauri_nspanel::ManagerExt`)
- `src-tauri/src/panel.rs` (tray icon creation + click → panel toggle, uses `monitor::get_monitor_with_cursor()`)
