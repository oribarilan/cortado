---
status: pending
---

# Focus terminal -- Windows support

## Goal

Port the macOS "focus terminal" action (sprint 13, task 03) to Windows.

## Context

On macOS, cortado can bring the owning terminal window to the foreground by walking the process ancestry to find the GUI app, then activating it via `NSRunningApplication`. On Windows, the equivalent would use the Win32 API.

## Approach (to be refined)

### Process ancestry

Use `CreateToolhelp32Snapshot` + `Process32First`/`Process32Next` to walk the process tree, or `NtQueryInformationProcess` for parent PID.

### Window focus

Use `SetForegroundWindow` + `EnumWindows` to find the window belonging to the terminal process. Or `AllowSetForegroundWindow` + `BringWindowToTop`.

Common Windows terminals to handle:
- Windows Terminal (`WindowsTerminal.exe`)
- PowerShell / cmd.exe (conhost)
- VS Code integrated terminal
- Git Bash (mintty)

### tmux equivalent

Windows users are less likely to use tmux. If they do (via WSL), the approach would differ. For the MVP, skip multiplexer support on Windows.

## Notes

- This is a follow-up to the macOS implementation. The Tauri command (`focus_session`) should already exist with a platform gate.
- Consider using the `windows` crate for Win32 API bindings.
