---
status: done
---

# Focus strategy: tmux pane switching

## Goal

Implement the tmux focus strategy for the `TerminalFocusResolver`. When tmux is detected in the PID ancestry, switch the tmux client to the exact pane containing the copilot session.

## How it works

1. Check `FocusContext.tmux_server_pid` -- if `None`, return `NotApplicable`.
2. Run `tmux list-panes -a -F '#{session_name}:#{window_index}.#{pane_index} #{pane_pid}'`.
3. Match `pane_pid` against `FocusContext.ancestors` to find the target pane.
4. Run `tmux list-clients -F '#{client_tty} #{client_session} #{client_pid}'`.
5. Pick the best client: prefer one already attached to the target session.
6. Run `tmux switch-client -c <client_tty> -t <pane>` + `tmux select-pane -t <pane>`.
7. Activate the terminal app (use `FocusContext.terminal_app_pid`).
8. Return `Focused`.

If any tmux command fails (tmux not installed, not running, etc.), return `Failed`.

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/tmux.rs` with `try_focus(ctx: &FocusContext) -> FocusResult`
- [ ] Parses `tmux list-panes` and `tmux list-clients` output
- [ ] Handles edge case: target session has no attached client (picks another)
- [ ] Handles edge case: `tmux` binary not found (returns `Failed`, not panic)
- [ ] Unit tests: tmux output parsing (panes, clients), pane matching, client selection
- [ ] `just check` passes

## Relevant files

- `src-tauri/src/terminal_focus/tmux.rs` -- new file
