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

| Hook | Status |
|------|--------|
| `sessionStart` | working (creates file if not exists) |
| `userPromptSubmitted` | working |
| `preToolUse` (ask_user) | question |
| `preToolUse` (other) | working |
| `postToolUse` | working |
| `sessionEnd` | cleanup (deletes file) |

## Files

- `plugin.json` -- plugin manifest (name: "cortado")
- `hooks.json` -- hook configuration (sessionStart, userPromptSubmitted, preToolUse, postToolUse, sessionEnd)
- `cortado-hook.sh` -- single hook script that dispatches by hook type
