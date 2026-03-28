---
sprint: 11
theme: Animations & Transitions
status: done
---

# Sprint 11 — Animations & Transitions

## Theme

Add purposeful motion throughout the app — loading states, view transitions, modal entrances, and feedback animations. The goal is polish, not spectacle: subtle, fast animations that make the UI feel responsive and alive without drawing attention to themselves.

Pulls in backlog item `optional-loading-animations.md`.

## Principles

- **Respect `prefers-reduced-motion`** — every animation must degrade gracefully. The existing pattern (disable transitions/animations under `reduce`) continues.
- **Fast by default** — most transitions should be 120–200ms. Nothing above 300ms unless there's a good reason.
- **No jank** — only animate `transform`, `opacity`, and `grid-template-rows` (already proven in the codebase). Avoid animating `height`, `width`, or layout-triggering properties.
- **Consistent tokens** — establish shared CSS custom properties for durations and easings so motion feels cohesive across all three windows.

## Task Sequence

Tasks are roughly ordered by dependency but most are parallelizable after task 01.

1. **01-animation-tokens** — Shared CSS custom properties for durations, easings, and a reduced-motion reset. Foundation for all other tasks.
2. **02-loading-states** — Loading/refresh animations for startup, manual refresh, and background refresh across menubar panel and main screen. (From backlog.)
3. **03-modal-animations** — Entrance/exit animations for the reset-defaults modal and delete-feed confirmation in Settings.
4. **04-settings-section-transitions** — Crossfade transition when switching between General / Feeds / Notifications sections.
5. **05-settings-feed-nav-transitions** — Slide/crossfade transition between the feed list and the feed edit form.
6. **06-main-screen-detail-transitions** — Content crossfade in the detail pane when selection changes, and enter/exit animation for the priority section.
7. **07-feedback-animations** — Animate test-result panel expansion, save-success indicators, and transient status feedback in Settings.

## Notes

- The menubar panel already has solid animation coverage (expand/collapse, shimmer, pulse, hover). This sprint focuses more on Settings and main screen, which are currently instant-swap everywhere.
- Task 01 should also audit existing animation values and normalize them to the new tokens.
- Keep bundle impact near zero — this is pure CSS + minor React state, no animation libraries.
