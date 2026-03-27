---
status: pending
---

# 03 — Detail Pane

## Goal

Build the right-side detail pane that shows full information for the currently focused activity. Updates live as keyboard focus moves through the list.

## Acceptance Criteria

- [ ] Right pane (~340px) shows details for the focused activity
- [ ] Content: feed label (uppercase), activity title, highest-status chip, field rows (key/value grid), "↗ Open" link
- [ ] Field values colored by status kind where applicable
- [ ] Detail pane updates immediately when focus changes (no animation delay)
- [ ] When no activity is focused (e.g., empty list), show a subtle empty state
- [ ] "↗ Open in [service]" link calls `open_activity` — label can be generic ("Open") or infer service from feed type
- [ ] Scroll independently if field list is long

## Notes

- The detail pane is read-only and non-focusable — it just reflects the list's focused item
- The existing menubar panel's expand/collapse detail body is a good reference for field rendering
- Respect `field_overrides` from config — only show fields where `visible` is not `false`
