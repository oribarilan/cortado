---
status: done
---

# Harness provider trait + Copilot implementation

## Goal

Create a generic `HarnessProvider` trait that abstracts how coding harness sessions are discovered and their status inferred. Implement the Copilot CLI provider as the first concrete implementation. This enables future providers (Claude Code, etc.) with identical user-facing behavior but different internal logic.

A **coding harness** is a terminal-based AI coding agent (e.g., GitHub Copilot CLI, Claude Code). Each harness has sessions that can be running, idle, or waiting for input.

## Architecture

```
src-tauri/src/feed/harness/
  mod.rs          # HarnessProvider trait, SessionInfo, SessionStatus
  copilot.rs      # CopilotProvider: reads ~/.copilot/session-state/
  feed.rs         # HarnessFeed: generic Feed impl over any provider (task 02)
```

### Shared types (provider-agnostic)

```rust
/// Status of a harness session, shared across all providers.
enum SessionStatus {
    /// Agent is actively working (processing, executing tools).
    Working,
    /// Agent asked the user a question.
    Question,
    /// Agent requested tool approval from user.
    Approval,
    /// Session is idle (agent finished, waiting for user input).
    Idle,
    /// Status could not be determined.
    Unknown,
}

/// A discovered harness session, provider-agnostic.
struct SessionInfo {
    /// Unique session identifier.
    id: String,
    /// Working directory of the session.
    cwd: String,
    /// Repository name (e.g., "oribarilan/cortado"). Optional.
    repository: Option<String>,
    /// Git branch name. Optional.
    branch: Option<String>,
    /// Current session status.
    status: SessionStatus,
    /// PID of the owning process (for terminal focus).
    pid: u32,
}
```

### Provider trait

```rust
/// Abstracts harness session discovery and status inference.
trait HarnessProvider: Send + Sync {
    /// Human-readable name of the harness (e.g., "Copilot", "Claude Code").
    fn harness_name(&self) -> &str;

    /// Feed type identifier (e.g., "copilot-session", "claude-code-session").
    fn feed_type(&self) -> &str;

    /// Discover all active sessions.
    fn discover_sessions(&self) -> Result<Vec<SessionInfo>>;
}
```

### `CopilotProvider`

Implements `HarnessProvider` for GitHub Copilot CLI:

- **Session discovery**: Glob `~/.copilot/session-state/*/inuse.*.lock`, check PID liveness via `kill(pid, 0)`
- **Metadata**: Parse `workspace.yaml` (id, cwd, repo, branch)
- **Status inference**: Read last line of `events.jsonl`, map event type to `SessionStatus`
- **PID**: Extract from lock file name/contents

All Copilot-specific parsing logic lives in `copilot.rs`. The shared types in `mod.rs` know nothing about Copilot's file formats.

## Parsing logic (Copilot-specific)

### `workspace.yaml`

- Parse with `serde-saphyr` (pure Rust, panic-free YAML parser).
- All fields except `id` are optional.
- If missing or malformed, skip the session.

### `events.jsonl` — last event

- Read last non-empty line via reverse-seek (never read full file).
- Parse JSON, extract `type` field.
- For `assistant.message` events, check `data.toolRequests`:
  - Any `name == "ask_user"` -> `Question`
  - Other tool requests -> `Approval`
  - No tool requests -> `Idle` (plain text response, user's turn)
- Map event type to `SessionStatus` per table in `main.md`.
- Missing file -> `Unknown`.

### Lock file + PID liveness

- Glob `inuse.*.lock` files in each session directory.
- Extract PID from filename pattern `inuse.<PID>.lock`.
- `libc::kill(pid, 0)` -> alive or dead.
- Dead PID -> skip session.

### Performance

- Complete discovery <50ms for ~12 active sessions.
- Never read full `events.jsonl`.
- Use `std::fs` (sync I/O — fast local files).

## Acceptance criteria

- [ ] `src-tauri/src/feed/harness/mod.rs` with `SessionStatus`, `SessionInfo`, `HarnessProvider` trait
- [ ] `src-tauri/src/feed/harness/copilot.rs` with `CopilotProvider`
- [ ] `CopilotProvider::discover_sessions()` handles: lock file discovery, PID liveness, YAML parsing, last-event reading
- [ ] All parsing is resilient: unknown events -> `Unknown`, missing fields -> skip, malformed files -> skip
- [ ] Unit tests: YAML parsing (valid, missing fields, malformed), event type mapping (all types), last-line reading, PID extraction
- [ ] Add `serde-saphyr` to `Cargo.toml` — approved for `workspace.yaml` parsing (pure Rust, panic-free, no C deps)
- [ ] Add `libc` to `Cargo.toml` — needed for `kill(pid, 0)` PID liveness check
- [ ] `just check` passes

## Notes

- The provider trait is intentionally minimal. Future providers (Claude Code) will have completely different file formats, directory structures, and lock mechanisms — that's fine, they just return `Vec<SessionInfo>`.
- `SessionInfo` is the contract between providers and the generic feed. Keep it lean.
- The `~/.copilot/session-state/` path should be configurable in `CopilotProvider` (for testing).

## Relevant files

- `src-tauri/src/feed/harness/mod.rs` — new (trait + shared types)
- `src-tauri/src/feed/harness/copilot.rs` — new (Copilot provider)
- `src-tauri/Cargo.toml` — add `serde-saphyr`, `libc`

