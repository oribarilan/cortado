---
status: pending
---

# 04 — Per-feed notify toggle

## Goal

Add a `notify` field to `FeedConfig` so users can disable notifications for specific feeds. Default is `true` (opt-out model).

## Acceptance criteria

- [ ] `notify: Option<bool>` added to `FeedConfig` (defaults to `true` when absent)
- [ ] Parsed from `feeds.toml` `[[feed]]` entries
- [ ] Preserved in config write-back (settings UI save)
- [ ] Exposed in settings UI feed editor form (checkbox/toggle in feed edit view)
- [ ] DTO updated in `settings_config.rs` to include `notify`
- [ ] `just check` passes

## Example config

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "personal/cortado"
notify = false  # Silence notifications for this feed
```

## Notes

- `notify` is a per-feed override. It only takes effect when global notifications are enabled.
- The settings UI feed editor already has a form with fields like `interval` and `retain` — `notify` is a new toggle in the same form.
- Consider placement in the feed editor UI: near the top (prominent) or grouped with other behavior settings (interval, retain).

## Relevant files

- `src-tauri/src/feed/config.rs` — `FeedConfig` struct, TOML parsing
- `src-tauri/src/settings_config.rs` — DTO for settings UI
- `src/settings/SettingsApp.tsx` — feed editor form
