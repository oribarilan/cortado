---
status: done
---

# 03 — CSS Normalization: Main Screen

## Goal

Refactor `src/main-screen/main-screen.css` to use shared design tokens from `tokens.css`, removing all inline theme definitions.

## Acceptance Criteria

- [ ] Imports `tokens.css`
- [ ] No `:root` block in this file
- [ ] No `@media (prefers-color-scheme)` block in this file
- [ ] All `--panel-*` references replaced with semantic tokens
- [ ] All `--s-*` status references replaced with `--status-*` tokens
- [ ] Font sizes use type-scale tokens
- [ ] Spacing uses spacing tokens where values map cleanly
- [ ] Panel root `border-radius: var(--radius-lg)` (standardized from 12px → 10px)
- [ ] Visual parity with current appearance in both light and dark

## Token Mapping

Same mapping as task 02 (panel and main-screen use identical token names).

## Notes

- Keep `--list-width`, `--dot-size`, `--row-h`, `--pad-x` as local vars if screen-specific.
- Keep `@keyframes pulse` and `@media (prefers-reduced-motion)` blocks.
- `--fz-title` is currently 12.5px here (vs 13px in panel). Both should snap to the same type-scale token — the slight visual shift is accepted.
