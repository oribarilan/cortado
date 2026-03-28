---
status: pending
---

# 05 — Priority Section (Needs Attention)

## Goal

Add an optional "Needs Attention" section at the top of the activity list that aggregates attention-negative activities from all feeds.

## Acceptance Criteria

- [ ] When enabled, a "⚑ Needs Attention" section appears at the top of the list, before feed groups
- [ ] Section contains activities with `AttentionNegative` as their derived status kind, from any feed
- [ ] Each row in this section shows the activity title + a small feed-hint label (e.g., "GitHub")
- [ ] Activities in this section are NOT duplicated in their feed group below (deduplicated)
- [ ] A separator line divides the attention section from the feed groups
- [ ] The section is hidden when there are no attention-negative activities
- [ ] Toggle stored in app settings: `panel.show_priority_section` (boolean, default: `true`)
- [ ] Toggle accessible from Settings window (General section)
- [ ] Keyboard navigation treats attention-section rows the same as feed-section rows

### Backend (settings schema)

- [ ] Add `panel` section to `AppSettings` struct in `app_settings.rs` (e.g., `MainScreenSettings { show_priority_section: bool }`) with `#[serde(default)]` so existing `settings.toml` files are handled gracefully
- [ ] Add Tauri commands to read/write `show_priority_section` (or reuse a generic settings update command if one exists)

## Notes

- The deduplication approach: when priority section is on, attention-negative activities are removed from their feed groups and only appear in the priority section. This avoids confusion about duplicates.
- Strictly `AttentionNegative` only — do not include `Waiting` or other status kinds.
