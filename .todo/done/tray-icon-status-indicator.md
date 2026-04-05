---
status: pending
---

# Tray icon status visualization and settings copy update

## Problem

The tray icon is static — it doesn't reflect the global rollup status. Users have to click the tray or open the panel to know if anything needs attention. The icon should serve as a passive, glanceable signal.

## Goal

1. **Tray icon status indicator** — show a colored dot, badge, or other subtle visualization on the tray icon that reflects the highest-severity status across all feeds (using the existing rollup logic). For example, a red dot when any feed has `AttentionNegative`, yellow for `Waiting`, etc.

2. **Update settings copy** — change the tray icon toggle description from:
   > Show tray icon and tray menu. When off, use the global shortcut or Spotlight to access Cortado.

   To something like:
   > Show tray icon for at-a-glance status. A colored indicator reflects the highest-priority state across your feeds.

   (Final wording to be refined during implementation.)

## Design questions

- What visualization works best in the macOS menubar? Options: colored dot overlay on the icon, swapping the icon entirely (e.g., different icon per status), a small badge number, or a colored ring/outline.
- Should the indicator animate (e.g., pulse for `Running`)? If so, respect `prefers-reduced-motion`.
- What does the icon look like when all feeds are `Idle` or no feeds are configured? Probably just the plain icon with no indicator.
- macOS menubar icons are typically template images (monochrome). A colored overlay may need to be a separate non-template layer — investigate feasibility.

## Relevant files

- `src-tauri/src/tray.rs` — tray icon setup
- `src-tauri/src/command.rs` — rollup status computation
- `src/settings/` — settings UI, toggle description copy
- `specs/status.md` — StatusKind severity ordering
