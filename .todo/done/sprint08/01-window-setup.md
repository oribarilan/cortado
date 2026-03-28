---
status: pending
---

# 01 — Window Setup & Global Hotkey

## Goal

Create the floating panel infrastructure: a new Tauri window with its own HTML entrypoint, converted to an NSPanel at runtime, toggled via ⌘+Shift+Space, centered on the active monitor.

## Acceptance Criteria

- [ ] New HTML entrypoint `main-screen.html` with a React root (empty placeholder UI is fine)
- [ ] New Tauri window `main-screen` configured in `tauri.conf.json`: hidden, transparent, undecorated, ~560×440
- [ ] Window converted to NSPanel on first show (reuse patterns from `fns.rs` / `tauri-nspanel`)
- [ ] Panel is floating, non-activating, moves to active space, no Dock icon
- [ ] Global shortcut ⌘+Shift+Space registered via `tauri-plugin-global-shortcut`
- [ ] Shortcut toggles: if hidden → center on active monitor + show; if visible → hide
- [ ] Panel hides when it resigns key (loses focus)
- [ ] Panel hides on Esc (frontend keydown handler)
- [ ] Coexists with existing menubar panel — both can be used independently
- [ ] Two-NSPanel coexistence verified: resign-key delegates are scoped to their own window label — showing/hiding the panel does not interfere with the menubar panel's visibility, and vice versa

## Notes

- Reference implementation: `ahkohd/tauri-macos-spotlight-example` (already explored)
- The existing NSPanel conversion logic lives in `fns.rs` (NOT `panel.rs` — that file handles tray icon/menu wiring). The `fns.rs` functions are hardcoded to the `"main"` window label. Create a parallel `main_screen.rs` module with its own NSPanel setup, centered positioning (no popover arrow), and lifecycle.
- Use `monitor::get_monitor_with_cursor()` for centering (the spotlight example does this)
- `tauri-plugin-global-shortcut` is NOT currently a dependency — must be added to `Cargo.toml` and registered in the plugin builder chain in `main.rs`
