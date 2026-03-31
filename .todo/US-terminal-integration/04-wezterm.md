---
status: done
---

# WezTerm focus strategy

## Goal

Add WezTerm pane focus using the `wezterm` CLI.

## API research

### CLI commands

WezTerm ships a powerful CLI that communicates with the running GUI instance:

| Command | Purpose | Since |
|---------|---------|-------|
| `wezterm cli list` | List all panes with metadata | 20220624 |
| `wezterm cli list --format json` | JSON output | 20220624 |
| `wezterm cli activate-pane --pane-id <id>` | Focus a specific pane | 20230326 |
| `wezterm cli activate-tab --tab-id <id>` | Focus a specific tab | — |
| `wezterm cli list-clients --format json` | List connected clients | — |

### `wezterm cli list --format json` fields

Each pane object contains:

| Field | Type | Notes |
|-------|------|-------|
| `window_id` | number | OS window ID |
| `tab_id` | number | Tab ID |
| `pane_id` | number | Pane ID (used for `activate-pane`) |
| `workspace` | string | Workspace name |
| `title` | string | Pane title |
| `cwd` | string | Current working directory (URL format: `file://host/path`) |
| `tty_name` | string? | Optional TTY device name |
| `is_active` | boolean | Whether pane is focused |
| `is_zoomed` | boolean | Whether pane is zoomed |
| `size` | object | `{ rows, cols, pixel_width, pixel_height, dpi }` |
| `cursor_x`, `cursor_y` | number | Cursor position |
| `tab_title`, `window_title` | string | Tab/window display titles |

**Note**: `pid` is NOT in `list` output. It's in `list-clients`. For matching, use `cwd` or `tty_name`.

### Focus approach

1. `wezterm cli list --format json` → find pane by CWD or TTY.
2. `wezterm cli activate-pane --pane-id <id>` → focus the pane.
3. Bring WezTerm window to front via `activate_app_by_name("WezTerm")`.

### Limitations

- **No PID** in `list` output — must match by CWD or TTY.
- **No `activate-window` command** — `activate-pane` focuses within WezTerm's mux but doesn't raise the OS window. Need app activation separately.
- CWD is a URL format (`file://hostname/path`), needs parsing.

### Compatibility

- `list`: since build 20220624
- `activate-pane`: since build 20230326
- No special config required — CLI connects to running GUI instance automatically.
- Bundle ID: `com.github.wez.wezterm`

## How it works

1. Check bundle ID is `com.github.wez.wezterm`.
2. Run `wezterm cli list --format json`.
3. Match pane by CWD (from `workspace.yaml`) or `tty_name`.
4. Run `wezterm cli activate-pane --pane-id <matched_id>`.
5. Activate the WezTerm app to bring window to front.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/terminals/wezterm.rs`
- [ ] Matches by CWD (primary) with TTY fallback
- [ ] Handles `wezterm` CLI not found gracefully (`NotApplicable`)
- [ ] Parses CWD URL format correctly
- [ ] Unit tests: JSON parsing, CWD matching, URL parsing
- [ ] `just check` passes

## Notes

- WezTerm's CLI is the most powerful among non-AppleScript terminals.
- CWD matching is reliable since `workspace.yaml` has the exact CWD.
- The CWD in list output is a URL (`file://host/path`) — extract the path component.
- `tty_name` is optional and may not always be present.
