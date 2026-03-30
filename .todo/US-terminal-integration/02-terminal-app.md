---
status: pending
---

# Terminal.app focus strategy

## Goal

Add macOS Terminal.app tab focus using AppleScript TTY matching.

## API research

### AppleScript objects

Terminal.app exposes a mature AppleScript dictionary:

- `application` > `windows` > `tabs`
- Each `tab` has a `tty` property (read-only, returns e.g. `/dev/ttys003`)
- `selected tab of window` — read/write, switches tabs
- `frontmost` — brings window to front
- `activate` — brings Terminal.app to foreground

### Key properties

| Object | Property | Type | Notes |
|--------|----------|------|-------|
| `tab` | `tty` | text (read-only) | Full TTY device path, e.g. `/dev/ttys003` |
| `tab` | `selected` | boolean | Whether this tab is the active tab |
| `window` | `selected tab` | tab (read/write) | Set to switch tabs |
| `window` | `frontmost` | boolean (read/write) | Bring window to front |

### Focus script

```applescript
tell application "Terminal"
    set ttyTarget to "/dev/ttys003"
    repeat with aWindow in windows
        repeat with aTab in tabs of aWindow
            if tty of aTab is ttyTarget then
                set selected tab of aWindow to aTab
                set frontmost to true
                activate
                return
            end if
        end repeat
    end repeat
end tell
```

### TTY resolution

Get copilot process TTY via `ps -p <pid> -o tty=` → returns e.g. `ttys003`. Prepend `/dev/` for the full device path.

**Important**: when tmux is in use, the copilot process TTY is a tmux PTY (e.g., `/dev/ttys107`), which is *not* the terminal tab's TTY. TTY matching only works without tmux — the tmux strategy handles the tmux case.

### Compatibility

- Works on all macOS versions with Terminal.app (essentially all of them).
- No version restrictions — `tty of tab` has been available for many years.
- No configuration required.
- Bundle ID: `com.apple.Terminal`

## How it works

1. Check bundle ID is `com.apple.Terminal`.
2. If tmux is in the process ancestry, return `NotApplicable` (TTY won't match).
3. Get copilot process TTY via `ps -p <pid> -o tty=`.
4. AppleScript: enumerate windows/tabs, match `tty of tab` against `/dev/<tty>`.
5. Set `selected tab`, set `frontmost`, `activate`.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminals/terminal_app.rs`
- [ ] Matches by TTY (precise, no heuristics)
- [ ] Returns `NotApplicable` when bundle ID doesn't match
- [ ] Returns `NotApplicable` when tmux is in use (TTY won't match — tmux strategy handles this)
- [ ] Unit tests: TTY path construction, bundle ID check
- [ ] `just check` passes

## Notes

- Terminal.app has the most mature and reliable AppleScript API on macOS.
- TTY matching is precise — no ambiguity, no heuristics.
- The only limitation is tmux (different PTY namespace).
