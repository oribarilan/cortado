---
status: pending
---

# Claude Code plugin

## Context

Create the Claude Code plugin that runs inside Claude Code sessions and writes interchange files to `~/.config/cortado/harness/<pid>.json`. This is modeled after the Copilot plugin (`plugins/copilot/`), adapted for Claude Code's plugin system.

**Value delivered**: Claude Code sessions become visible to Cortado's harness system.

## Related Files

- `plugins/copilot/cortado-hook.sh` -- reference implementation (Copilot hook script)
- `plugins/copilot/plugin.json` -- reference manifest (Copilot format)
- `plugins/copilot/hooks.json` -- reference hooks config (Copilot format)
- `specs/harness-interchange.md` -- interchange file format spec

## Dependencies

- None

## Acceptance Criteria

- [ ] `plugins/claude-code/` directory exists with proper Claude Code plugin structure:
  - `.claude-plugin/plugin.json` -- manifest with name `"cortado"`, description, version
  - `hooks/hooks.json` -- hook event registrations for SessionStart, UserPromptSubmit, PreToolUse, PermissionRequest, PostToolUse, SessionEnd
  - `scripts/cortado-hook.sh` -- the hook script (referenced via `${CLAUDE_PLUGIN_ROOT}/scripts/cortado-hook.sh`)
- [ ] Hook script includes `# cortado-plugin-version: 1` header for version detection
- [ ] Hook script writes interchange JSON v1 to `~/.config/cortado/harness/$PPID.json` with `"harness": "claude-code"`
- [ ] Status mapping:
  - `SessionStart` -> `working`
  - `UserPromptSubmit` -> `working`
  - `PreToolUse` -> `question` for `AskUserQuestion` tool, `working` otherwise
  - `PermissionRequest` -> `approval`
  - `PostToolUse` -> `working` (guarded: don't overwrite `question`/`approval`)
  - `SessionEnd` -> `idle`
- [ ] JSON input fields use snake_case (`session_id`, `tool_name`, `cwd`) -- NOT camelCase like Copilot
- [ ] Git metadata (repo, branch) is resolved from session cwd
- [ ] Atomic writes (temp file + mv)
- [ ] No jq dependency -- bash builtins and standard unix tools only
- [ ] `README.md` in `plugins/claude-code/` with brief description

## Verification

- **Ad-hoc**: Review the script against the Copilot reference (`plugins/copilot/cortado-hook.sh`) and verify structural parity. Verify the hooks.json matches Claude Code's expected format (from docs). Verify the plugin.json manifest is valid.

## Notes

### Claude Code hook format differences from Copilot

Claude Code hooks config (`hooks/hooks.json`):
```json
{
  "description": "Cortado session tracking for Claude Code",
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/scripts/cortado-hook.sh SessionStart",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

Key differences from Copilot:
- Event names are PascalCase (`SessionStart` not `sessionStart`)
- Uses `"command"` field (not `"bash"`)
- Uses `"timeout"` in seconds (not `"timeoutSec"`)
- Has a `"matcher"` field (use empty string to match all)
- Scripts referenced via `${CLAUDE_PLUGIN_ROOT}` (not relative paths)
- Nested structure: event -> matcher groups -> hooks array
- No `"version"` field at top level (Copilot has `"version": 1`)
- Optional top-level `"description"` field for plugin hooks
- JSON input uses snake_case: `tool_name`, `session_id` (not `toolName`, `sessionId`)

### Events that differ from Copilot

- **`Stop` is per-turn, not per-session.** It fires every time Claude finishes responding. Do NOT use it for session end -- it would cause working/idle flicker on every turn. Use `SessionEnd` instead.
- **`SessionEnd`** fires when the session terminates (equivalent of Copilot's `sessionEnd`). Has a 1.5s default timeout for plugin hooks, but our file write is fast enough.
- **`PermissionRequest`** is a dedicated event (no Copilot equivalent). Fires when a permission dialog is about to be shown. Receives `tool_name` and `tool_input` like PreToolUse.
- **`UserPromptSubmit`** (not `userPromptSubmitted` like Copilot).

### Tool name for question status

Claude Code's `PreToolUse` hook receives JSON with a `tool_name` field. The tool that asks users questions is **`AskUserQuestion`** -- "Asks the user one to four multiple-choice questions." This is the equivalent of Copilot's `ask_user`.

Map: `PreToolUse` where `tool_name == "AskUserQuestion"` -> `question`.

### PostToolUse guard

Same issue as Copilot: concurrent tool calls can cause PostToolUse for other tools to overwrite the `question` or `approval` status. The guard logic must:
1. Read the current interchange file's status
2. If status is `question` or `approval`, skip the write
3. Use `tool_name` (snake_case) to check -- if `tool_name == "AskUserQuestion"`, skip

### Script location

The canonical Claude Code plugin layout puts scripts at `scripts/` (plugin root), not `hooks/scripts/`. Our structure:
```
plugins/claude-code/
  .claude-plugin/plugin.json
  hooks/hooks.json
  scripts/cortado-hook.sh
  README.md
```
