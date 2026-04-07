# Task: Split SettingsApp.tsx into smaller components

## Context
`src/settings/SettingsApp.tsx` is ~2350 lines, well beyond the ~500 line guideline in AGENTS.md. It accumulates multiple responsibilities: general settings, notification settings, feed list/edit, focus/terminal settings, and several inline components (DurationInput, UserFilterField, shortcut recording).

**Value delivered**: Better maintainability, faster navigation, and adherence to the single-responsibility principle.

## Related Files
- `src/settings/SettingsApp.tsx`
- `src/settings/settings.css`

## Dependencies
- None

## Acceptance Criteria
- [ ] SettingsApp.tsx is under ~500 lines (routing/layout shell only)
- [ ] Each settings section is its own component file (GeneralSection, NotificationSection, FeedSection, FocusSection)
- [ ] Shared inline components extracted (DurationInput, UserFilterField)
- [ ] No behavioral changes -- pure refactor
- [ ] `just check` passes

## Scope Estimate
Medium

## Notes
Pre-existing debt, not introduced by any specific story. The file grew organically as settings sections were added.
