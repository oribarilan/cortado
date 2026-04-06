---
status: done
---

# 07 -- Settings UI: Theme + Text Size Controls

## Goal

Add appearance controls to the Settings > General section so users can switch theme and text size.

## Acceptance Criteria

- [ ] "Appearance" subsection visible in Settings > General
- [ ] Theme: segmented control with labels "Light", "Dark", "System" -- default "System"
- [ ] Text size: segmented control with labels "S", "M", "L", "XL" -- default "M"
- [ ] Changes save immediately on selection (no save button)
- [ ] Changes emit `appearance-changed` event (via `save_settings`) so all windows update live
- [ ] Selections persist across app restarts
- [ ] Reusable `<SegmentedControl>` component created (could be used for future controls)
- [ ] Segmented control styled consistently with the design token system
- [ ] `just check` passes

## Implementation Notes

- The segmented control should accept: `options: { label: string, value: string }[]`, `value: string`, `onChange: (value: string) => void`.
- Place the Appearance subsection above the existing General controls (autostart, menubar icon, priority section).
- The `SegmentedControl` component can live in `src/settings/` or `src/shared/` depending on reuse expectations.

## Notes

- The native vibrancy material on NSPanels follows system appearance automatically. If the user forces light/dark, vibrancy may mismatch slightly. Acceptable for now.
