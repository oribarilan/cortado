---
status: done
---

# Real shell feed execution

## Goal

Replace the shell feed stub with real command execution so shell feeds produce live activity values from user-configured commands.

## Acceptance criteria

- [x] `ShellFeed::poll()` executes the configured command via shell (`sh -c`) and captures output.
- [x] Command execution runs via async process APIs (no blocking process calls on async runtime threads).
- [x] Command success maps stdout to the configured field type:
  - `text` → raw trimmed stdout
  - `number` → parsed numeric value (error if parse fails)
  - `url` → raw trimmed stdout as URL value
  - `status` → status value with deterministic severity mapping:
    - `ok|pass|passing|success|healthy` → `success`
    - `warn|warning` → `warning`
    - `err|error|fail|failing|critical` → `error`
    - `pending|running|in_progress` → `pending`
    - any other value → `neutral`
    - matching is case-insensitive; emitted status `value` is the trimmed stdout
- [x] Non-zero exits return an error containing exit context and stderr when available.
- [x] Command spawn failures (e.g., shell unavailable) return meaningful poll errors.
- [x] Poll path enforces a per-poll command timeout of 10s; timeout surfaces as a feed poll error.
- [x] Existing feed identity, provided field metadata, and single-activity shape remain unchanged.
- [x] `just check` passes.

## Notes

- Keep this task focused on shell feed behavior only; do not introduce scheduler changes here.
- If `tokio` needs process support features, modify existing dependency config rather than adding a new crate.

## Relevant files

- `src-tauri/src/feed/shell.rs`
- `src-tauri/Cargo.toml` (only if tokio feature adjustment is required)
