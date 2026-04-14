---
status: pending
---

# Update feed: Enter action in panel

## Goal

Every activity has exactly one primary action, triggered by Enter in the panel and by clicking the action link in the tray. This already works for PR feeds (open URL), agent feeds (focus terminal), and Actions feeds (open run URL). But for the auto-update feed, pressing Enter in the panel does nothing -- the `install_update` Tauri command fails silently (the catch block in `installUpdate` swallows the error with no user feedback).

Make the update feed's Enter action work reliably in the panel, matching the tray behavior.

## Current state

- **Tray**: Works. Expanding an update activity shows an "Install update" / "Update plugin" button. Clicking it calls `invoke("install_update")` or `invoke("install_opencode_plugin")`.
- **Panel**: Broken. `openFocusedActivity` correctly dispatches to `installUpdate()` / `installPluginUpdate()`, but the backend command can fail (e.g., "updater not available" in dev, or download failure), and the catch block silently resets `installing` to false. The user sees nothing.
- **DetailPane**: Shows the "Install update" button, which has the same silent-failure problem.

## Design

1. **Surface errors**: When `invoke("install_update")` or `invoke("install_opencode_plugin")` fails, show the error to the user (e.g., a brief inline error message in the detail pane) instead of swallowing it.
2. **Consistent feedback**: Both panel and tray should show "Installing..." while the command runs and an error message if it fails.
3. **No new actions**: Keep it to one action per activity. Don't add a separate "view release" action.

## Acceptance criteria

- [ ] Pressing Enter on an app-update activity in the panel triggers install (same as tray button)
- [ ] Pressing Enter on a plugin-update activity triggers plugin update (same as tray button)
- [ ] If the install/update command fails, the user sees an error message (not silent swallow)
- [ ] The "Installing..." / "Updating..." loading state is visible in the detail pane

## Related files

- `src/main-screen/MainScreenApp.tsx` -- `installUpdate`, `installPluginUpdate`, `openFocusedActivity`, `DetailPane`
- `src/App.tsx` -- tray install buttons (reference implementation)
- `src-tauri/src/command.rs` -- `install_update` command (line 282)
