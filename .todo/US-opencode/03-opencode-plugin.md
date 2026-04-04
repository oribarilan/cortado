---
status: done
---

# Build OpenCode plugin

## Goal

Create a TypeScript plugin for OpenCode that publishes session state to the harness interchange format. When a user installs this plugin, their OpenCode sessions become visible in Cortado.

## Acceptance criteria

- [ ] `plugins/opencode/` directory exists with a complete npm package
- [ ] `package.json` with `@opencode-ai/plugin` as peer dependency
- [ ] Plugin entry point exports a properly typed plugin function
- [ ] Uses the `event` hook to receive bus events and track `session.status`:
  - `session.status` with `type: "busy"` -> status = `working`
  - `session.status` with `type: "idle"` -> status = `idle`
  - `session.status` with `type: "retry"` -> status = `working` (with retry attempt/message in summary)
- [ ] Writes session state to `~/.config/cortado/harness/<pid>.json` (PID-based filename)
- [ ] All file writes are atomic (write to temp file, then `fs.renameSync`)
- [ ] Creates the interchange directory if it doesn't exist
- [ ] Populates all interchange fields: version, harness (`"opencode"`), id (OpenCode session ID when available, else PID), pid, cwd, repository, branch, status, summary, last_active_at
- [ ] Deletes the session file on session end
- [ ] Cleans up on process exit (SIGTERM, SIGINT, uncaught exceptions) using `fs.unlinkSync`
- [ ] Unit tests covering: file writing, status mapping, atomic writes, cleanup
- [ ] `plugins/opencode/justfile` with `check`, `test`, `format`, `build` recipes
- [ ] `.gitignore` for `node_modules/` and build output
- [ ] `tsconfig.json` and build configuration
- [ ] README with installation and usage instructions
- [ ] Package can be built and is ready for npm publish

## Notes

### OpenCode plugin system

Users add the plugin to their `opencode.json`:
```json
{
  "plugin": ["cortado-opencode"]
}
```

Or for local development:
```json
{
  "plugin": ["./.opencode/plugins/cortado.ts"]
}
```

### Plugin contract (`@opencode-ai/plugin`)

The plugin exports a function receiving `PluginInput` with:
- `client` -- typed OpenCode SDK client
- `directory` -- project directory path
- `worktree` -- git worktree root
- `$` -- Bun shell for running commands

The plugin returns `Hooks`, which includes:
- `event({ event })` -- receives ALL bus events (the one we use)

### Event mapping

OpenCode emits `session.status` events with a discriminated union:
```typescript
type SessionStatusInfo =
  | { type: "idle" }
  | { type: "busy" }
  | { type: "retry", attempt: number, message: string, next: number }
```

These map directly to interchange status values.

### Session metadata

- `cwd` from `directory` or `worktree` (plugin input)
- `repository` / `branch` from `$ git` commands (via the `$` shell helper)
- `pid` from `process.pid`
- `id` from session context (the `sessionID` field in status events). If unavailable before first event, use `process.pid` as the `id` field value and update it in-place when session events arrive. The *filename* is always `<pid>.json` -- no renames needed.

### Cleanup

Register handlers for `process.on('exit')`, `process.on('SIGTERM')`, `process.on('SIGINT')` to delete the session file. Use `fs.unlinkSync` in exit handlers (async not available).

### Justfile

The plugin has its own `justfile` with recipes for `check`, `test`, `format`, `build`. The root `justfile` cascades into it (see task 04).
