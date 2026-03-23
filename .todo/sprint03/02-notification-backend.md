---
status: pending
---

# Desktop notification backend wiring

## Goal

Establish a reliable backend path for desktop notifications so feed runtime code can emit OS notifications safely.

## Acceptance criteria

- [ ] Notification backend is wired in Rust with Tauri v2-compatible APIs for desktop.
- [ ] Required capability permission(s) for notifications are added under `src-tauri/capabilities/`.
- [ ] Notification dispatch path is encapsulated behind a small helper/module (not scattered across feed code).
- [ ] Desktop payload contract is documented in code/tests (title/body and optional sound only).
- [ ] Dispatch failures are surfaced to logs/diagnostics but do not fail feed polling.
- [ ] Existing tray/menu behavior is unchanged by this task.
- [ ] `just check` passes.

## Notes

- Keep this task focused on backend wiring and permissions, not feed diff logic.
- Mobile-only notification features (channels/actions/scheduling) are out of scope.

## Relevant files

- `src-tauri/src/main.rs`
- `src-tauri/src/feed/` (new helper module or existing runtime module)
- `src-tauri/Cargo.toml` (if needed)
- `src-tauri/capabilities/core.json`
