---
status: pending
---

# 07 — Panel Lifecycle

## Goal

Polish the main screen panel's show/hide behavior to feel instant and correct across all edge cases.

## Acceptance Criteria

- [ ] On show: scroll to top, focus first activity, reset detail pane
- [ ] On show: suppress any collapse/transition animations (instant state)
- [ ] Hides when panel resigns key (clicking elsewhere)
- [ ] Hides on Esc keydown
- [ ] Hides on space/desktop change
- [ ] Toggle: ⌘+Shift+Space while visible hides, while hidden shows
- [ ] No Dock icon flash or activation policy issues
- [ ] Panel appears on the monitor where the cursor is (centered)
- [ ] Multi-monitor: correct centering when displays have different sizes/arrangements
- [ ] Panel does not steal focus from the menubar panel (both can exist but only one is key at a time)
- [ ] Two NSPanels coexisting: verify that resign-key events are scoped — showing/hiding the main screen must not interfere with the menubar panel's visibility, and vice versa. Each panel's delegate should check its own window label before acting.

## Notes

- The menubar panel already handles most of these concerns in `fns.rs` — reuse or adapt those patterns
- The spotlight reference handles `window_did_resign_key` to auto-hide, and uses `ActivationPolicy::Prohibited` — these should already be in place from task 01
- Test with multiple displays if possible
