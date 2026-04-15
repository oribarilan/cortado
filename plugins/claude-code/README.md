# cortado-claude-code

Claude Code plugin for [Cortado](https://github.com/oribarilan/cortado) session tracking.

## How it works

This plugin uses Claude Code's hooks system to track session lifecycle events and publish state to the [harness interchange format](../../specs/harness-interchange.md). Cortado reads these interchange files to display live session status.

## Installation

Install via Cortado Settings:

1. Add a **Claude Code Sessions** feed in Settings
2. Click **Install Plugin** when prompted
3. Cortado registers a local marketplace and installs the plugin via `claude plugin install`

The plugin is automatically loaded by Claude Code on every session start.

## Status tracking

| Hook | Condition | Status |
|------|-----------|--------|
| `SessionStart` | file doesn't exist yet | working |
| `UserPromptSubmit` | | working |
| `PreToolUse` | `tool_name` is `AskUserQuestion` | question |
| `PreToolUse` | other tools | working |
| `PermissionRequest` | | approval |
| `PostToolUse` | current status is not `question`/`approval` | working |
| `PostToolUse` | current status is `question` or `approval` | (no write -- preserves status) |
| `SessionEnd` | | idle |

### Hook ordering

Claude Code hooks have some differences from Copilot CLI:

- **Stop vs SessionEnd**: `Stop` fires per-turn (every time Claude finishes responding). `SessionEnd` fires when the session actually terminates. This plugin uses `SessionEnd` for the idle transition.
- **PermissionRequest**: A dedicated event that fires when a permission dialog appears. Maps to the `approval` status.
- **Concurrent tools**: Same issue as Copilot -- when `AskUserQuestion` runs alongside other tools, `PostToolUse` for the other tools can fire while the question is still pending. The script reads the current file status and refuses to overwrite `question` or `approval`.
- **Session end**: Writes `idle` instead of deleting the file. This matches the Copilot/OpenCode plugin behavior -- the session appears as idle until GenericProvider's PID liveness check cleans it up.

## Files

- `.claude-plugin/plugin.json` -- plugin manifest (name: "cortado")
- `hooks/hooks.json` -- hook configuration (SessionStart, UserPromptSubmit, PreToolUse, PermissionRequest, PostToolUse, SessionEnd)
- `scripts/cortado-hook.sh` -- single hook script that dispatches by hook type
