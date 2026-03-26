---
status: pending
---

# 04 — Settings UI: Feeds config

## Goal

Build the frontend GUI for managing feed configurations visually. Users should be able to list, add, edit, and remove feeds without touching the TOML file directly.

## Acceptance criteria

- [ ] Settings window has a sidebar or tab navigation with "General" and "Feeds" sections
- [ ] Feeds section shows a list of configured feeds (name + type)
- [ ] User can add a new feed (picks a type, then fills in required fields)
- [ ] User can edit an existing feed (form pre-populated with current values)
- [ ] User can remove a feed (with confirmation)
- [ ] Feed editor form shows appropriate fields based on feed type:
  - `github-pr`: repo, token (masked), interval, retain
  - `ado-pr`: org, project, token (masked), interval, retain
  - `shell`: command, interval, retain
- [ ] Field overrides section lets user toggle field visibility and change labels
- [ ] Save button validates via backend and shows errors inline if invalid
- [ ] Successful save shows a confirmation and a note that restart is needed for changes to take effect
- [ ] Cancel / discard changes option
- [ ] Config file path (`~/.config/cortado/feeds.toml`) displayed prominently in the Feeds section
- [ ] "Open in editor" button opens feeds.toml in the default text editor
- [ ] "Reveal in Finder" button opens the config directory with the file selected
- [ ] UI follows macOS design conventions (native-feeling forms, spacing, typography)
- [ ] `just check` passes cleanly

## Notes

### Feed type-specific fields

The UI needs to know which fields are required/optional per feed type. This can be:
- A static map in the frontend (simplest, since feed types are curated)
- Or a Tauri command that returns the schema per type

Recommend the static map approach for now — the feed types are known and stable.

### UX flow

**Pattern: F2 — Breadcrumb Replace** (see `showcases/settings-edit-flow-refined-showcase.html`)

1. User opens Settings → Feeds section (sidebar)
2. Sees L5b two-line card list of feeds (or empty state with "Add your first feed" prompt)
3. Config path bar at bottom: `~/.config/cortado/feeds.toml` with "Open in editor" / "Reveal" buttons
4. Clicks "Add Feed" or a feed card → main area replaces with form, breadcrumb shows `Feeds › Feed Name`
5. Fills in fields → clicks Save
6. Backend validates → on success, writes file, shows "Restart needed" banner
7. On error, highlights invalid fields with messages
8. Click "Feeds" in breadcrumb to go back to list

### Design specs

- **Layout**: L5b — sidebar nav + two-line feed cards with left-edge indicator + type badge
- **Color**: Teal accent (hue 178), blue-gray neutrals (hue 250), light/dark via system preference
- **Typography**: Space Grotesk for settings window (SF Pro Text remains for panel)
- **Edit pattern**: F2 breadcrumb replace — form replaces list, breadcrumb for navigation
- References: `showcases/settings-l5-sidebar-cards-showcase.html`, `showcases/settings-color-theme-showcase.html`, `showcases/settings-edit-flow-refined-showcase.html`

### Sensitive fields (tokens)

- Token fields should use `<input type="password">` with a toggle to reveal
- Tokens are stored in the TOML file as-is (no encryption — same as current behavior)
- Future: consider moving tokens to the system keychain (out of scope for this sprint)

### Design

Design decisions are finalized — see `.todo/sprint06/main.md` for full details.
