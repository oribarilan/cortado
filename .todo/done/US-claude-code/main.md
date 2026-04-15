# US-claude-code

## Goal

Add Claude Code as a supported coding agent, matching the existing Copilot and OpenCode integrations. Users should be able to track active Claude Code sessions in Cortado with status updates, repo/branch info, and terminal focus -- installed via a single click in Settings.

## Background

The harness system is already agent-agnostic. The `GenericProvider` reads interchange JSON files from `~/.config/cortado/harness/`, filtering by the `harness` field. Adding a new agent requires:

1. A **plugin** that runs inside the agent and writes interchange files (like `plugins/copilot/` and `plugins/opencode/`)
2. A **backend match arm** in `instantiate_harness_feed()` to wire `"claude-code-session"` to `GenericProvider::new("claude-code")`
3. **Tauri commands** for plugin install/uninstall/check (like the copilot and opencode ones)
4. A **frontend catalog entry** in `feedTypes.ts`
5. **Spec/doc updates**

No changes to `harness/mod.rs`, `harness/feed.rs`, `harness/generic.rs`, or `harness_watcher.rs` -- the generic system handles everything.

### Claude Code plugin system

Claude Code uses a **plugin system** with a directory structure:

```
plugin-name/
  .claude-plugin/
    plugin.json          # manifest: name, description, version (optional)
  hooks/
    hooks.json           # event handlers
  scripts/
    cortado-hook.sh      # hook script
```

Hooks are shell commands that fire on lifecycle events. The relevant events are:

| Event | When | Status mapping |
|-------|------|----------------|
| `SessionStart` | Session begins or resumes | `working` |
| `UserPromptSubmit` | User sends a prompt | `working` |
| `PreToolUse` | Before a tool executes | `question` for `AskUserQuestion`, `working` otherwise |
| `PermissionRequest` | Permission dialog appears | `approval` |
| `PostToolUse` | After a tool completes | `working` (guard against overwriting `question`/`approval`) |
| `SessionEnd` | Session terminates | `idle` |

**Important:** `Stop` fires per-turn (every time Claude finishes responding), NOT per-session. `SessionEnd` fires when the session actually terminates. Using `Stop` would cause `working`/`idle` flicker on every turn.

Hook scripts receive JSON on stdin. JSON fields use **snake_case** (unlike Copilot's camelCase): `session_id`, `tool_name`, `tool_input`, `cwd`.

Scripts are referenced via `${CLAUDE_PLUGIN_ROOT}` for portable paths.

**Key differences from Copilot:**
- `${CLAUDE_PLUGIN_ROOT}` for script paths (Copilot uses relative paths)
- hooks.json uses nested matcher groups: event -> `[{matcher, hooks: [{type, command, timeout}]}]`
- Event names are PascalCase (`SessionStart` not `sessionStart`)
- Uses `"command"` field (not `"bash"`), `"timeout"` in seconds (not `"timeoutSec"`)
- `PermissionRequest` is a dedicated event (Copilot has no equivalent)
- `AskUserQuestion` tool maps to `question` (Copilot uses `ask_user`)
- JSON input fields are snake_case (`tool_name` not `toolName`, `session_id` not `sessionId`)

### Installation

Claude Code's `claude plugin install` command installs from **marketplaces only** -- there is no `--dir` flag. Raw paths in `enabledPlugins` are not recognized (tested).

Use a **local marketplace**: `claude plugin marketplace add <path>` accepts a local directory. Cortado writes the plugin files + a `marketplace.json` to `~/.config/cortado/marketplace/`, registers the marketplace, then installs the plugin from it. This uses the official plugin system with full `${CLAUDE_PLUGIN_ROOT}` support, namespacing, and update semantics.

See task 02 for the full directory structure, marketplace.json schema, and install/check/uninstall flows.

## Definition of Done

- [ ] A user can add a "Claude Code Sessions" feed in Settings with a single config line (`type = "claude-code-session"`)
- [ ] The Settings UI shows dependency status (claude binary), plugin install/uninstall button, and setup guidance
- [ ] Active Claude Code sessions appear as activities with correct status (working/idle/question/approval), repo, branch, and summary
- [ ] Opening an activity focuses the terminal running Claude Code (with tmux pane support)
- [ ] Plugin auto-update detection works (outdated plugin shows update prompt in the cortado-update feed)
- [ ] `just check` passes cleanly

## Task Priority

1. `01-plugin.md` -- Create the Claude Code plugin (hook script + plugin manifest). This is the core deliverable.
2. `02-backend.md` -- Backend wiring: match arm, Tauri commands, plugin embedding. Quick mechanical work.
3. `03-frontend.md` -- Frontend catalog entry. Quick mechanical work.
4. `04-update-detection.md` -- Wire plugin update detection into the cortado-update feed.
5. `05-spec-and-docs.md` -- Update specs and docs.

Tasks 02 and 03 can be done in parallel. Task 04 depends on 02. Task 05 can be done last or in parallel with 04.

## Cross-Cutting Concerns

- **Plugin version header:** Use `# cortado-plugin-version: 1` as the first comment line in the hook script, matching the Copilot convention. This enables outdated plugin detection.
- **Harness name:** Use `"claude-code"` as the harness name in interchange files (matching the feed type prefix `claude-code-session`).
- **PID strategy:** Use `$PPID` since hooks run as child processes of Claude Code. Same as Copilot. Verified: Claude Code is Node.js-based, and `$PPID` in a bash script spawned by Node.js always resolves to the Node.js process PID regardless of spawn method.
- **No jq dependency:** The hook script must work without jq, using only bash builtins and standard unix tools. Match the Copilot plugin's approach.
- **Atomic writes:** Write to a temp file then `mv`, same as Copilot.
- **Session end = idle:** Write `"idle"` on `SessionEnd` instead of deleting the file, matching Copilot/OpenCode behavior. GenericProvider's PID liveness check handles cleanup.
- **Snake_case JSON fields:** Claude Code's hook input uses `session_id`, `tool_name`, `tool_input` (not camelCase like Copilot). The hook script must use the correct field names.
- **Install strategy:** Uses a local marketplace at `~/.config/cortado/marketplace/`. See task 02.
