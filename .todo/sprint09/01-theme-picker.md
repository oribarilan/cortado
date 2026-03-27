---
status: pending
---

# 14 — Theme Picker (Light / Dark / System)

## Goal

Add a theme preference control to Settings > General that lets the user force light mode, dark mode, or follow the system setting. The choice must apply to all app windows (menubar panel, main screen, settings).

## Acceptance Criteria

- [ ] New `theme` setting in `AppSettings`: `"system"` (default), `"light"`, or `"dark"`
- [ ] Three-option segmented control or dropdown in Settings > General
- [ ] Theme applies immediately on change (no restart required)
- [ ] All three CSS files (`styles.css`, `main-screen.css`, `settings.css`) respect the chosen theme
- [ ] When `"system"` is selected, the app follows `prefers-color-scheme` as it does today
- [ ] The chosen theme persists across app restarts

## Implementation approach

1. **Rust side**: Add `theme: String` to `AppSettings` (default `"system"`), with serde default.
2. **CSS refactor**: Currently the three CSS files use `@media (prefers-color-scheme: ...)` inconsistently (main-screen defaults light, others default dark). Refactor all three to:
   - Default to dark tokens in `:root`
   - Override with light tokens under `[data-theme="light"]` AND `@media (prefers-color-scheme: light)` when `[data-theme="system"]` (or no attribute)
   - When `data-theme="dark"`, always use dark tokens regardless of system setting
   - When `data-theme="light"`, always use light tokens regardless of system setting
3. **Frontend**: Each window's entry point reads the theme setting on mount and sets `document.documentElement.dataset.theme`. Listen for a `theme-changed` event (emitted when settings are saved) to update live.
4. **Settings UI**: Add a segmented control or select in General section.

## Notes

- The native vibrancy material (`NSVisualEffectMaterial::Popover`) on the NSPanels adapts automatically to the system appearance. If the user forces light/dark, the vibrancy may mismatch. This is acceptable for now — the vibrancy is behind the content and mostly invisible. If it becomes a problem, we can look into `NSAppearance` overrides later.
- Normalize the three CSS files to use the same default direction (dark-first) to reduce inconsistency.
