# US-opencode: OpenCode Harness Integration

## Theme

Track OpenCode coding sessions in Cortado, the same way we track Copilot CLI sessions. This requires an **OpenCode plugin** that publishes session state to the filesystem, and a **Cortado harness provider** that reads it.

The plugin uses OpenCode's typed event system (`@opencode-ai/plugin` -> `event` hook) to receive `session.status` events and write session state files. Cortado reads these files using a **generic harness provider** -- a reusable provider that any future coding agent can adopt with the same file format.

## How It Works

### Interchange format

A standard way for coding agent plugins to publish session state for Cortado.

**Location:** `~/.config/cortado/harness/` (consistent with Cortado's config directory convention)

**Structure:**
```
~/.config/cortado/harness/
  <pid>.json    # One file per active session, named by process PID
```

Each session file contains a flat JSON object with version, harness name, session identity (id, cwd, repo, branch), runtime info (pid, status, summary), and timestamps (last_active_at is required).

**Status values:** `working`, `idle` -- mapped from OpenCode's `session.status` event types (`busy` -> `working`, `idle` -> `idle`, `retry` -> `working` with retry info in summary). The interchange format also supports `question` and `approval` for future agents that have those concepts.

**File writes:** All writes use atomic write (write to temp file, then `rename`). This prevents Cortado from reading partially-written files.

**Lifecycle:**
- The agent plugin creates/updates the file on relevant events
- The plugin deletes the file on session end or process exit
- Cortado checks PID liveness and cleans up stale files (dead PID)
- Files with unrecognized `version` are logged and skipped

### OpenCode plugin

A TypeScript plugin in `plugins/opencode/` that hooks into OpenCode's event system via `@opencode-ai/plugin`. It uses the `event` hook to receive all bus events and translates `session.status` events into interchange file writes:

- `session.status` with `type: "busy"` -> status = `working`
- `session.status` with `type: "idle"` -> status = `idle`
- `session.status` with `type: "retry"` -> status = `working` (with retry info in summary)
- Session end / process exit -> delete state file

Project metadata (cwd, repo, branch) comes from the plugin input context (`directory`, `worktree`) and git commands.

Distributed as an npm package so users can add it to their `opencode.json`.

### Generic harness provider (Rust)

A `GenericProvider` in `src-tauri/src/feed/harness/generic.rs` that:
- Reads all `.json` files from the harness interchange directory
- Filters by `harness` field (e.g., `"opencode"`)
- Checks PID liveness via `libc::kill(pid, 0)`
- Cleans up stale session files
- Returns normalized `Vec<SessionInfo>`

Registered as feed type `opencode-session`, using `GenericProvider::new("opencode")`. Future agents using the interchange format only need one line of Rust to register.

### FSEvents watcher

Instead of polling the harness directory every 30s, Cortado uses the `notify` crate to watch for file changes via macOS FSEvents. This reduces detection latency from 30s to ~100ms and benefits all harness feeds (including Copilot, if migrated later).

A fallback timer (60s) ensures robustness if the watcher misses events.

## Design Decisions

- **Generic format over OpenCode-specific:** Future agents (Cursor, Aider, etc.) reuse the same format and provider.
- **Plugin in this repo:** Easier to develop alongside Cortado. Published to npm separately.
- **`~/.config/cortado/harness/`:** Consistent with Cortado's existing config directory (`~/.config/cortado/`). Always the production path, even when Cortado runs in dev mode -- the harness directory is a cross-app contract that agent plugins write to.
- **PID-based filenames:** Session files are named `<pid>.json`. The PID is always available immediately, unique per running process, and avoids rename dances when the agent's internal session ID isn't immediately known. The agent session ID is stored inside the JSON as a metadata field.
- **Copilot stays as-is:** Can migrate to the interchange format later once we're confident in the design.
- **Atomic writes:** Plugin writes to temp file then renames. Prevents Cortado from reading partial JSON.
- **Event hook, not SSE:** OpenCode exposes a local HTTP server with SSE (`GET /event`), but it's only active when the user starts OpenCode in server mode (not the default TUI mode). The plugin's `event` hook receives the same events in-process, works in all modes, and handles multi-instance natively (each plugin writes its own files). This avoids SSE connection management and server discovery complexity.
- **FSEvents over polling:** The `notify` crate provides near-instant file change detection. A debounce window (~200ms) prevents redundant polls from multiple FSEvents per atomic write. A fallback timer provides robustness.
- **No Question/Approval:** OpenCode's status model is `busy`/`idle`/`retry` -- it doesn't distinguish "AI thinking" from "asking user a question." The interchange format supports `question` and `approval` for future agents, but the OpenCode plugin won't use them.

## Task Sequencing

```
01-spec-interchange ─────────┐
                              ├──> 04-wire-feed-type ──> 06-update-specs ──┐
02-generic-provider ──────────┘                                            │
                                                                           ├──> 07-e2e-test
03-opencode-plugin ────────────────────────────────────────────────────────┘

05-fsevents-watcher (independent, can parallel with 02-04)
```

Task 01 is the prerequisite -- defines the format everything else depends on. Tasks 02 and 03 can be done in parallel after that (Rust provider and TS plugin are independent). Task 04 wires the provider into Cortado's feed system. Task 05 adds FSEvents watching (independent, benefits all harness feeds). Task 06 updates specs. Task 07 is the final end-to-end validation.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-spec-interchange.md` | Define the generic harness interchange format spec |
| 02 | `02-generic-provider.md` | Implement `GenericProvider` in Rust |
| 03 | `03-opencode-plugin.md` | Build the OpenCode plugin (TypeScript) |
| 04 | `04-wire-feed-type.md` | Register `opencode-session` feed type in Cortado |
| 05 | `05-fsevents-watcher.md` | Add FSEvents-based file watching for harness feeds |
| 06 | `06-update-specs.md` | Update specs and docs for the new feed type |
| 07 | `07-e2e-test.md` | End-to-end validation |

## Open Questions

- **npm package name:** `cortado-opencode`? `@cortado/opencode`? Check availability before publishing.
- **PID recycling:** `kill(pid, 0)` can't detect PID reuse after a crash. Same limitation as the Copilot provider. Accepted risk.
- **`notify` crate approval:** New dependency. Well-maintained (10M+ downloads), de facto standard for file watching in Rust. Needs user approval per AGENTS.md policy.
