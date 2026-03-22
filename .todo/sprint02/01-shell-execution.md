---
status: pending
---

# Real shell feed execution

## Goal

Replace the shell feed stub with real command execution so shell feeds produce live activity values from user-configured commands.

## Acceptance criteria

- [ ] `ShellFeed::poll()` executes the configured command via shell (`sh -c`) and captures output.
- [ ] Command execution runs via async process APIs (no blocking process calls on async runtime threads).
- [ ] Command success maps stdout to the configured field type:
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
- [ ] Non-zero exits return an error containing exit context and stderr when available.
- [ ] Command spawn failures (e.g., shell unavailable) return meaningful poll errors.
- [ ] Poll path enforces a per-poll command timeout of 10s; timeout surfaces as a feed poll error.
- [ ] Existing feed identity, provided field metadata, and single-activity shape remain unchanged.
- [ ] `just check` passes.

## Notes

- Keep this task focused on shell feed behavior only; do not introduce scheduler changes here.
- If `tokio` needs process support features, modify existing dependency config rather than adding a new crate.

## Relevant files

- `src-tauri/src/feed/shell.rs`
- `src-tauri/Cargo.toml` (only if tokio feature adjustment is required)
