---
status: pending
---

# Split bloated files: SettingsApp.tsx and settings_config.rs

## Goal

Refactor the two largest multi-responsibility files into smaller, focused modules.

## File 1: `src/settings/SettingsApp.tsx` -- 2009 lines (HIGH)

**Problem**: The entire Settings window lives in a single component -- 30+ state variables, 4 distinct sections (General, Notifications, Feeds, Focus), feed CRUD logic, inline form validation, hotkey recording, autostart management, notification permission handling, plugin setup UI, and utility functions.

**Suggestion**:
- Extract section components: `GeneralSection`, `NotificationsSection`, `FeedsSection`, `FocusSection`
- Extract `FeedEditor` component for the feed add/edit form
- Move utility functions (`parseDurationString`, `DurationInput`, `validateFeed`, `keyEventToShortcut`, `formatShortcut`) to shared utils

## File 2: `src-tauri/src/settings_config.rs` -- 1237 lines (HIGH)

**Problem**: Mixes distinct responsibilities -- feed config DTO/serialization, Tauri commands for feed CRUD, OpenCode plugin lifecycle (check/install/uninstall), Copilot CLI plugin lifecycle (check/install/uninstall), feed dependency checking, and feed test-polling.

**Suggestion**:
- Extract plugin management into `plugin_manager.rs` (or separate `plugins/opencode.rs` + `plugins/copilot.rs`)
- Keep feed config DTO/serialization and CRUD commands together

## Acceptance criteria

- [ ] `SettingsApp.tsx` is split into section components, each in its own file
- [ ] Shared utility functions are extracted to a common location
- [ ] `settings_config.rs` plugin management code is extracted to a separate module
- [ ] No behavioral changes -- pure refactor
- [ ] `just check` passes
