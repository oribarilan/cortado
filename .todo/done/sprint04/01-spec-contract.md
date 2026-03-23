---
status: done
---

# Spec contract update for retained activities + duration strings

## Goal

Align `specs/main.md` with sprint04 scope before implementation.

## Acceptance criteria

- [x] Spec defines duration-string config contract (parsed by `jiff`) for `interval`.
- [x] Spec explicitly states integer interval values are unsupported.
- [x] Spec defines optional `retain` duration-string config with omitted = no retention.
- [x] Spec defines retained-activity runtime lifecycle semantics.
- [x] Spec defines tray rendering semantics for retained activities (hollow dot, active-first ordering).
- [x] Spec states retention is in-memory only for sprint04.
- [x] Terminology uses Feed/Activity/Field and introduces Retained Activity consistently.

## Notes

- This task is spec-only.

## Relevant files

- `specs/main.md`
