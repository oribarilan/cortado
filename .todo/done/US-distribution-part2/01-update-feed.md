---
status: done
---

# Update Awareness Feed

## Goal

Add a built-in Cortado feed that checks for new versions using the Tauri updater plugin and surfaces updates as standard activities in the tray and panel.

## Acceptance criteria

- [ ] New feed type: `cortado-update` (internal, not user-configured via TOML)
- [ ] Implements the existing `Feed` trait — polls `latest.json`, produces `Activity` with standard fields
- [ ] When a new version is available, surfaces an activity: title "Cortado vX.Y.Z available"
- [ ] Activity fields: version, release notes (from `latest.json` `notes`), publish date
- [ ] Status field: `AttentionPositive` when update available — always shows as action-needed, no dismiss
- [ ] When app is up to date, feed produces no activities (empty feed, hidden from view)
- [ ] Expanding the activity shows release notes and an "Install update" action
- [ ] Clicking "Install update" triggers Tauri updater's download + install flow
- [ ] After install, prompts user to restart (or auto-relaunches per Tauri updater behavior)
- [ ] Feed is enabled by default, always visible in both tray and panel
- [ ] Feed polls infrequently (e.g., every 6 hours)

## Notes

- This is a **regular feed**, not an event feed. It follows the existing `Feed` trait pattern: poll, produce activities, use StatusKind for attention routing. No new architecture needed.
- The update activity intentionally has no dismiss option — when an update is available, it stays visible until the user installs it or the app is restarted at the new version.
- Unlike user-configured feeds, this one is built-in. The user can disable it in settings if desired.
- The Tauri updater plugin provides the check logic. The feed wraps it to surface the result as a standard Cortado activity.
- Consider: should the tray icon rollup reflect the update availability? Since the activity uses a real StatusKind, it naturally participates in rollup.
- Depends on the Tauri updater plugin being configured (task 03) and `latest.json` being published (CD pipeline).

## Architecture sketch

```
cortado-update feed (implements Feed trait)
  |
  |- poll(): checks latest.json via tauri-plugin-updater
  |- returns Vec<Activity> — one activity if update available, empty if current
  |- activity has StatusKind::AttentionPositive
  |- on user action: triggers updater.downloadAndInstall()
  |- on completion: triggers process.relaunch()
```

## Relevant files

- `src-tauri/src/feed/cortado_update.rs` (to create)
- `src-tauri/src/feed/mod.rs` — register the new feed type
- `src/App.tsx` — render the update activity, handle install action
- `src-tauri/tauri.conf.json` — updater plugin config
