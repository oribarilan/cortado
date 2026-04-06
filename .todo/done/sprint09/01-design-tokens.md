---
status: done
---

# 01 -- Design Tokens + Font Loading

## Goal

Establish the shared design token system and font loading that all other tasks depend on.

## Acceptance Criteria

- [ ] Google Fonts `<link>` for Space Grotesk (weights 400, 500, 600) in `index.html`, `main-screen.html`, `settings.html`
- [ ] `src/shared/tokens.css` exists with all design tokens
- [ ] Dark color values in `:root`, light values via `[data-theme="light"]` and `@media (prefers-color-scheme: light)` on `:root:not([data-theme="dark"])`
- [ ] Text size attributes: `[data-text-size="s"]` = 12px, default = 13px, `[data-text-size="l"]` = 14px, `[data-text-size="xl"]` = 15px
- [ ] Type scale tokens use `rem` (so they scale with root font-size)
- [ ] Spacing and radius tokens use `px`
- [ ] Global font stack, resets, and base styles defined
- [ ] File can be imported by all three entry points without side effects beyond `:root` styling

## Token Inventory

**Colors (semantic):**
- `--surface` -- main background
- `--surface-raised` -- sidebar, cards
- `--surface-inset` -- deeper card areas
- `--text-primary`, `--text-secondary`, `--text-tertiary`
- `--border`, `--separator`, `--hover`, `--expanded-bg`

**Status:**
- `--status-attention-negative`, `--status-waiting`, `--status-running`, `--status-attention-positive`, `--status-idle`

**Accent:**
- `--accent`, `--accent-dim`, `--accent-soft`, `--danger`

**Type scale (rem):**
- `--text-2xs` (~9px), `--text-xs` (~10px), `--text-sm` (~11px), `--text-base` (~13px), `--text-lg` (~14px)

**Font weights:**
- `--font-normal: 400`, `--font-medium: 500`, `--font-semibold: 600`

**Spacing (px):**
- `--space-1: 2px`, `--space-2: 4px`, `--space-3: 6px`, `--space-4: 8px`, `--space-5: 10px`, `--space-6: 12px`, `--space-7: 16px`, `--space-8: 20px`, `--space-9: 24px`

**Radius:**
- `--radius-sm: 3px`, `--radius-md: 5px`, `--radius-lg: 10px`, `--radius-full: 999px`

## Notes

- Dark color values come from existing `@media (prefers-color-scheme: dark)` blocks in `styles.css` and `main-screen.css` (they're identical).
- Light color values come from the existing `:root` defaults in `styles.css` and `main-screen.css` (also identical).
- Settings-specific surface tokens (`--surface-raised`, `--surface-inset`) use the existing settings `--s` and `--s2` values.
- Accent tokens (`--accent`, `--accent-dim`, `--accent-soft`) use the existing settings `--ac*` values.
- The font stack is: `"Space Grotesk", "SF Pro Text", -apple-system, BlinkMacSystemFont, system-ui, sans-serif`
