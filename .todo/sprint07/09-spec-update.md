---
status: pending
---

# 09 — Spec update

## Goal

Update `specs/main.md` to document the notifications feature. Remove notifications from the Phase 1 non-goals and add a proper notifications section.

## Changes to make

1. **Remove from non-goals**: Delete "Advanced notification policies (digesting, scheduling, status-change alerts, channels/actions)" from the Non-goals section (or reword to reflect what's still out of scope).

2. **Add notifications section** covering:
   - Trigger model (activity rollup kind change)
   - Configuration layers (global settings.toml + per-feed notify toggle)
   - Notification modes (All, EscalationOnly, SpecificKinds)
   - Delivery presets (Immediate, Grouped, Digest)
   - New/removed activity events
   - Click action behavior
   - Startup suppression
   - `settings.toml` config format

3. **Add `settings.toml` reference** to the Configuration section (alongside `feeds.toml`).

4. **Update tech stack** if needed (add `tauri-plugin-notification`).

## Acceptance criteria

- [ ] Non-goals updated to reflect current scope
- [ ] Notifications section added with complete behavioral spec
- [ ] `settings.toml` documented in Configuration section
- [ ] Config format examples included
- [ ] Spec is consistent with implementation

## Notes

- Keep the spec concise and focused on behavior/contracts, not implementation details.
- Reference `specs/status.md` for status kind definitions rather than duplicating.
