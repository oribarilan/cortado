# US: Update Changelog

## Theme

When an app update is available, show the user what changed. The update feed's detail pane should display aggregated changelog entries from the user's current version through the version about to be installed, so they can see exactly what they're getting before hitting "Install."

## Current state

The update feed shows a single "update available" activity with version and an optional `notes` field (from `latest.json`). The detail pane renders fields as flat key-value pairs. There is no multi-line or rich text rendering. The app maintains a `CHANGELOG.md` in Keep a Changelog format, but it's only in the repo -- not shipped or fetched at runtime.

## Progress

- Changelog parser implemented and tested (`src-tauri/src/feed/changelog.rs`, 27 tests)
- UI design decided: Variant B (collapsible per-version), all expanded by default
- Showcase created: `showcases/update-changelog-showcase.html`

## Design decisions

- **UI variant**: Collapsible per-version, all expanded by default. Version headers as landmarks, section headings color-coded (Added=green, Changed=yellow, Fixed=blue). Same rendering in both panel and tray.
- **Data transport**: JSON-serialized into a `FieldValue::Text` field named `changelog`. Frontend parses and renders as structured content. Avoids changing the core Activity struct.

## Sequencing

Tasks are sequential:

1. **Backend: fetch and parse changelog** -- wire the existing parser into the update feed. Fetch CHANGELOG.md from GitHub, parse, attach as a field.
2. **Frontend: render changelog in detail pane** -- shared changelog component for both panel and tray. Collapsible per-version, all expanded by default.
