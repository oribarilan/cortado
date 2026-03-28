---
status: done
---

# 00 — UX Design Spec

## Goal

Create `specs/ux_design.md` documenting the app's style system design decisions, motivation, and reasoning. This serves as the living reference for all UX/design choices — why Space Grotesk, why these token names, why this theme mechanism, etc.

## Acceptance Criteria

- [ ] `specs/ux_design.md` exists and covers all topics below
- [ ] Motivation and reasoning explained for each decision, not just the "what"

## Topics to Cover

- **Design token system**: why semantic naming, why a shared `tokens.css`, token inventory
- **Typography**: why Space Grotesk, font stack, type scale definition and rationale
- **Color palette**: dark-first baseline rationale, color token semantics, status color system
- **Theme system**: light/dark/system mechanism, how `data-theme` attribute works, CSS architecture
- **Text size scaling**: rationale for root font-size approach, the 4 levels (S/M/L/XL), why `rem` for type and `px` for spacing
- **Spacing scale**: the scale steps, rationale
- **Border radius scale**: the scale steps, standardization decisions (e.g., panel root 10px)
- **Surface hierarchy**: `--surface`, `--surface-raised`, `--surface-inset` — what each is for
- **Accent and status colors**: semantics and usage
- **Cross-window consistency**: how appearance settings propagate via events

## Notes

- This is a spec, not implementation docs. Focus on the "why" and the design contract, not CSS syntax.
- Should reference `specs/main.md` and `specs/status.md` where relevant (e.g., status colors map to StatusKind).
- Write this after implementation stabilizes (task 04 completion) so it reflects final decisions.
