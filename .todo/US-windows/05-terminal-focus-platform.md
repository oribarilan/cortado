---
status: pending
---

# Task: Platform-conditional terminal focus

## Goal

Gate the terminal focus system behind `#[cfg(target_os = "macos")]` and provide a Windows stub that gracefully reports "not supported on this platform."

## Acceptance criteria

- [ ] The entire `terminal_focus/` module declaration in `main.rs` is gated behind `#[cfg(target_os = "macos")]`
- [ ] A minimal `terminal_focus` stub for Windows exists (either inline in `command.rs` or as a thin module) that returns "Terminal focus is not supported on Windows"
- [ ] The Tauri command `focus_terminal` compiles on both platforms -- on Windows it returns an error/unsupported status
- [ ] `AXIsProcessTrusted()` FFI (`#[link(name = "ApplicationServices")]`) is inside the macOS-gated module (no separate gating needed)
- [ ] The `FocusCaps` type returned to the frontend includes a `platform_supported: bool` field so the UI can conditionally render the terminal focus sections
- [ ] `check_focus_caps` Tauri command: on macOS returns full capabilities (unchanged); on Windows returns `platform_supported: false` with all other fields zeroed/default
- [ ] macOS terminal focus behavior is completely unchanged
- [ ] Both targets compile cleanly

## Notes

- Full Windows terminal focus (Windows Terminal, PowerShell, cmd.exe via `SetForegroundWindow`) could be a future enhancement. For now, graceful degradation is sufficient.
- The `ps`-based PID ancestry walking (`pid_ancestry.rs`) uses POSIX `ps` -- this is inside the macOS-gated module, no separate handling needed.
- The `check_accessibility_permission` function uses `ApplicationServices.framework` FFI -- also inside the macOS-gated module.
- Since the entire module is cfg-gated, the `FocusCaps` struct definition itself should remain cross-platform (it's a data type the frontend needs), but the logic that populates it is platform-conditional.

## Related files

- `src-tauri/src/terminal_focus/` (entire directory -- ~1,400 lines)
- `src-tauri/src/command.rs` (commands: `focus_terminal`, `check_focus_caps`)
- `src/settings/SettingsApp.tsx` (Ghostty and Accessibility settings sections -- task 07 handles frontend)
