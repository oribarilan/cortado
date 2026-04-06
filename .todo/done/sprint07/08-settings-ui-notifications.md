---
status: done
---

# 08 -- Settings UI -- Notifications tab

## Goal

Add a "Notifications" section to the settings window sidebar and build the full notification preferences UI.

## UI structure

### Sidebar
Add a third nav item: 🔔 Notifications (between General and Feeds, or after Feeds).

### Notifications section content

1. **Master toggle** -- "Enable notifications" (on/off)

2. **Notification mode** (disabled when master is off):
   - Radio group or segmented control:
     - All -- notify on any status change
     - Escalation only -- notify only when status worsens
     - Specific kinds -- reveal checkboxes:
       - ☑ Needs attention (AttentionNegative)
       - ☑ Ready to go (AttentionPositive)
       - ☐ Waiting
       - ☐ In progress (Running)
       - ☐ Idle

3. **Delivery preset** (disabled when master is off):
   - Radio group or segmented control:
     - Immediate -- instant, one per change
     - Grouped (default) -- batched per feed

4. **Activity events** (disabled when master is off):
   - ☑ Notify when new activities appear
   - ☑ Notify when activities are removed

5. **Permission status** -- show macOS notification permission status with a "Request permission" button if not granted.

## Acceptance criteria

- [ ] "Notifications" nav item in settings sidebar
- [ ] Master toggle reads/writes `settings.toml` via Tauri commands
- [ ] Notification mode selector with dynamic "Specific kinds" checkboxes
- [ ] Delivery preset selector (Immediate / Grouped)
- [ ] New/removed activity toggles
- [ ] Permission status display with request button
- [ ] All controls disabled (visually and functionally) when master toggle is off
- [ ] Settings persist across app restarts (via `settings.toml`)
- [ ] Follows existing settings UI design patterns (teal accent, consistent spacing)
- [ ] `just check` passes

## Notes

- Reuse the existing settings CSS and component patterns from the General and Feeds sections.
- Consider grouping related settings with subtle section dividers and labels.
- The permission status is read from `tauri-plugin-notification`'s JS API.

## Relevant files

- `src/settings/SettingsApp.tsx` -- settings UI, sidebar nav
- `src/settings/settings.css` -- settings styles
- Task 01 output -- `get_settings` / `save_settings` Tauri commands
- Task 03 output -- notification config types (for form structure)
