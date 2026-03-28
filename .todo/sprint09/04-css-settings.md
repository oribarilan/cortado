---
status: pending
---

# 04 — CSS Normalization: Settings

## Goal

Refactor `src/settings/settings.css` to use shared design tokens. This is the most complex normalization — settings uses a different token naming scheme and has 8 component-level `@media (prefers-color-scheme: light)` overrides that must be reworked.

## Acceptance Criteria

- [ ] Imports `tokens.css`
- [ ] No `:root` block in this file
- [ ] No `@media (prefers-color-scheme)` blocks in this file (root or component-level)
- [ ] All old tokens replaced with semantic names (see mapping below)
- [ ] All 8 component-level light overrides converted to token-based values
- [ ] Font-family declarations removed (now in `tokens.css`)
- [ ] Granular font sizes (9px, 9.5px, 10.5px, 11.5px) snapped to nearest type-scale step
- [ ] Spacing uses spacing tokens where values map cleanly
- [ ] 4 inline styles in `SettingsApp.tsx` moved to CSS classes
- [ ] Visual parity with current appearance in both light and dark

## Token Mapping

| Old | New |
|-----|-----|
| `--bg` | `--surface` |
| `--s` | `--surface-raised` |
| `--s2` | `--surface-inset` |
| `--t1` | `--text-primary` |
| `--t2` | `--text-secondary` |
| `--t3` | `--text-tertiary` |
| `--border` | `--border` |
| `--hover` | `--hover` |
| `--ac` | `--accent` |
| `--ac-dim` | `--accent-dim` |
| `--ac-soft` | `--accent-soft` |
| `--danger` | `--danger` |

## Component-Level Light Overrides to Rework

Each of these currently uses `@media (prefers-color-scheme: light)` with hardcoded color values. They must be converted to use tokens or `color-mix()` from tokens so they respond to `data-theme` automatically.

1. **`.feed-card:hover`** — hardcoded light hover colors → use `--hover` or `color-mix()` from `--surface-raised`
2. **`.btn-primary:hover`, `.btn-danger-sm`, `.btn-danger-sm:hover`** — hardcoded button colors → derive from `--accent`/`--danger` tokens
3. **`.save-error`, `.save-success`** — hardcoded feedback colors → derive from `--danger`/`--status-attention-positive`
4. **`.setting-row`** — hardcoded background → use `--surface-raised` or `--hover`
5. **`.toggle`, `.toggle::after`, `.toggle.on`, `.toggle.on::after`** — hardcoded toggle colors → derive from `--accent`/`--surface` tokens
6. **`.dep-banner-warning`, `code`, `a`** — hardcoded warning/link colors → derive from `--danger`/`--accent`
7. **`.test-panel.success`, `.test-panel.error`** — hardcoded panel colors → derive from status/danger tokens
8. **`.modal-backdrop`, `.modal-dialog`** — hardcoded backdrop/dialog colors → derive from `--surface`/`--border`

## Inline Styles to Move (SettingsApp.tsx)

- `style={{ marginTop: 16 }}` → CSS class
- `style={{ flex: 1 }}` (×2) → CSS class
- `style={{ margin: 0 }}` → CSS class

## Notes

- This is the largest single task. Consider doing a first pass (token rename + root block removal) and a second pass (component-level override rework).
- Some component-level overrides may become unnecessary once tokens are theme-aware — the default token values may already produce the right result. Test each before adding complexity.
- The `font-family: inherit` declarations in specific components can be removed since the global font stack now applies everywhere.
