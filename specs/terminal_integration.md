# Terminal Integration

Cortado's terminal focus system navigates to the exact terminal tab/pane running a coding agent session. This document specifies the architecture, supported terminals, and integration strategy.

## Design rationale

The core problem: a user has multiple terminal tabs/windows open, each running different coding agent sessions. When they click "Open in Ghostty" on a session activity, cortado needs to bring the *right* tab to focus — not just the terminal app.

This is hard because:

- There's no universal API for terminal tab management across macOS terminals.
- When tmux is involved, the terminal app isn't in the process ancestry — the process tree goes copilot → shell → tmux server → launchd, not through the terminal.
- Each terminal has a different scripting API (AppleScript, CLI, remote control) with different capabilities and matching mechanisms (TTY, PID, CWD, tab name).

The design addresses this with a **strategy waterfall** — each layer tries a more specific approach, falling back to less precise ones. Adding support for a new terminal requires only implementing one function.

## Architecture

Terminal focus uses a **two-phase** approach: an optional tmux pre-step followed by a terminal strategy waterfall. The phases are composable — tmux handles pane-level navigation, then the terminal strategy handles app/tab-level activation.

```
Phase 1: tmux pre-step (optional, terminal-agnostic)
  |  Navigates to the exact pane within tmux.
  |  Uses: tmux list-panes, select-window, select-pane.
  |  Does NOT activate the terminal app — that's Phase 2's job.
  |  Skipped if: tmux not detected, or tmux disabled in settings.
  |
Phase 2: Terminal strategy waterfall
  |  Each terminal has its own strategy, selected by bundle ID.
  |  Focuses the correct tab/window using the terminal's native API,
  |  then activates the app.
  |  Only one strategy runs (first bundle ID match).
  |  Fallback: app activation (brings terminal to front, no tab targeting).
```

### Why two phases?

tmux was originally a competing strategy in the waterfall (position 0). It navigated the pane, activated the app by PID, and returned `Focused` — preventing terminal-specific strategies from ever running. This broke Ghostty+tmux: tmux would bring Ghostty to the foreground, but since the Ghostty strategy never ran, the wrong tab could remain visible.

The fix: tmux is a **composable pre-step** that only navigates within tmux, then *always* falls through to the terminal waterfall. This lets the Ghostty strategy switch to the correct tab (matching by tmux session name) after tmux has already navigated to the correct pane.

### tmux behavior (Phase 1)

tmux navigation is implemented as `tmux::try_navigate()`, which returns `Result<bool, String>`:
- `Ok(true)` — tmux pane navigation succeeded
- `Ok(false)` — tmux not applicable (not detected, or disabled in settings)
- `Err(msg)` — tmux command failed (logged, falls through to Phase 2)

When tmux is detected in the process ancestry:

- **Session has an attached client (common case)**: uses `select-window` + `select-pane` to navigate within the existing tab. Does *not* steal the tab or switch what it displays.
- **Session is detached (no terminal tab viewing it)**: uses `switch-client` to attach the session to an available terminal tab. This is common in tmux workflows where sessions are detached and reattached later — cortado handles it seamlessly.

After tmux navigation, Phase 2 (terminal waterfall) always runs to activate the app and switch to the correct tab.

tmux can be toggled off in Settings > Agents. When disabled, Phase 1 is skipped entirely, but terminal strategies in Phase 2 still use tmux *data* (to map PIDs to sessions) for tab matching (e.g., Ghostty maps tmux session name to tab name).

## tmux integration

tmux is the highest-precision, most universal focus strategy. It works with any terminal emulator and provides exact pane targeting.

### How it works

1. **Pane discovery**: `tmux list-panes -a -F '#{session_name}:#{window_index}.#{pane_index} #{pane_pid}'` — finds which pane contains the copilot process (by PID or ancestor PID match).
2. **Client discovery**: `tmux list-clients -F '#{client_pid} #{client_session}'` — finds which terminal tab is viewing the target session.
3. **Navigate** (pane-level only, no app activation):
   - If the session has a client: `tmux select-window` + `tmux select-pane` — navigates within the existing tab (non-destructive).
   - If the session is detached (no terminal tab viewing it): `tmux switch-client` — attaches the session to an available terminal tab. Detached sessions are common in tmux workflows; cortado brings them back without manual reattachment.

App activation is left to Phase 2 (terminal strategy waterfall), which handles both bringing the terminal to the foreground and switching to the correct tab.

### Terminal resolution with tmux

When tmux is in the process ancestry, the terminal app is *not* a direct ancestor of the copilot process:

```
copilot → shell → tmux server → launchd   (no terminal in this chain)
```

To find the terminal, cortado:
1. Finds the tmux client attached to the copilot's specific session (via `tmux list-clients` + `tmux list-panes`).
2. Walks that client's PID ancestry to find the terminal app.

This per-session client resolution is critical when multiple terminals run tmux simultaneously — a copilot in Terminal.app's tmux session correctly resolves to Terminal.app, not Ghostty.

### Ghostty + tmux interaction

Ghostty tabs map 1:1 to tmux sessions (each tab runs a separate tmux session, and the tab name = tmux session name). With the two-phase architecture, both phases contribute:

1. **Phase 1 (tmux pre-step)**: navigates to the correct pane within the tmux session.
2. **Phase 2 (Ghostty strategy)**: maps the copilot PID to a tmux session name, finds the Ghostty tab with that name via AppleScript, switches to it, and activates the app.

This gives both pane-level precision (tmux) and tab-level precision (Ghostty) in a single focus action.

### Settings

tmux integration is controlled by a toggle in Settings > Agents:
- **Enabled (default)**: pane-level switching via tmux commands.
- **Disabled**: tmux data is still used for terminal detection and Ghostty tab matching, but tmux pane/window switching commands are not executed.

### Commands used

| Command | Purpose |
|---------|---------|
| `tmux list-panes -a -F '...'` | Discover all panes with PIDs |
| `tmux list-clients -F '...'` | Find clients and their sessions |
| `tmux select-window -t <target>` | Switch to window (non-destructive) |
| `tmux select-pane -t <target>` | Focus pane within window |
| `tmux switch-client -c <tty> -t <target>` | Switch client to different session (destructive) |

### Terminal-specific strategy contract

Each terminal strategy is a function:

```rust
fn try_focus(ctx: &FocusContext) -> FocusResult
```

A strategy must:
1. Check `ctx.terminal_app_bundle` — return `NotApplicable` if not its terminal.
2. Attempt to focus the correct tab/window using terminal-native APIs.
3. Return `Focused`, `NotApplicable`, or `Failed`.

Adding a new terminal: implement `try_focus`, add it to the strategy list in `mod.rs`.

### Terminal detection

The terminal app is identified during the PID ancestry walk by matching process names against a known list (`KNOWN_TERMINALS` in `pid_ancestry.rs`). This is more reliable than `NSRunningApplication` (which is blocked by macOS security in some contexts).

When tmux is involved, the terminal isn't in the copilot's direct ancestry. The code finds the tmux client attached to the *copilot's specific session* (not just any client) and walks that client's ancestry to find the terminal. This ensures a copilot running in Terminal.app resolves to Terminal.app, even if Ghostty is also running tmux sessions.

### Security

All strings interpolated into AppleScript commands are escaped via `escape_applescript()` to prevent injection attacks. This handles double quotes and backslashes — the two special characters in AppleScript strings.

### Performance

Focus info (terminal name, tmux detection) is cached per session ID. The PID ancestry walk runs once when a session is first discovered; subsequent polls reuse the cached result. The cache is pruned when sessions disappear.

## Supported terminals

### Implemented

| Terminal | Strategy | Matching | Precision | Notes |
|----------|----------|----------|-----------|-------|
| **Any (via tmux)** | `tmux select-window/select-pane` | PID ancestry → tmux pane | Exact pane | Pre-step, not a waterfall entry. Runs before terminal strategies. |
| **Ghostty** | AppleScript `focus` | tmux session name (exact) or CWD substring | Tab-level | Requires 1.3+. Best with tmux (exact session name match). |
| **Terminal.app** | AppleScript `tty of tab` | TTY | Exact tab | Most mature AppleScript API. No config needed. |
| **iTerm2** | AppleScript `tty of session` | TTY | Exact pane | Handles split panes. `select` on window/tab/session. |
| **WezTerm** | `wezterm cli list --format json` | CWD/TTY | Pane-level | No PID in list output. CWD matching with URL parsing. |
| **kitty** | `kitty @ focus-window --match pid:<pid>` | PID | Exact window | Requires `allow_remote_control` in kitty config. |
| **Alacritty** | None | N/A | App-only | No tabs, no scripting API. App activation fallback. |
| **Warp** | None | N/A | App-only | No scripting dictionary. No session focus API. |

### Terminal.app strategy

Uses AppleScript to enumerate tabs and match by TTY:

```applescript
tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            if tty of t is "/dev/ttysNNN" then
                set selected tab of w to t
                set index of w to 1
                activate
                return
            end if
        end repeat
    end repeat
end tell
```

TTY matching is precise — each tab has a unique TTY device path. Only works without tmux (with tmux, the copilot's TTY is a tmux PTY, not the terminal's).

### iTerm2 strategy

Uses AppleScript to enumerate sessions (split panes) and match by TTY:

```applescript
tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if (tty of s) is "/dev/ttysNNN" then
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

Focus sequence: select window → select tab → select session → activate. Handles split panes naturally (sessions are siblings within a tab).

### WezTerm strategy

Uses the `wezterm` CLI:

```bash
wezterm cli list --format json    # List panes with cwd, tty_name, pane_id
wezterm cli activate-pane --pane-id <id>   # Focus the pane
```

CWD in list output is a URL format (`file://hostname/path`) — needs path extraction. `activate-pane` focuses within WezTerm's mux but doesn't raise the OS window — needs separate app activation.

### kitty strategy

Uses kitty's remote control protocol:

```bash
kitty @ ls                                    # JSON with all windows, tabs, pids, cwds
kitty @ focus-window --match pid:<pid>        # Focus by PID (switches OS windows too)
```

Requires `allow_remote_control yes` or `allow_remote_control socket-only` in `kitty.conf`. When not enabled, the strategy returns `NotApplicable` (not an error).

## TTY resolution

For terminals that support TTY matching (Terminal.app, iTerm2), the copilot process TTY is resolved via:

```bash
ps -p <copilot_pid> -o tty=
```

Returns e.g. `ttys107`. Full device path: `/dev/ttys107`.

When tmux is in use, the copilot's TTY is a tmux PTY, not the terminal's PTY. TTY matching won't work — the tmux strategy (Layer 1) handles this case instead.

## Implementation

```
src-tauri/src/terminal_focus/
  mod.rs              # FocusContext, FocusResult, two-phase focus_terminal(), app_activation, escape_applescript
  pid_ancestry.rs     # PID walk, tmux client resolution, terminal detection
  tmux.rs             # Phase 1: tmux pane navigation pre-step (try_navigate)
  e2e.rs              # E2E tests (ignored, run via `just local-e2e`)
  terminals/
    mod.rs            # Phase 2: terminal strategy registry, shared TTY resolution helper
    ghostty.rs        # Ghostty 1.3+ AppleScript tab switching
    terminal_app.rs   # macOS Terminal.app AppleScript TTY matching
    iterm2.rs         # iTerm2 AppleScript TTY matching (split pane support)
    wezterm.rs        # WezTerm CLI-based pane focusing (CWD/TTY matching)
    kitty.rs          # kitty remote control PID matching
```

## Settings

The Agents settings tab shows:

- **tmux integration**: toggle on/off, detection status
- **Ghostty tab switching**: availability based on version, scripting status
- **Accessibility**: window focus by title (future, requires OS permission)

Terminal-specific strategies are auto-detected and do not require user configuration.
