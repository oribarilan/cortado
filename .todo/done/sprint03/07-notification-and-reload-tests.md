---
status: deferred
---

# Notification + config-change-warning test coverage

## Goal

Add deterministic tests that lock sprint03 behavior for per-feed notifications and config-change warning UX.

## Acceptance criteria

- [ ] Config parser tests cover `notify` default false, explicit true/false, and invalid-type failure.
- [ ] Notification diff tests verify only newly discovered activities emit events.
- [ ] Notification tests verify disabled feeds do not emit notifications.
- [ ] Startup seed tests verify notification suppression during seeding.
- [x] Config-change tracker tests verify mtime/size transitions and missing↔present file changes.
- [x] Tray warning integration path is covered for config-changed state without synchronous repoll regression.
- [ ] Tests remain isolated (no real OS notification delivery requirement, no external CLI/network side effects).
- [ ] `just check` passes.

## Notes

- Use test doubles around notification dispatch to keep tests deterministic.
- Prefer unit/integration tests around runtime boundaries over brittle UI assertions.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/src/feed/runtime.rs`
- `src-tauri/src/tray.rs`
- `src-tauri/src/main.rs`

## Progress notes

- Config-change warning checks are currently covered in `ConfigChangeTracker` unit tests and tray loop behavior wiring.
- Add explicit tray-level unit tests when menu composition is further refactored for testability.
