---
status: pending
---

# Fix hide_empty_feeds not updating in tray

## Goal

The `hide_empty_feeds` setting in the tray (`App.tsx`) is read once during bootstrap and never re-fetched. Changing it in Settings has no effect until restart. Fix this to match the pull-on-show pattern used by `show_priority_section` in the main screen.

## Acceptance criteria

- [ ] Changing `hide_empty_feeds` in Settings takes effect in the tray without restart
- [ ] Use pull-on-show pattern (re-fetch on panel show) or listen for `appearance-changed`/`settings-changed` event

## Notes

- `App.tsx:30` — `hideEmptyFeeds` state initialized once in `bootstrap()` useEffect (line 159-161)
- `MainScreen` already does pull-on-show for `show_priority_section` — follow the same pattern
- This is a standalone bugfix, not related to the restart mechanism, but grouped here since it's a settings reactivity gap
