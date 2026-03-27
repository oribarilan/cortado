---
status: pending
---

# Settings section transitions

## Goal

Add a crossfade transition when switching between General, Feeds, and Notifications sections in the Settings sidebar, so the content swap feels smooth rather than instant.

## Acceptance criteria

- [ ] Clicking a sidebar section triggers a crossfade of the main content area (old content fades out, new content fades in).
- [ ] Transition duration uses `--duration-normal`.
- [ ] The sidebar active state updates immediately (no delay on the nav highlight).
- [ ] No layout shift during the transition — the content area should not resize or jump.
- [ ] Respects `prefers-reduced-motion` (instant swap when reduced motion is preferred).

## Notes

- A simple opacity transition is sufficient — no need for directional slides between sections.
- Implementation options:
  - CSS transition on a wrapper with a brief `opacity: 0` → `opacity: 1` cycle triggered by a key change.
  - A small React state machine: `visible → fading-out → fading-in → visible`.
- Avoid unmounting/remounting the section content during the fade if it causes flicker. Keeping both mounted and toggling visibility+opacity may be smoother.
- If crossfade feels too complex for the first pass, a simple fade-out-then-in (sequential, not overlapping) is fine.
