---
status: done
---

# Frontend: per-feed notification override UI

## Goal

Replace the simple per-feed notification toggle with the two-toggle design: enable on/off + "Use specific notification settings for this feed" with expandable radio group.

## Acceptance criteria

- [ ] "Notifications" toggle preserved (on/off for the feed)
- [ ] "Use specific notification settings for this feed" toggle added below (default: off)
- [ ] When off: hint reads "Uses global mode (Worth Knowing)" (reflecting actual global mode name)
- [ ] When on: expandable radio group appears with the same 4 modes as the global selector
- [ ] Selecting "Specific kinds" in the per-feed radios reveals kind chips (same 4-chip design)
- [ ] Disabling "Notifications" toggle grays out and disables the feed-specific toggle
- [ ] Feed form saves the correct `FeedNotifyOverride` value (Off / Global / specific mode)
- [ ] Editing an existing feed with a per-feed mode override shows the correct state
- [ ] Expand/collapse animates smoothly

## Related files

- `src/settings/SettingsApp.tsx` -- feed edit form, per-feed notify toggle (lines ~1941-1956)
- `src/settings/SettingsApp.tsx` (line 49) -- `FeedConfigDto` type (local to settings component)
- `showcases/notification-mode-showcase.html` -- visual reference (per-feed section, variant H)
