---
status: done
---

# 02 — tauri-plugin-notification setup

## Goal

Install and configure `tauri-plugin-notification` so Cortado can send macOS Notification Center notifications. Verify permission flow works.

## Context

Cortado currently has no notification capability. The official Tauri notification plugin (`tauri-plugin-notification`) provides cross-platform notification support with permission handling, action callbacks, and sound support.

Plugin source: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/notification

## Acceptance criteria

- [ ] `tauri-plugin-notification` added to `Cargo.toml` (Rust) and `package.json` (JS API)
- [ ] Unused `system-notification` crate removed from `Cargo.toml`
- [ ] Plugin initialized in `main.rs` builder (`.plugin(tauri_plugin_notification::init())`)
- [ ] Capability permissions added (`notification:default` or specific allows)
- [ ] Permission request flow works — app requests notification permission on first use
- [ ] Smoke test: can send a basic notification from Rust side
- [ ] `just check` passes

## Notes

- Check plugin version compatibility with the Tauri version used in the project.
- The plugin handles macOS permission prompts automatically, but we should handle the "denied" case gracefully (show a message in settings UI).
- Consider when to request permission: on first notification attempt? On app startup? When user enables notifications in settings?
- The JS API (`@tauri-apps/plugin-notification`) is needed for the settings UI to check permission status.

## Relevant files

- `src-tauri/Cargo.toml` — add Rust dependency
- `package.json` — add JS dependency
- `src-tauri/src/main.rs` — plugin init
- `src-tauri/capabilities/core.json` or `settings.json` — permissions
