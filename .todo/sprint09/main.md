---
sprint: 9
theme: Theme System — Light / Dark / System Preference
status: pending
---

# Sprint 09 — Theme System

## Theme

Introduce an app-wide theme system with a user-facing light/dark/system picker. This requires normalizing the three existing CSS files (which use `prefers-color-scheme` inconsistently), introducing a `data-theme` attribute mechanism, adding a `theme` setting to `AppSettings`, and wiring live theme switching across all windows.

## Decisions

- **Picker location**: Settings > General (single segmented control or dropdown).
- **Default**: `"system"` — follows OS appearance, matching current behavior.
- **Scope**: All app windows (menubar panel, main screen, settings).

## Task Sequence

1. **01-theme-picker** — Full implementation: CSS normalization, `data-theme` attribute, settings schema, UI control, cross-window live updates.

## Notes

- The three CSS files currently default to different directions: `styles.css` and `settings.css` default dark, `main-screen.css` defaults light. All need normalizing to a consistent dark-first baseline.
- Native vibrancy material on NSPanels follows the system appearance automatically. If the user forces a theme, vibrancy may mismatch slightly — acceptable for now.
