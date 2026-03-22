---
status: pending
---

# Shell feed stub

## Goal

Implement a `ShellFeed` struct that satisfies the `Feed` trait. Returns hardcoded output (no real command execution). Proves the trait works for a single-activity, user-configured-fields feed type.

## Acceptance criteria

- [ ] `src-tauri/src/feed/shell.rs` exists with `ShellFeed` struct
- [ ] Implements `Feed` trait with hardcoded poll data
- [ ] `provided_fields()` returns a single user-configured field (from config)
- [ ] `poll()` returns exactly one activity with the configured field populated
- [ ] Can be constructed from a `FeedConfig` (takes command, label from config)
- [ ] `just check` passes

## Notes

- This is a stub. No process spawning, no real command execution.
- A shell feed always has exactly one activity (the command itself).
- Field definition comes from the config — the user decides what the field is called and labeled. For the stub, default to a field named "output" with label "Output".
- The constructor should extract `command` from the config's type-specific table and return an error if missing.

## Relevant files

- `src-tauri/src/feed/shell.rs` — new file
- `src-tauri/src/feed/mod.rs` — add `pub mod shell;`
