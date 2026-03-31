---
status: done
---

# iTerm2 focus strategy

## Goal

Add iTerm2 tab/session focus using AppleScript TTY matching.

## API research

### AppleScript objects

iTerm2 exposes a comprehensive AppleScript dictionary (`iTerm2.sdef`):

- `application` > `windows` > `tabs` > `sessions`
- Split panes are **sibling sessions within the same tab**
- Each `session` has a `tty` property (read-only, returns e.g. `/dev/ttys003`)
- All objects support `select` to focus them

### Key properties

| Object | Property | Type | Notes |
|--------|----------|------|-------|
| `session` | `tty` | text (read-only) | Full TTY device path |
| `session` | `unique ID` | text (read-only) | Stable session identifier |
| `session` | `name` | text | Session name/title |
| `session` | `is at shell prompt` | boolean | Requires shell integration |
| `tab` | `current session` | session | The active pane in this tab |
| `window` | `current tab` | tab | The active tab |

### Commands available

- `select` on window, tab, and session — focuses that object
- `split vertically` / `split horizontally` on session — creates split panes
- `write text` on session — sends text input

### Focus script

```applescript
set targetTTY to "/dev/ttys003"

tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if (tty of s) is targetTTY then
                    tell w to select
                    tell t to select
                    tell s to select
                    activate
                    return
                end if
            end repeat
        end repeat
    end repeat
end tell
```

### Focus sequence

The correct order for reliable focus:
1. `select` the window (brings it forward)
2. `select` the tab (switches to it)
3. `select` the session (activates the pane if split)
4. `activate` (brings iTerm2 to front)

### Version notes

- Window `id` changed in **3.0.4**: was string like `"window-1"`, now integer. Old form available as `alternate identifier`.
- `tty of session` has been stable across all modern versions.
- Python API also available but AppleScript is simpler for our use case.

### Compatibility

- Works on iTerm2 3.0+ (all modern versions).
- No configuration required.
- Bundle ID: `com.googlecode.iterm2`

## How it works

1. Check bundle ID is `com.googlecode.iterm2`.
2. If tmux is in the process ancestry, return `NotApplicable` (TTY won't match).
3. Get copilot process TTY via `ps -p <pid> -o tty=`.
4. AppleScript: enumerate windows/tabs/sessions, match `tty of session` against `/dev/<tty>`.
5. `select` window, tab, session in order. `activate`.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminals/iterm2.rs`
- [ ] Matches by TTY (precise)
- [ ] Returns `NotApplicable` when tmux is in use
- [ ] Returns `NotApplicable` when bundle ID doesn't match
- [ ] Unit tests: TTY matching, bundle ID check
- [ ] `just check` passes

## Notes

- iTerm2's session model naturally handles split panes — `select` on a session activates the correct pane.
- TTY matching only works without tmux. With tmux, the copilot TTY is a tmux PTY.
- iTerm2 also has a Python API, but AppleScript is sufficient and requires no setup.
- **App name caveat**: the macOS app may be installed as `iTerm.app` (not `iTerm2.app`). The AppleScript application name is `"iTerm"` or `"iTerm2"` depending on version — test both. The E2E test in `e2e.rs` already handles this with a fallback check.
