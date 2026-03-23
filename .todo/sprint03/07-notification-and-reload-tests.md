---
status: pending
---

# Notification + reload test coverage

## Goal

Add deterministic tests that lock sprint03 behavior for per-feed notifications and full config reload.

## Acceptance criteria

- [ ] Config parser tests cover `notify` default false, explicit true/false, and invalid-type failure.
- [ ] Notification diff tests verify only newly discovered activities emit events.
- [ ] Notification tests verify disabled feeds do not emit notifications.
- [ ] Startup/reload seed tests verify notification suppression during seeding.
- [ ] Runtime reload tests verify add/remove/update feed config effects after reload.
- [ ] Reload failure tests verify last-known-good runtime remains active.
- [ ] Tray reload integration path is covered (or equivalent integration-level runtime test) without synchronous repoll regression.
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
