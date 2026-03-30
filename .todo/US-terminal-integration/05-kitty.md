---
status: pending
---

# kitty focus strategy

## Goal

Add kitty window focus using kitty's remote control protocol.

## API research

### Remote control commands

kitty exposes a JSON-based remote control protocol:

| Command | Purpose | Notes |
|---------|---------|-------|
| `kitty @ ls` | List all OS windows, tabs, and windows (panes) | Returns JSON |
| `kitty @ focus-window --match pid:<pid>` | Focus window by PID | Exact PID match |
| `kitty @ focus-window --match cwd:<regex>` | Focus window by CWD | Regex match |
| `kitty @ focus-tab --match <matcher>` | Focus a tab | Various matchers |

### `kitty @ ls` JSON structure

```json
[
  {
    "id": 1,
    "platform_window_id": 123456,
    "is_active": true,
    "is_focused": true,
    "tabs": [
      {
        "id": 7,
        "title": "bash",
        "is_active": true,
        "windows": [
          {
            "id": 42,
            "title": "bash",
            "pid": 98765,
            "cwd": "/Users/me/project",
            "cmdline": ["bash"],
            "is_self": false,
            "foreground_processes": [...]
          }
        ]
      }
    ]
  }
]
```

### Key fields per window (pane)

| Field | Type | Notes |
|-------|------|-------|
| `pid` | number | Shell process PID |
| `cwd` | string | Current working directory |
| `cmdline` | array | Command line arguments |
| `title` | string | Window title |
| `id` | number | Window ID |
| `foreground_processes` | array | Currently running foreground processes |

### Match selectors

The `--match` parameter supports:
- `pid:<pid>` — exact PID match
- `cwd:<regex>` — regex match on CWD
- `cmdline:<regex>` — regex match on command line
- `title:<regex>` — regex match on title
- `id:<id>` — window ID

### Focus behavior

`focus-window` with `switch_os_window_if_needed=True` (default) will:
1. Switch to the correct OS window (if multiple kitty windows)
2. Switch to the correct tab
3. Focus the matching pane

This brings kitty to the foreground on the desktop.

### Configuration required

Remote control must be explicitly enabled:

```conf
# kitty.conf
allow_remote_control yes
# or more restrictive:
allow_remote_control socket-only
listen_on unix:/tmp/mykitty
```

- **Not enabled**: `kitty @ ls` will fail with a permission error.
- **From inside kitty**: kitten mappings work without `allow_remote_control` (they use the internal protocol).

### What happens when not enabled

When remote control is not enabled and called externally:
- kitty refuses the request with an error.
- Our strategy should return `NotApplicable`, not `Failed`.

### Compatibility

- Remote control has been available since kitty 0.14+ (2019).
- `focus-window` is stable across all modern versions.
- No macOS-specific limitations.
- Bundle ID: `net.kovidgoyal.kitty`

## How it works

1. Check bundle ID is `net.kovidgoyal.kitty`.
2. Run `kitty @ ls` — if it fails (remote control not enabled), return `NotApplicable`.
3. Parse JSON, match window by PID (copilot PID or ancestor).
4. Run `kitty @ focus-window --match pid:<matched_pid>`.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminals/kitty.rs`
- [ ] Matches by PID (precise, using copilot PID or ancestor)
- [ ] Handles remote control not enabled gracefully (`NotApplicable`)
- [ ] Handles `kitty` not installed gracefully (`NotApplicable`)
- [ ] Unit tests: JSON parsing, PID matching, error handling
- [ ] `just check` passes

## Notes

- kitty's PID matching is the most precise of all non-tmux strategies — it matches the actual shell process PID.
- `foreground_processes` in the JSON could also be used for matching, but `pid` is simpler.
- The `allow_remote_control` requirement is the main adoption barrier — users must opt in.
