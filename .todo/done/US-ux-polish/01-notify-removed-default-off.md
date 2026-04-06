---
status: done
---

# Default "Removed activities" notification toggle to OFF

## Goal

Change the default value of `notify_removed_activities` from `true` to `false`. Activities disappearing (e.g., a PR being merged) is usually expected and not worth a notification by default.

## Changes needed

| File | Location | Change |
|------|----------|--------|
| `src-tauri/src/app_settings.rs` | `Default` impl | `notify_removed_activities: true` → `false` |
| `src/settings/SettingsApp.tsx` | initial state (~L267) | `notify_removed_activities: true` → `false` |
| `src/settings/SettingsApp.tsx` | reset defaults (~L1989) | `notify_removed_activities: true` → `false` |

## Tests to update

- `src-tauri/src/app_settings.rs` -- default assertion tests
- `src-tauri/src/notification/integration_tests.rs` -- any tests asserting the default is `true`

## Acceptance criteria

- [ ] Default for `notify_removed_activities` is `false` in backend and frontend
- [ ] Users who previously explicitly enabled it keep their setting
- [ ] `just check` passes

## Notes

- Since `settings.toml` only persists values that differ from defaults, users who never touched the toggle will get the new default on next launch.
