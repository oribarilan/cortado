---
status: done
---

# Build Cortado extension for Copilot CLI

## Goal

Create a single-file JavaScript extension (`extension.mjs`) that hooks into Copilot CLI's extension system, tracks session status via event subscriptions, and writes interchange-format JSON files to `~/.config/cortado/harness/`.

## Acceptance criteria

- [ ] `plugins/copilot/src/extension.mjs` exists as a complete, self-contained extension
- [ ] First line: `// cortado-plugin-version: 1` (version header for update detection)
- [ ] Imports `joinSession` from `@github/copilot-sdk/extension` (auto-resolved by CLI)
- [ ] Calls `joinSession()` with no hooks registered (avoids hook overwrite bug)
- [ ] Subscribes to session events via `session.on()` for status tracking:
  - `user.message` -> status = `working`
  - `assistant.turn_start` -> status = `working`
  - `tool.execution_start` with `toolName === "ask_user"` -> status = `question`
  - `tool.execution_start` (other) -> status = `working`
  - `assistant.message` with `ask_user` in `toolRequests` -> status = `question`
  - `assistant.message` with other `toolRequests` -> status = `approval`
  - `session.idle` -> status = `idle`
  - `assistant.turn_end` -> status = `idle` (fallback if `session.idle` doesn't fire reliably)
  - `session.shutdown` -> delete file
- [ ] Writes interchange-format JSON to `~/.config/cortado/harness/<ppid>.json`:
  - `version: 1`
  - `harness: "copilot"`
  - `id`: initially `String(process.ppid)`, updated if session ID becomes available
  - `pid: process.ppid` (CLI process PID, NOT extension PID)
  - `cwd: process.cwd()`
  - `status`: current status string
  - `last_active_at`: ISO 8601 timestamp
  - `repository`: parsed from `git remote get-url origin` (optional)
  - `branch`: from `git rev-parse --abbrev-ref HEAD` (optional)
  - `summary`: left empty (matches OpenCode behavior; summary is only populated for specific events like retries)
- [ ] All file writes are atomic (write to temp file with random suffix, then `fs.renameSync`)
- [ ] Creates `~/.config/cortado/harness/` directory if it doesn't exist (`fs.mkdirSync` with `recursive: true`)
- [ ] Cleans up session file on process exit via `process.on('exit')`, `process.on('SIGTERM')`, `process.on('SIGINT')`, `process.on('uncaughtException')`
- [ ] SIGTERM and SIGINT handlers call `process.exit(0)` after cleanup (Node.js doesn't auto-exit after caught signals)
- [ ] Uses `fs.unlinkSync` in exit handlers (async not available in exit handlers)
- [ ] Delete function silently ignores `ENOENT` (file may be already deleted by `session.shutdown` handler)
- [ ] Initial session file written immediately after `joinSession()` succeeds (status: `idle`)
- [ ] `plugins/copilot/README.md` with usage instructions
- [ ] No `package.json` needed (SDK is auto-resolved by Copilot CLI runtime)
- [ ] `node --check plugins/copilot/src/extension.mjs` passes (syntax validation, added to root `just check`)

## Notes

### Copilot CLI extension discovery

User-scoped extensions live at `~/.copilot/extensions/<name>/extension.mjs`. Cortado installs to `~/.copilot/extensions/cortado/extension.mjs`. The CLI automatically discovers and forks extensions on session start.

### SDK availability

The `@github/copilot-sdk` package is automatically resolved by the Copilot CLI runtime. No `npm install` is needed. This means:
- The extension can only run inside a Copilot CLI session (not standalone)
- No build step needed -- the `.mjs` source IS the artifact
- `include_str!` in Rust will embed it directly

### Why `session.on()` instead of hooks

The hook overwrite bug (copilot-cli#2076) means if multiple extensions register hooks (e.g., `onSessionStart`, `onPreToolUse`), only the last-loaded extension's hooks fire. Using `session.on()` event listeners avoids this -- multiple listeners coexist safely. All status information we need is available via events.

### PID strategy

`process.ppid` = Copilot CLI process PID (the extension is a forked child). This is correct for:
- **Terminal focus:** PID tree walk from CLI PID finds the owning terminal
- **Liveness:** `kill(ppid, 0)` checks if the CLI is alive
- **File naming:** `<ppid>.json` matches the convention

### Git metadata

Use `child_process.execFileSync` to resolve git info:
```javascript
import { execFileSync } from "node:child_process";

function gitRemoteUrl(cwd) {
  try {
    return execFileSync("git", ["-C", cwd, "remote", "get-url", "origin"], { encoding: "utf-8" }).trim();
  } catch { return undefined; }
}
```

Parse `owner/repo` from SSH (`git@...`) or HTTPS URLs, same logic as the OpenCode plugin.

### Hot-reload (`/clear`) behavior

When a user runs `/clear` in Copilot CLI, the extension process is terminated (SIGTERM) and a new one is forked. Since `process.ppid` (the CLI's PID) stays the same, the new instance writes to the same `<ppid>.json` file. The exit handler on the old instance deletes the file; the new instance recreates it. This is a brief gap (~1-2s) where the session file doesn't exist — acceptable.

### Testing considerations

Since the SDK is auto-resolved, unit testing the full extension requires mocking `@github/copilot-sdk/extension`. However, the pure logic (status mapping, file writing, git parsing, URL parsing) can be extracted into testable functions and tested independently. Consider a `plugins/copilot/test/` directory with tests for the interchange logic (same pattern as OpenCode's `test/interchange.test.ts`).

At minimum, `node --check plugins/copilot/src/extension.mjs` validates syntax and is added to the root `just check`.
