---
status: pending
---

# 08 — Integration Testing

## Goal

End-to-end verification that the panel works correctly alongside the existing menubar panel.

## Acceptance Criteria

- [ ] `just check` passes cleanly (format + lint + test, no warnings)
- [ ] Panel opens/closes via ⌘+Shift+Space
- [ ] Menubar panel still works via tray click
- [ ] Both can be opened independently without interference
- [ ] Keyboard navigation works: ↑↓ through activities, Enter opens URL, Esc closes
- [ ] Priority section toggles on/off and shows correct items
- [ ] Detail pane updates live with focus
- [ ] Light and dark mode both render correctly
- [ ] Panel state resets correctly on each show
- [ ] No console errors or Rust panics

## Notes

- Manual testing is primary here — the NSPanel and global shortcut behaviors can't be unit-tested easily
- Add unit tests for any shared utility functions extracted during this sprint
- Verify no regressions in existing menubar panel behavior
