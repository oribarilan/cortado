# cortado-opencode

OpenCode plugin that publishes session state for [Cortado](https://github.com/oribarilan/cortado). When installed, your OpenCode coding sessions appear in Cortado's tray and panel.

## Installation

**Recommended:** Add an `opencode-session` feed in Cortado's Settings -- it offers a one-click "Install Plugin" button that handles everything. Cortado detects when the plugin is outdated and prompts you to update.

**Manual:** Copy `src/plugin-bundle.ts` to your OpenCode global plugins directory:

```bash
cp src/plugin-bundle.ts ~/.config/opencode/plugins/cortado-opencode.ts
```

## What it does

Tracks OpenCode session status and writes state to `~/.config/cortado/harness/`. Cortado reads these files and shows your sessions with repo, branch, and status.

Detected states:
- **working** -- agent is busy (from `session.status` busy/retry events)
- **idle** -- agent is idle (from `session.status` idle events)
- **question** -- agent asked a question and is waiting for your answer (from `question.asked` events)
- **approval** -- agent needs permission to proceed (from `permission.asked` events)

## Configuration

No configuration needed. The plugin automatically:
- Detects your project's git repository and branch
- Tracks session status changes (busy, idle, retry)
- Detects when the agent is waiting for your input (questions, permissions)
- Cleans up state files when the session ends

## Requirements

- OpenCode with plugin support
- Cortado with `opencode-session` feed configured
