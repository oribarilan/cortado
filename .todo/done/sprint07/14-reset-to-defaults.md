---
status: done
---

# 14 -- Reset to default button

## Goal

Add a "Reset to defaults" button in the Notifications settings tab that restores all notification settings to their default values.

## Alternatives considered

### A. Frontend-only reset using hardcoded defaults (Recommended)

Define the default `NotificationSettings` object in the frontend, and on click, call `save_settings` with those defaults.

```ts
const DEFAULT_NOTIFICATION_SETTINGS = {
  enabled: true,
  mode: "all",
  delivery: "grouped",
  notify_new_activities: true,
  notify_removed_activities: true,
};
```

**Pros:** Simple. No new backend code. The defaults are already defined in Rust (`NotificationSettings::default()`), and duplicating 5 fields in TS is trivial.
**Cons:** Defaults are defined in two places (Rust and TS). If they drift, the reset button produces different values than a missing `[notifications]` section.

### B. Backend `reset_settings` command that writes `NotificationSettings::default()`

Add a Tauri command that resets just the notifications section to Rust defaults and returns them.

**Pros:** Single source of truth for defaults.
**Cons:** More code for something that saves 5 lines of duplication.

### C. Backend `get_default_settings` query command

Return the Rust defaults without writing, let the frontend populate the form and save on user confirmation.

**Pros:** Clean separation. User sees the reset before committing.
**Cons:** Two round-trips (get defaults, then save). Over-engineered.

## Recommendation

**Option A** -- frontend-only with hardcoded defaults. The defaults are simple (5 scalar fields), unlikely to change, and if they do drift, the Rust `#[serde(default)]` will correct them on next load anyway. A confirmation dialog before resetting prevents accidental data loss.

## Acceptance criteria

- [ ] "Reset to defaults" button at the bottom of the Notifications tab
- [ ] Confirmation prompt before resetting ("Reset notification settings to defaults?")
- [ ] Resets: enabled=true, mode=all, delivery=grouped, notify_new=true, notify_removed=true
- [ ] Persists the reset via `save_settings`
- [ ] UI updates immediately to show default values
- [ ] `just check` passes

## Relevant files

- `src/settings/SettingsApp.tsx` -- button + reset logic
