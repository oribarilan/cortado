# cortado-copilot

Copilot CLI plugin for [Cortado](https://github.com/oribarilan/cortado) session tracking.

## How it works

This plugin uses Copilot CLI's hooks system to track session lifecycle events and publish state to the [harness interchange format](../../specs/harness-interchange.md). Cortado reads these interchange files to display live session status.

## Installation

Install via Cortado Settings:

1. Add a **Copilot Sessions** feed in Settings
2. Click **Install Plugin** when prompted
3. Cortado runs `copilot plugin install` with the embedded plugin files

The plugin is automatically loaded by Copilot CLI on every session start.

## Manual installation

```bash
copilot plugin install ./plugins/copilot
```

## Status tracking

| Hook | Condition | Status |
|------|-----------|--------|
| `sessionStart` | file doesn't exist yet | working |
| `userPromptSubmitted` | | working |
| `preToolUse` | `toolName` is `ask_user` | question |
| `preToolUse` | other tools | working |
| `postToolUse` | current status is not `question` | working |
| `postToolUse` | current status is `question` | (no write -- preserves question) |
| `sessionEnd` | | idle |

### Hook ordering

Copilot CLI fires hooks with some non-obvious ordering:

- **Prompt mode** (`-p`): `userPromptSubmitted` fires before `sessionStart`. The script handles this by skipping `sessionStart` when a file already exists.
- **Concurrent tools**: When copilot calls `ask_user` alongside `report_intent`, `postToolUse(report_intent)` fires while `ask_user` is still waiting for input. Without protection, this would overwrite the `question` status. The script reads the current file status and refuses to overwrite `question`.
- **Session end**: Writes `idle` instead of deleting the file. This matches the OpenCode plugin behavior -- the session appears as idle until GenericProvider's PID liveness check cleans it up.

## Files

- `plugin.json` -- plugin manifest (name: "cortado")
- `hooks.json` -- hook configuration (sessionStart, userPromptSubmitted, preToolUse, postToolUse, sessionEnd)
- `cortado-hook.sh` -- single hook script that dispatches by hook type
