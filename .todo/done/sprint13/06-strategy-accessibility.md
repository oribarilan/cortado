---
status: done
---

# Focus strategy: Accessibility API

## Goal

Implement the Accessibility API focus strategy. When the user has granted Accessibility permission, use `AXUIElement` to find and raise the specific terminal window containing the copilot session.

## How it works

1. Check if Accessibility permission is granted (`AXIsProcessTrusted()`).
2. If not granted, return `NotApplicable`.
3. Create `AXUIElementCreateApplication(terminal_app_pid)`.
4. Get `kAXWindowsAttribute` -- list of windows.
5. For each window, get `kAXTitleAttribute`.
6. Match window title against known patterns (CWD path, repo name, session summary, etc.).
7. Call `AXUIElementPerformAction(window, kAXRaiseAction)` on the matching window.
8. Activate the app with `NSRunningApplication.activate()`.
9. Return `Focused`.

If no matching window is found, return `Failed`.

## Permission model

Accessibility permission is granted per-app in System Settings > Privacy & Security > Accessibility. The user must add cortado to the list.

The settings UI (task 07) will:
- Show whether Accessibility permission is currently granted.
- Explain what it enables ("Focus the exact terminal window, not just the app").
- Link to System Settings to grant permission.

This strategy should **never prompt** for permission on its own. It checks `AXIsProcessTrusted()` and returns `NotApplicable` if not granted.

## Window title matching

Terminal window titles typically contain:
- The current directory (e.g., `~/repos/personal/cortado`)
- The running command (e.g., `copilot`)
- Custom shell prompt info

The matching algorithm:
1. Try exact match on CWD from `workspace.yaml`
2. Try substring match on repo name (e.g., `cortado`)
3. Try substring match on `copilot` (the running command)
4. If multiple matches, prefer the one with the most specific match

This is inherently heuristic -- window titles vary by terminal config and shell prompt.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/accessibility.rs` with `try_focus(ctx: &FocusContext) -> FocusResult`
- [ ] Checks `AXIsProcessTrusted()` -- returns `NotApplicable` if not granted
- [ ] Enumerates windows of terminal app via `AXUIElement`
- [ ] Matches window by title (CWD, repo name, or command)
- [ ] Raises the matched window with `kAXRaiseAction`
- [ ] Never prompts for permission -- only checks
- [ ] Unit tests: title matching logic
- [ ] `just check` passes

## Notes

- The `AXUIElement` API requires `ApplicationServices` framework. Check if this is already linked by Tauri/AppKit.
- `AXIsProcessTrusted()` is in `ApplicationServices/HIServices`.
- This strategy is lower precision than tmux or terminal scripting, but works across any terminal app without per-app scripting.
- On macOS Sonoma+, the first call to an AX API may trigger a system permission dialog. Since we check `AXIsProcessTrusted()` first, this shouldn't happen -- but test carefully.

## Relevant files

- `src-tauri/src/terminal_focus/accessibility.rs` -- new file
