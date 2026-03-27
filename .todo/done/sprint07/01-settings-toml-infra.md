---
status: done
---

# 01 — settings.toml infrastructure

## Goal

Create the `settings.toml` config system for global app preferences. This is the foundation for notification settings and future global preferences (autostart could migrate here later).

## Context

Currently, Cortado has no global settings file. Feed config lives in `feeds.toml`, and autostart is managed directly by the OS via `tauri-plugin-autostart`. Sprint 07 introduces `~/.config/cortado/settings.toml` as the canonical home for app-level preferences.

## Acceptance criteria

- [ ] `AppSettings` Rust struct defined with serde Serialize/Deserialize
- [ ] Parser reads `~/.config/cortado/settings.toml`, returns defaults if file doesn't exist
- [ ] Writer serializes `AppSettings` back to TOML, creates parent dirs if needed
- [ ] Backup before overwrite (same pattern as `feeds.toml`)
- [ ] Tauri commands: `get_settings` and `save_settings` exposed to frontend
- [ ] Capability permissions added for new commands
- [ ] `AppSettings` initially contains a `notifications` section (populated in task 03)
- [ ] `just check` passes

## Notes

- Follow the same pattern as `feeds.toml` config: load at startup, persist to file.
- The `AppSettings` struct should be extensible — use `#[serde(default)]` liberally so missing fields get defaults.
- **Reload behavior**: The master notification toggle takes effect immediately (watched via `Arc<RwLock<...>>` or similar). All other notification settings take effect on next poll cycle. This means `AppSettings` needs a shared-state mechanism for at least the `enabled` flag.

## Relevant files

- `src-tauri/src/feed/config.rs` — reference for TOML parsing pattern
- `src-tauri/src/settings_config.rs` — existing settings DTO (for feed config in settings UI)
- `src-tauri/src/command.rs` — Tauri commands
- `src-tauri/capabilities/settings.json` — capability permissions
