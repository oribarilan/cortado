---
status: done
priority: high
---

# Generalize Coding Agent Session Focusing

## Goal

Enable session focusing for all coding agent feed types (opencode, and any future harness-based agents), not just Copilot. The backend already supports this -- the gap is a single hardcoded frontend check.

## Context

Session focusing lets users click an activity to bring the agent's terminal tab/pane to the foreground. The backend (`terminal_focus/` module) is fully agent-agnostic: it takes a PID, walks the process tree, and uses terminal-specific strategies (tmux, Ghostty, iTerm2, etc.) to focus the exact pane.

The `focus_session` Tauri command already searches all harness feeds. The opencode plugin already publishes `pid` in its session state. Everything works end-to-end -- except `supportsFocus()` in `src/shared/utils.ts:102` is hardcoded to `feed.feed_type === "copilot-session"`, so opencode sessions never show the focus action in the UI.

## Approach

Implemented **field-based detection**: `supportsFocus()` checks for the presence of a `focus_app` field on the activity, making it work automatically for any harness feed. The `feed` parameter was removed entirely since it's no longer needed.

## Acceptance criteria

- [x] OpenCode sessions support focusing (click-to-focus in tray and panel)
- [x] `supportsFocus()` is generalized -- not hardcoded to a single feed type
- [x] Any future harness-based feed type gets focusing automatically if it provides a PID
- [x] Copilot session focusing continues to work as before

## Related files

- `src/shared/utils.ts` -- `supportsFocus()` (now field-based)
- `src/shared/utils.test.ts` -- unit tests for `supportsFocus()`
- `src/App.tsx` -- tray focus invocation
- `src/main-screen/MainScreenApp.tsx` -- panel focus invocation

## Scope Estimate

Small
