---
status: pending
---

# Optional: config file watcher (hot-reload)

## Goal

Watch `~/.config/cortado/feeds.toml` for changes and reload feeds without restarting the app.

## Notes

- Currently config is loaded once at app launch.
- This would use a file watcher (e.g., `notify` crate) to detect changes.
- Needs debouncing (edits in progress shouldn't trigger rapid reloads).
- On reload: parse new config, diff against current feeds, add/remove/update feeds in the registry.
- Error handling: if the new config is invalid, keep the old config and surface an error in the UI.
- Alternative simpler approach: reload on panel open (check file mtime, only re-parse if changed).

## Naming note

- Azure DevOps feed type naming should be `ado-pr` (not `azdo-pr`) when this backlog item is eventually picked up alongside feed hot-reload support.
