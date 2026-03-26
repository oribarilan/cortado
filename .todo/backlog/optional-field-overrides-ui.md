---
status: pending
---

# Field Overrides UI

## Goal

Evaluate whether field overrides (toggle field visibility, change labels) should be user-facing in the settings GUI, or remain a TOML-only power-user feature.

## Context

Field overrides let users customize how a feed's fields are displayed in the menubar panel:
- `visible: false` hides a field from the activity detail view
- `label: "Custom Name"` renames a field's display label

Currently supported in `feeds.toml` via `[feed.fields.<name>]` blocks. The config backend already reads/writes them. No GUI exists.

## Open questions

- Is this a user-facing feature, or an implementation detail for feed authors?
- If user-facing, should it be in the settings GUI or remain TOML-only?
- Could field overrides be auto-discovered from the feed's `provided_fields()` rather than requiring the user to know field names?
