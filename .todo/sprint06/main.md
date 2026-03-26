# Sprint 06 — Settings Experience

## Theme

Add a settings window to Cortado, turning it from a panel-only menubar app into one that also has a proper windowed settings experience. The settings window serves two purposes:

1. **General preferences** — "Start on system startup" toggle (via `tauri-plugin-autostart`)
2. **Feed configuration GUI** — A visual editor for the TOML-based feed configs, so non-power-users don't need to hand-edit `~/.config/cortado/feeds.toml`

## Architecture decisions

### Multi-page frontend (separate entry points)

The menubar panel and settings window are fundamentally different surfaces:
- Panel: transparent, undecorated, NSPanel, compact
- Settings: standard decorated macOS window, full-width forms

Use **Vite multi-page** with a separate `settings.html` entry point and a dedicated React root (`src/settings/`). This keeps bundle sizes small per window and avoids loading panel code in settings or vice versa.

### Settings window lifecycle

- Declare the window in `tauri.conf.json` with `"create": false`
- Create lazily from Rust when user clicks "Settings..." in tray menu
- If window already exists, show + focus it (no duplicates)
- Standard decorated macOS window with titlebar

### Config read/write

- New Tauri commands to read/write `feeds.toml` as structured data
- Reuse existing config parser for validation
- Back up config before overwriting
- After save, existing config-change detection flags restart-needed

### Autostart

- Use the official `tauri-plugin-autostart` with `MacosLauncher::LaunchAgent`
- Expose via JS API: `enable()`, `disable()`, `isEnabled()`

## Task sequencing

Tasks are sequential — each builds on the prior:

1. **Settings window infrastructure** — Tauri config, Rust window management, Vite multi-page, tray menu item, capabilities
2. **Autostart plugin** — Install, wire up, add to settings UI as first "General" setting
3. **Config backend commands** — Read/write/validate `feeds.toml` from the frontend
4. **Settings UI: Feeds config** — Visual feed editor (list, add, edit, remove feeds)

## Design decisions

### 1. Color theme / accent

**Decision: Teal (hue 178).**

Accent scale (OKLCH, hue 178):
- `--ac-soft`: `oklch(94% 0.03 178)` — backgrounds, active nav highlight
- `--ac-light`: `oklch(80% 0.08 178)` — borders, secondary accents
- `--ac-base`: `oklch(65% 0.12 178)` — primary accent (light theme)
- `--ac-mid`: `oklch(50% 0.1 178)` — text on light backgrounds
- `--ac-dim`: `oklch(38% 0.06 178)` — dark-mode backgrounds, toggle fill
- Dark-mode foreground accent: `oklch(72% 0.14 178)`

Neutrals remain blue-gray (hue 250) from the existing panel. Semantic status colors unchanged.
Settings window respects `prefers-color-scheme` (light/dark), same as panel.

Reference: `showcases/settings-color-theme-showcase.html`

**Status: Decided**

### 2. Full settings UX flow

**Decision: F2 — Breadcrumb Replace.**

Edit/add flow:
- Clicking a feed card (or "+ New feed") replaces the main content area with a form
- A breadcrumb `Feeds › Feed Name` sits at the top, with "Feeds" as a clickable link back to the list
- Delete button lives in the breadcrumb row (right-aligned)
- Save / Discard buttons at the bottom of the form
- Sidebar stays unchanged — "Feeds" remains the active nav item throughout
- Type-specific fields appear dynamically based on feed type selection

Additional UX elements:
- Config path bar at the bottom of the feed list: `~/.config/cortado/feeds.toml` with "Open in editor" and "Reveal" buttons
- Empty state when no feeds: prompt pointing to "+ New feed" button
- Validation errors shown inline below fields
- "Restart needed" banner after successful save
- Token fields use `type="password"` with reveal toggle

Reference: `showcases/settings-edit-flow-refined-showcase.html` (F2 section)

**Status: Decided**

### 3. Technical mechanism: GUI ↔ TOML config

**Decision: Option C — Clean rewrite + backup.**

- Parse TOML → existing `FeedConfig` structs → serialize back as clean, normalized TOML.
- Back up old file as `feeds.toml.bak` before every GUI save.
- Reuse the existing parser for validation. No new TOML crate needed.
- Loses comments/formatting on GUI save, but the backup is a safety net.
- Can upgrade to `toml_edit` preserving round-trip later if demand emerges.

Rationale: The spec doesn't require comment preservation. `FeedConfig` already captures `type_specific` as a raw `toml::Table`, so no feed data is lost — only formatting. KISS.

#### Config-file-driven UX

The settings UI must make clear that `feeds.toml` is the source of truth:
- Show the full config file path (e.g. `~/.config/cortado/feeds.toml`) in the UI
- **"Open in editor"** button — opens the TOML file in the user's default text editor (via `open` command)
- **"Reveal in Finder"** button — opens the containing directory with the file selected
- These reinforce that the GUI is a convenience layer over a text file, not a replacement

**Status: Decided**

## Scope boundaries

**In scope:**
- macOS only (consistent with Phase 1)
- Settings window with General + Feeds sections
- Autostart toggle
- Feed CRUD (add, edit, remove feeds)
- Field override editing (visible, label)
- Config validation before save
- Config backup before overwrite
- Config file path displayed in settings UI
- "Open in editor" button for feeds.toml
- "Reveal in Finder" button for config directory

**Out of scope (future work):**
- Config file watcher (already in backlog as `optional-config-file-watcher.md`)
- Import/export config
- Undo/redo in settings
- Settings sync across devices
