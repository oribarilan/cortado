# US-copilot-plugin: Copilot CLI Extension for Interchange-Based Session Tracking

## Theme

Replace the native `CopilotProvider` -- which reads Copilot's internal files (`workspace.yaml`, `events.jsonl`, lock files) and infers status in Rust -- with a **Copilot CLI extension** that publishes session state to the harness interchange format. This follows the exact same pattern as the OpenCode plugin: build an extension, embed it in the Cortado binary, prompt the user to install it when configuring the feed, and require it for session tracking.

The result is simpler Rust code (reuses `GenericProvider`), more accurate status tracking (the extension has real-time event access), and consistent UX across both coding agent integrations.

## How It Works

### Copilot CLI extension system

Copilot CLI supports a **full extension system** distinct from the shell-command hooks in `.github/hooks/`. Extensions are persistent Node.js child processes that attach to the agent session via JSON-RPC:

- **User-scoped:** `~/.copilot/extensions/<name>/extension.mjs` (global, applies to all repos)
- **Project-scoped:** `.github/extensions/<name>/extension.mjs` (per-repo)
- **Entry point:** Must be named `extension.mjs` (ES modules only, no TypeScript)
- **SDK:** `@github/copilot-sdk/extension` is auto-resolved by the CLI runtime -- no npm install needed
- **Lifecycle:** Extensions are forked on session start, receive SIGTERM on session end, and hot-reload on `/clear`

This is the equivalent of OpenCode's `~/.config/opencode/plugins/` directory. The user-scoped path (`~/.copilot/extensions/cortado/extension.mjs`) is what Cortado installs to.

### Cortado extension

A single `.mjs` file in `plugins/copilot/src/extension.mjs` that:

1. Calls `joinSession()` from the Copilot SDK to attach to the session
2. Subscribes to session events via `session.on()` to track status changes
3. Writes interchange-format JSON files to `~/.config/cortado/harness/`
4. Cleans up the session file on process exit

### Event-to-status mapping

Uses `session.on()` event subscriptions exclusively -- NO lifecycle hooks registered. This avoids the **hook overwrite bug** ([copilot-cli#2076](https://github.com/github/copilot-cli/issues/2076)) where multiple extensions registering hooks causes only the last-loaded extension's hooks to fire. All needed data is available via events and process context (`process.cwd()`, exit handlers).

| Event | Interchange Status |
|-------|-------------------|
| `user.message` | `working` |
| `assistant.turn_start` | `working` |
| `tool.execution_start` with `toolName === "ask_user"` | `question` |
| `tool.execution_start` (other tools) | `working` |
| `assistant.message` with `ask_user` in `toolRequests` | `question` |
| `assistant.message` with other `toolRequests` | `approval` |
| `session.idle` | `idle` |
| `assistant.turn_end` | `idle` (fallback if `session.idle` doesn't fire) |
| `session.shutdown` | delete file |

The `question` status maps to `AttentionPositive` in the harness feed, giving users visible "needs attention" signals when Copilot asks a question.

### PID handling

The extension runs as a **child process** of the CLI. It uses `process.ppid` (the CLI's PID) for both the interchange file name (`<ppid>.json`) and the `pid` field. This is correct because:

- Terminal focus walks up the PID tree from the harness PID to find the owning terminal -- using the CLI's PID puts us one step closer
- Liveness checks (`kill(pid, 0)`) target the CLI process, which is the "source of truth" for whether the session is alive
- When the CLI exits, the extension receives SIGTERM and cleans up; if it doesn't (SIGKILL), `GenericProvider` detects the dead PID and auto-deletes the stale file

### CWD and git metadata

- `process.cwd()` provides the working directory (inherited from the CLI's cwd)
- Git repo/branch resolved via `child_process.execFileSync` (same approach as OpenCode plugin, but using Node.js child_process instead of Bun's `$`)

### Session identity

- File named `<ppid>.json` (CLI process PID)
- `id` field starts as `String(process.ppid)`, can be updated if a session ID becomes available via events
- `harness` field: `"copilot"`

## What Changes

### Removed

- `src-tauri/src/feed/harness/copilot.rs` -- the entire native CopilotProvider (YAML parsing, JSONL event inference, lock file scanning)
- `serde-saphyr` dependency from `Cargo.toml` (confirmed only used by CopilotProvider)

### Added

- `plugins/copilot/src/extension.mjs` -- the Cortado extension for Copilot CLI
- `check_copilot_extension` / `install_copilot_extension` Tauri commands in `settings_config.rs`
- `dependency` and `setup` fields on the `copilot-session` feed type in `feedTypes.ts`
- Copilot extension update detection in `cortado_update.rs`

### Changed

- `feed/mod.rs`: `copilot-session` now uses `GenericProvider::new("copilot")` instead of `CopilotProvider::new()`
- `feedTypes.ts`: `copilot-session` gains `dependency` (checks for `copilot` binary) and `setup` (extension check/install)
- Notes in `feedTypes.ts` updated to reflect the new discovery mechanism

## Design Decisions

- **Extension system over shell hooks:** Copilot CLI's `.github/hooks/` shell hooks are per-repo, stateless, and invoke a new process per event. The extension system (`~/.copilot/extensions/`) is user-scoped (global), persistent (single process per session), and provides structured event subscriptions. It's the direct analogue of OpenCode's plugin system.
- **`session.on()` events over lifecycle hooks:** The hook overwrite bug (copilot-cli#2076) means registering `onSessionStart`/`onPreToolUse` etc. can silently break other extensions. Using `session.on()` event listeners avoids this -- multiple listeners coexist safely. We get cwd from `process.cwd()`, cleanup from process exit handlers, and the first `user.message` event provides the initial prompt if needed. Full parity with OpenCode's status tracking: working, idle, question (`AttentionPositive`), and approval.
- **`process.ppid` for PID:** The extension is a child process of the CLI. Using the parent PID (CLI) rather than the child PID (extension) gives correct terminal focus resolution and accurate liveness checks.
- **Plain JavaScript (.mjs):** The Copilot CLI extension system only supports `.mjs` entry points. No TypeScript, no build step. The source file IS the artifact.
- **Summary left empty:** Matches OpenCode behavior, where summary is only populated during retry events. The activity title in Cortado shows repo + branch + cwd regardless.
- **No justfile cascade:** Unlike the OpenCode plugin (TypeScript, has build/test/format), the Copilot extension is a single `.mjs` file with no build step. A `node --check` syntax validation is added to the root `just check` instead of a full justfile cascade.
- **Version header:** Same `// cortado-plugin-version: N` convention as the OpenCode plugin, enabling update detection.

## Task Sequencing

```
01-copilot-extension ───────────┐
                                 ├──> 03-update-feed-catalog ──┐
02-backend-setup-commands ───────┘                              │
                                                                ├──> 06-update-specs
04-switch-to-generic-provider (after 01) ──────────────────────┘
05-plugin-update-detection (after 02, parallel with 03-04)
```

Tasks 01 and 02 are semi-independent (02 needs the extension file from 01 for embedding). Tasks 03, 04, and 05 can proceed in parallel once their dependencies are met. Task 06 is last.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-copilot-extension.md` | Build the Cortado extension for Copilot CLI |
| 02 | `02-backend-setup-commands.md` | Add check/install Tauri commands, embed extension source |
| 03 | `03-update-feed-catalog.md` | Add dependency + setup to copilot-session in feedTypes.ts |
| 04 | `04-switch-to-generic-provider.md` | Replace CopilotProvider with GenericProvider, remove copilot.rs |
| 05 | `05-plugin-update-detection.md` | Add copilot extension update check to CortadoUpdateFeed |
| 06 | `06-update-specs.md` | Update specs, docs, and changelog |

## Resolved Questions

- **`serde_saphyr` removal:** Confirmed only used in `copilot.rs` (5 hits, all in that file). Remove `serde-saphyr` (note: hyphen in Cargo.toml, underscore in Rust source) from `Cargo.toml`.
- **Summary field:** Left empty, matching OpenCode behavior. Summary is only populated for specific events (e.g., retries in OpenCode). The activity title shows repo + branch + cwd which provides sufficient context.
- **Copilot CLI binary name:** Confirmed as `copilot` (at `/opt/homebrew/bin/copilot`).

## Open Questions

- **Hook overwrite bug timeline:** The bug (copilot-cli#2076) may be fixed in a future CLI version. If fixed, we could optionally register `onSessionStart` for richer initial context (e.g., `initialPrompt` as summary). Current design works without hooks, so this is a future enhancement.
- **Session resume:** Copilot CLI's `--resume` flag resumes existing sessions. The extension should handle this gracefully (same cwd, potentially different session ID). Hooks have a known bug where they don't fire on resume (copilot-cli#1503), but `session.on()` events should still work.
- **`session.idle` reliability:** If `session.idle` doesn't fire reliably after `assistant.turn_end`, consider adding `assistant.turn_end` as a fallback idle trigger. Verify empirically.
