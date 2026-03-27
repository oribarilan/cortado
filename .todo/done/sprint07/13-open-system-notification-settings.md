---
status: done
---

# 13 -- Open system notification settings

## Goal

Add a "Configure in System Settings" button that opens macOS System Settings directly to Cortado's notification preferences.

## Alternatives considered

### A. URL scheme deep link via `open` command (Recommended)

macOS supports a URL scheme that opens System Settings to a specific app's notification preferences:

```
x-apple.systempreferences:com.apple.Notifications-Settings.extension?id=com.cortado.app
```

This can be opened with the existing `open_activity` Tauri command (which shells out to `open`) or a new dedicated command.

**Pros:** No new dependencies. Uses the same `open` pattern already in the codebase. Deep-links directly to Cortado's notification settings pane.
**Cons:** Relies on an undocumented Apple URL scheme that could change in future macOS versions. Works on Monterey+ (12+).

### B. Open generic Notifications pane (no app-specific targeting)

```
open "x-apple.systempreferences:com.apple.preference.notifications"
```

**Pros:** More stable URL.
**Cons:** Opens the general notifications list -- user has to scroll and find Cortado manually. Poor UX.

### C. AppleScript UI automation

Script System Settings to navigate to Cortado's entry.

**Pros:** Could be more precise.
**Cons:** Fragile, requires accessibility permissions, breaks across macOS versions. Bad idea.

## Recommendation

**Option A** -- URL scheme deep link. The app's bundle ID is `com.cortado.app`. This reuses the existing `open_activity` command pattern. If the URL scheme changes in a future macOS release, we update one string.

## Implementation

Frontend button calls `invoke("open_activity", { url })` with the deep link URL. Since `open_activity` validates `http://`/`https://` only, either:
- Add a new `open_system_settings` Tauri command that calls `open` without URL validation, OR
- Use the `shell:allow-open` capability to open non-http URLs.

Recommendation: new `open_notification_settings` command that constructs the URL server-side from the app's bundle identifier. This avoids hardcoding the bundle ID in the frontend and keeps the URL validation strict for activity links.

## Acceptance criteria

- [ ] "Configure in System Settings" button in the Notifications tab
- [ ] Opens macOS System Settings to Cortado's notification preferences
- [ ] Works on macOS 12+ (Monterey, Ventura, Sonoma, Sequoia)
- [ ] New Tauri command: `open_notification_settings`
- [ ] `just check` passes

## Relevant files

- `src-tauri/src/command.rs` -- new command
- `src/settings/SettingsApp.tsx` -- button in notifications tab
- `src-tauri/capabilities/settings.json` -- may need permission for the new command
