---
status: done
---

# Clickable pipeline hidden count

## Goal
Replace the separate hidden count and All pipelines button with one disclosure control in the Tray and Panel.

## Definition of done
- [x] `N hidden ▾` reveals the full snapshot; `Show less ▴` restores filtering in both windows.
- [x] No control appears when nothing is hidden in the normal view. Expanded feeds keep a way back after live updates, including an empty snapshot.
- [x] Mouse and keyboard activation work without opening an activity; focus styling is visible.
- [x] Specs, help text, tests, and changelog match; required checks and independent review pass.

## Task order
1. `01-disclosure.md`: shared control, copy, tests, and verification.

## Constraints
- Keep the existing per-feed, per-window view state, filtering rules, and refresh cadence. No backend, config, notification, or dependency changes.
- Preserve the unrelated deletion at `videos/.opencode/skills/remotion-best-practices`.
