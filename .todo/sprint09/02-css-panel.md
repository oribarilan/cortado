---
status: pending
---

# 02 — CSS Normalization: Menubar Panel

## Goal

Refactor `src/styles.css` to use shared design tokens from `tokens.css`, removing all inline theme definitions.

## Acceptance Criteria

- [ ] Imports `tokens.css` (via entry point or direct import)
- [ ] No `:root` block in this file (all tokens come from `tokens.css`)
- [ ] No `@media (prefers-color-scheme)` block in this file
- [ ] All `--panel-*` references replaced with semantic tokens
- [ ] All `--s-*` status references replaced with `--status-*` tokens
- [ ] Font sizes use type-scale tokens
- [ ] Font weights use weight tokens
- [ ] Spacing uses spacing tokens where values map cleanly
- [ ] Border-radius uses radius tokens
- [ ] Panel root `border-radius: var(--radius-lg)` (10px, unchanged)
- [ ] Visual parity with current appearance in both light and dark

## Token Mapping

| Old | New |
|-----|-----|
| `--panel-surface` | `--surface` |
| `--panel-text` | `--text-primary` |
| `--panel-text-secondary` | `--text-secondary` |
| `--panel-text-tertiary` | `--text-tertiary` |
| `--panel-border` | `--border` |
| `--panel-separator` | `--separator` |
| `--panel-hover` | `--hover` |
| `--panel-expanded-bg` | `--expanded-bg` |
| `--s-attention-negative` | `--status-attention-negative` |
| `--s-waiting` | `--status-waiting` |
| `--s-running` | `--status-running` |
| `--s-attention-positive` | `--status-attention-positive` |
| `--s-idle` | `--status-idle` |
| `--fz-feed` / `--fz-title` / `--fz-field` | type-scale tokens |
| `--fw-feed` / `--fw-title` | `--font-semibold` / `--font-normal` |
| `--row-radius` | `--radius-sm` |

## Notes

- Keep `--dur-expand`, `--dur-hover`, `--dur-chevron`, `--ease-expand`, `--ease-default` as local vars (animation tokens are out of scope).
- Keep `--pad-x`, `--feed-gap`, `--activity-row-h`, `--dot-size`, `--field-indent`, `--field-row-gap` as local vars if they don't map cleanly to the spacing scale — or replace where they do.
- Keep `@keyframes shimmer`, `@keyframes pulse`, and `@media (prefers-reduced-motion)` blocks.
- `color-mix()` expressions that reference tokens should continue working — just update the token name inside them.
