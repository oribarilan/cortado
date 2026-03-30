---
status: pending
---

# Prompt to use tmux for detached sessions

## Goal

When a user tries to open a copilot-session activity that is in a detached tmux session, and tmux integration is disabled in settings, show a confirmation popup asking if they want to use tmux for this one action.

## Context

Detached tmux sessions have no terminal tab viewing them. Without tmux commands, the only option is app activation (brings the terminal to front but can't show the detached session). The tmux strategy is the only way to reattach a detached session.

Currently, the waterfall silently falls through to app activation, which isn't helpful — the user expects to see their session but gets a generic terminal focus instead.

## Proposed UX

When all of these are true:
- tmux is disabled in settings
- the target session is in a detached tmux session (no client attached)
- the Ghostty/terminal-specific strategy also fails (no matching tab)

Show a dialog:

> This session is in a detached tmux session. tmux integration is disabled, so cortado can't reattach it.
>
> [Use tmux this time]  [Cancel]

"Use tmux this time" runs the tmux strategy for this single action without changing the setting.

## Notes

- This requires the backend to distinguish "detached tmux session" from "no terminal detected" — currently both result in app activation.
- The frontend needs a way to receive the "detached session" signal and show a dialog before falling back.
- Consider: should we also offer "Enable tmux integration" as a third button to change the setting permanently?
