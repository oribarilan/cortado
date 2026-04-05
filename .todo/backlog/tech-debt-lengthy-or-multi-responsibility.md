---
status: pending
---

# Tech debt: lengthy or multi-responsibility files

## Goal

Refactor bloated files that have grown beyond a single responsibility or are hard to navigate due to sheer size.

## Flagged files

### 1. `src/settings/SettingsApp.tsx` — 1930 lines (HIGH)

**Problem**: The entire Settings window lives in a single component — 30+ state variables, 4 distinct sections (General, Notifications, Feeds, Focus), feed CRUD logic, inline form validation, hotkey recording, autostart management, notification permission handling, plugin setup UI, and utility functions.

**Suggestion**:
- Extract section components: `GeneralSection`, `NotificationsSection`, `FeedsSection`, `FocusSection`
- Extract `FeedEditor` component for the feed add/edit form
- Move utility functions (`parseDurationString`, `DurationInput`, `validateFeed`, `keyEventToShortcut`, `formatShortcut`) to shared utils

---

### 2. `src-tauri/src/settings_config.rs` — 1237 lines (HIGH)

**Problem**: Mixes 4 distinct responsibilities — feed config DTO/serialization, Tauri commands for feed CRUD, OpenCode plugin lifecycle (check/install/uninstall), Copilot CLI plugin lifecycle (check/install/uninstall), feed dependency checking, and feed test-polling.

**Suggestion**:
- Extract plugin management into `plugin_manager.rs` (or separate `plugins/opencode.rs` + `plugins/copilot.rs`)
- Keep feed config DTO/serialization and CRUD commands together

---

### ~~3. `src-tauri/src/feed/mod.rs` — 862 lines~~ DROPPED

Not worth splitting. ~430 lines are tests, and the remaining code (types, trait, registry, factories) is tightly coupled. Splitting would scatter related code across files with circular imports for no readability gain.

## Not flagged (long but cohesive)

These files are large (650–1700 lines) but single-responsibility and don't need splitting:

| File | Lines | Notes |
|------|-------|-------|
| `src/settings/settings.css` | 1711 | Well-sectioned, single purpose |
| `src-tauri/src/feed/ado_pr.rs` | 1455 | ~50% tests, single feed impl |
| `src-tauri/src/feed/harness/feed.rs` | 1002 | ~50% tests, single feed impl |
| `src-tauri/src/feed/github_actions.rs` | 983 | ~65% tests, single feed impl |
| `src-tauri/src/feed/github_pr.rs` | 873 | Single feed impl |
| `src-tauri/src/feed/http_health.rs` | 843 | Single feed impl |
| `src-tauri/src/feed/runtime.rs` | 686 | Feed polling runtime, cohesive |
| `src-tauri/src/terminal_focus/mod.rs` | 696 | Focus orchestration, borderline |
