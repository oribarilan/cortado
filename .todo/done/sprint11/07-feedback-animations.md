---
status: done
---

# Feedback & status animations

## Goal

Animate transient feedback elements in Settings -- the test-result panel, save-success indicators, and other status messages -- so they feel polished rather than appearing/disappearing instantly.

## Acceptance criteria

### Test result panel
- [ ] When a feed test completes, the result panel expands into view (height + opacity animation) rather than appearing instantly.
- [ ] When the user dismisses or navigates away, the panel collapses out.
- [ ] Uses `--duration-normal` and `--ease-out`.

### Save/success feedback
- [ ] The inline "Saved" indicator in notification settings already has an opacity fade -- verify it uses the new animation tokens.
- [ ] Any other success/error feedback in Settings (e.g., after saving a feed) uses a consistent fade-in entrance.

### General
- [ ] All feedback animations respect `prefers-reduced-motion`.
- [ ] Feedback elements that auto-dismiss (e.g., timed "Saved" indicators) animate out before being removed.

## Notes

- The test result panel can use the `grid-template-rows` expand pattern from the menubar panel's activity details.
- Keep feedback animations short -- the user is waiting for a result, not watching a show.
- This task is a good candidate for a final polish pass after the other animation tasks are done.
