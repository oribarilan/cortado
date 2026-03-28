---
status: pending
---

# 13 — Panel Focus Highlight Alignment

## Goal

Align the panel's focused-row styling with the app's design language. Currently the panel uses a filled background + hard outline, while the menubar panel and settings use subtler, outline/border-only focus states gated by keyboard activity.

## Acceptance Criteria

- [ ] Focused row uses the same outline-only style as the menubar panel (no filled background)
- [ ] Focus ring is only visible during keyboard navigation (gate on a `keyboard-active` class, same pattern as the menubar panel)
- [ ] Mouse clicks still update the selected row but don't show a focus ring
- [ ] The selected row gets a subtle hover-like background to indicate it's current, distinct from the strong focus outline

## Notes

- The menubar panel uses `.panel-root.keyboard-active :focus-visible` to gate focus outlines. The panel uses virtual focus (state-driven `.focused` class, not native `:focus-visible`), so the gating mechanism needs to track keyboard vs mouse input via event listeners (same `keydown` → add class, `mousedown` → remove class pattern).
- Keep the status-colored dot and the overall row layout unchanged — this is purely about the selection/focus visual treatment.
