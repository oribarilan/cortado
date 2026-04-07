---
status: pending
---

# Frontend: global notification mode selector

## Goal

Update the Notifications tab in settings to show the new mode radio group with descriptions, and update the Specific Kinds chip row to use 4 chips (merging Attention+/-).

## Acceptance criteria

- [ ] Radio group shows: Worth Knowing, Need Attention, All changes, Specific kinds
- [ ] Each radio has a description line (see showcase)
- [ ] Worth Knowing is pre-selected when mode is the default
- [ ] Selecting "Specific kinds" reveals kind chip toggles
- [ ] Kind chips: Attention, Waiting, Running, Idle (4 chips, not 5)
- [ ] Attention chip maps to both `attention_positive` and `attention_negative` in the saved settings
- [ ] `saveNotifSettings` sends the correct mode value to the backend
- [ ] Old `escalation_only` UI option removed entirely

## Related files

- `src/settings/SettingsApp.tsx` -- notification settings section (lines ~1300-1484)
- `src/settings/SettingsApp.tsx` (line 16) -- `NotificationSettings` type (local to settings component)
- `showcases/notification-mode-showcase.html` -- visual reference
