---
status: done
---

# Focus strategy: per-terminal AppleScript

## Goal

Implement the terminal-specific AppleScript focus strategy. When the terminal app supports scripting, find and focus the specific window/tab/pane containing the copilot session.

## Supported terminals

| Terminal     | Match by | Focus method | Notes |
|-------------|----------|--------------|-------|
| Terminal.app | TTY via `tty of tab` | `set index of tab to 1` + `activate` | TTY available from copilot process |
| iTerm2       | TTY via `tty of session` | `select` tab + `activate` | Also supports PID matching |
| Ghostty 1.3+ | Working directory via `working directory of terminal` | `focus` terminal | No TTY/PID exposed yet (issue #11592). Match by CWD from workspace.yaml. Partial precision. |

## How it works

1. Check `FocusContext.terminal_app_bundle` against known scriptable terminals.
2. If not a known terminal, return `NotApplicable`.
3. Get the copilot process's TTY via `ps -p <pid> -o tty=`.
4. Build the appropriate AppleScript for the detected terminal.
5. Run via `Command::new("osascript").args(["-e", &script])`.
6. If the script finds and focuses the window, return `Focused`.
7. If no matching window found, return `Failed` (fallback to next strategy).

### Example: Terminal.app

```applescript
tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            if (tty of t is "/dev/ttysNNN") then
                set selected tab of w to t
                set frontmost of w to true
                activate
                return
            end if
        end repeat
    end repeat
end tell
```

### Example: iTerm2

```applescript
tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if (tty of s is "/dev/ttysNNN") then
                    select w
                    select t
                    activate
                    return
                end if
            end repeat
        end repeat
    end repeat
end tell
```

### Example: Ghostty (CWD fallback)

```applescript
tell application "Ghostty"
    set matches to every terminal whose working directory contains "project-dir"
    if (count of matches) > 0 then
        focus (item 1 of matches)
    end if
end tell
```

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminal_script.rs` with `try_focus(ctx: &FocusContext) -> FocusResult`
- [ ] Supports Terminal.app, iTerm2, Ghostty 1.3+
- [ ] Falls back gracefully if terminal app is not recognized (`NotApplicable`)
- [ ] Falls back if AppleScript can't find matching window (`Failed`)
- [ ] Unit tests: script generation for each terminal, bundle ID matching
- [ ] `just check` passes

## Notes

- Ghostty AppleScript was introduced in 1.3.0. Ghostty <1.3 will return `NotApplicable` (the `tell application` command will work but there are no scriptable objects).
- Ghostty doesn't expose `tty` or `pid` properties yet (issue #11592). CWD matching from `workspace.yaml` is the best available fallback.
- The `FocusContext` should carry the copilot process TTY (derived from the PID) for use by this strategy. This may require extending `FocusContext` or `pid_ancestry.rs`.

## Relevant files

- `src-tauri/src/terminal_focus/terminal_script.rs` — new file
- `src-tauri/src/terminal_focus/pid_ancestry.rs` — may need to add TTY lookup
