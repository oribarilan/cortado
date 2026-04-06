# Harness Interchange Format

Version: **1**

## Purpose

A standard file-based contract between coding agent plugins (OpenCode, Copilot, Cursor, etc.) and Cortado's generic harness provider. Any coding agent that writes session state in this format can be tracked by Cortado without agent-specific code on the Cortado side.

## Directory

```
~/.config/cortado/harness/
```

This path is fixed regardless of whether Cortado runs in dev or production mode. The harness directory is a cross-app contract -- agent plugins don't know about Cortado's dev/prod distinction.

Plugins must create the directory if it doesn't exist.

## File naming

Each active session is represented by a single JSON file named by the **process PID**:

```
~/.config/cortado/harness/<pid>.json
```

PID-based filenames are always immediately available, unique per running process, and avoid rename dances when the agent's internal session ID isn't known at startup. The agent's session ID is stored inside the JSON as a metadata field (`id`).

**PID strategy varies by integration type:**

- **In-process plugins** (e.g., OpenCode): use `process.pid` -- the plugin runs inside the agent process, so the agent's PID is directly available.
- **Plugin hooks (child process)** (e.g., Copilot CLI): use `process.ppid` -- the hook runs as a child process spawned by the agent, so the parent PID is the agent's PID. This ensures the interchange file is named after the agent process (for PID liveness checks and terminal focus), not the short-lived hook process.

## Schema

Each file is a flat JSON object:

| Field            | Type     | Required | Description |
|------------------|----------|----------|-------------|
| `version`        | `number` | yes      | Schema version. Currently `1`. |
| `harness`        | `string` | yes      | Agent identifier (e.g., `"opencode"`, `"cursor"`). Used by the provider to filter sessions. |
| `id`             | `string` | yes      | Agent's internal session ID. May equal the PID as a string if the real session ID isn't available yet. |
| `pid`            | `number` | yes      | OS process ID of the agent process. Must match the filename. |
| `cwd`            | `string` | yes      | Working directory of the session. |
| `status`         | `string` | yes      | One of: `working`, `idle`, `question`, `approval`. See [Status values](#status-values). |
| `last_active_at` | `string` | yes      | ISO 8601 timestamp of the last meaningful event. |
| `repository`     | `string` | no       | Repository name (e.g., `"owner/repo"`). |
| `branch`         | `string` | no       | Git branch name. |
| `summary`        | `string` | no       | Agent-generated session summary or current task description. |

### Examples

**OpenCode** (in-process plugin -- `pid` is the agent's own PID):

```json
{
  "version": 1,
  "harness": "opencode",
  "id": "sess_abc123",
  "pid": 42567,
  "cwd": "/Users/dev/repos/my-project",
  "status": "working",
  "last_active_at": "2026-03-15T14:30:00Z",
  "repository": "dev/my-project",
  "branch": "feature/new-thing",
  "summary": "Implementing user authentication"
}
```

**Copilot CLI** (plugin hook (child process) -- `pid` is the parent Copilot process via `process.ppid`):

```json
{
  "version": 1,
  "harness": "copilot",
  "id": "copilot-98765",
  "pid": 98765,
  "cwd": "/Users/dev/repos/another-project",
  "status": "question",
  "last_active_at": "2026-03-15T14:35:00Z",
  "repository": "dev/another-project",
  "branch": "main",
  "summary": "Refactoring database layer"
}
```

## Status values

| Value      | Meaning | Maps to `SessionStatus` |
|------------|---------|------------------------|
| `working`  | Agent is actively working (processing, executing tools, retrying) | `Working` |
| `idle`     | Session is idle (agent finished, waiting for user input) | `Idle` |
| `question` | Agent asked the user a question | `Question` |
| `approval` | Agent is waiting for tool/action approval | `Approval` |

Not all agents support all statuses. OpenCode uses only `working` and `idle`. Copilot CLI (via the Cortado plugin) uses `working` and `question`.

Any unrecognized status value is treated as `Unknown` (mapped to `Idle` in the UI).

## Atomic writes

All file writes **must** use atomic write: write to a temporary file in the same directory, then `rename()` to the final path. This prevents Cortado from reading partially-written JSON.

```
# Pseudocode
write(~/.config/cortado/harness/.42567.json.tmp, json_content)
rename(~/.config/cortado/harness/.42567.json.tmp, ~/.config/cortado/harness/42567.json)
```

## Lifecycle

### Plugin responsibilities

1. **Create** the session file on the first relevant event (session start, first status change).
2. **Update** the file on each status change or meaningful event. Always update `last_active_at`.
3. **Delete** the file when the session ends normally.
4. **Cleanup on exit:** Register handlers for process exit signals (`SIGTERM`, `SIGINT`, `exit`) to delete the file synchronously. Use synchronous I/O in exit handlers (async is not available).

### Cortado responsibilities

1. **Read** all `.json` files from the harness directory.
2. **Filter** by `harness` field to select sessions for the configured feed type.
3. **Check PID liveness** via `kill(pid, 0)`. Skip sessions with dead PIDs.
4. **Clean up stale files:** Delete session files whose PID is no longer alive. This handles crashes where the plugin couldn't clean up.
5. **Skip unknown versions:** If a file has a `version` value that the provider doesn't recognize, log a warning and skip it. Do not error.
6. **Handle gracefully:** Missing directory (return empty), malformed JSON (log warning, skip), I/O errors on individual files (log warning, skip).

## Deduplication and status priority

When multiple sessions share the same `cwd` (e.g., two OpenCode instances in the same repo), Cortado consolidates them into a single activity. The winner is selected by **status urgency**, not recency:

| Priority | Statuses              | Rationale |
|----------|-----------------------|-----------|
| Highest  | `question`, `approval`| User action required -- must surface |
| Medium   | `working`             | Agent is active -- informational |
| Lowest   | `idle`, unknown       | Nothing happening -- least urgent |

Within the same priority tier, the session with the most recent `last_active_at` wins.

When dedup collapses multiple sessions, the surviving activity gets a **stable CWD-derived ID** (not the original session ID). This prevents the UI row from jumping when the winning session changes between polls. Single-session CWDs keep their original session ID.

### Implications for plugin authors

- **Set status accurately.** Status drives which session surfaces in the UI when multiple exist. A session stuck on `working` when it's actually waiting for a question will hide a sibling's `question` status.
- **Always update `last_active_at`.** It's the tiebreaker within the same priority tier.
- **`idle` is the "I'm done" signal.** Transition to `idle` promptly when the agent finishes -- don't leave stale `working` statuses that could mask a sibling's attention-needed state.

## Versioning

The `version` field is required. The current version is `1`.

When Cortado encounters a file with an unrecognized version:
- Log a warning with the file path and version number.
- Skip the file (do not include it in results).
- Do not error or crash.

This allows forward compatibility: newer plugins can write version 2 files while older Cortado versions safely ignore them.

## PID liveness and recycling

PID liveness is checked via `kill(pid, 0)` (Unix) which returns success if the process exists and the caller has permission to signal it.

**Known limitation:** PID recycling. After a crash, the OS may reassign the same PID to an unrelated process. `kill(pid, 0)` will return true for the recycled PID, causing Cortado to keep a stale session file until the recycled process also exits. This is an accepted risk -- PID recycling is rare on modern macOS (PIDs go up to ~99999 before wrapping) and the impact is a briefly stale session entry.

## Provider registration

To track a new agent type in Cortado:

1. Write a plugin or hook integration for the agent that publishes session state in this format with a unique `harness` name. For in-process plugins (like OpenCode), use `process.pid`; for plugin hooks (child processes, like Copilot CLI), use `process.ppid`.
2. Register the provider in Cortado: `GenericProvider::new("agent-name")` -- one line of Rust.
3. Add a feed type entry (e.g., `"agent-session"`) in the feed catalog.

No agent-specific parsing logic is needed on the Cortado side.
