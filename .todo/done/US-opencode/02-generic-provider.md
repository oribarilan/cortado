---
status: done
---

# Implement GenericProvider

## Goal

Build a Rust harness provider that reads session state from the generic interchange format. This provider is agent-agnostic -- it reads any session file from the interchange directory and filters by harness name.

## Acceptance criteria

- [ ] `src-tauri/src/feed/harness/generic.rs` exists and implements `HarnessProvider`
- [ ] Reads all `.json` files from `~/.config/cortado/harness/`
- [ ] Parses interchange JSON into `SessionInfo`
- [ ] Filters sessions by `harness` field matching the provider's configured name
- [ ] Checks PID liveness via `libc::kill(pid, 0)` -- skips dead sessions
- [ ] Cleans up stale session files (dead PID) on read
- [ ] Skips files with unrecognized `version` (logs warning)
- [ ] Handles missing/malformed files gracefully (logs warning, skips)
- [ ] Handles missing interchange directory gracefully (returns empty)
- [ ] `harness/mod.rs` updated with `pub mod generic;`
- [ ] Unit tests cover: valid file parsing, stale cleanup, malformed files, missing directory, harness filtering, unknown version skipping

## Notes

Follow the same patterns as `copilot.rs`:
- `GenericProvider::new(harness_name: &str)` constructor
- `MAX_SESSIONS` cap on discovery
- Return empty vec if directory doesn't exist

Key difference from `CopilotProvider`: status inference happens in the *plugin*, not in Cortado. The provider just reads the `status` field from JSON.

Use `dirs::home_dir()` + hardcoded `.config/cortado/harness/` for the directory path. Do NOT use `app_env::config_dir()` -- the harness directory is a cross-app contract that agent plugins write to, and plugins don't know about Cortado's dev/prod distinction. Both dev and production Cortado read harness files from the same path (`~/.config/cortado/harness/`).

The provider should be reusable -- `GenericProvider::new("opencode")` for OpenCode, `GenericProvider::new("cursor")` for a hypothetical Cursor plugin, etc.
