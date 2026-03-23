---
status: done
---

# Document current custom-feed primitives

## Goal

Document what users can already do today to define/customize feeds, without introducing a new primitive.

## Acceptance criteria

- [x] README/spec docs describe shared feed primitives clearly:
  - [x] `interval` and `retain`
  - [x] field overrides (`visible`, `label`)
- [x] README clarifies shell feed as current custom feed escape hatch and its limits (single command output field model).
- [x] Documentation distinguishes curated feed types vs custom shell feed usage.
- [x] No new custom feed primitive is implemented in sprint05.
- [x] Docs match actual implemented behavior.

## Relevant files

- `README.md`
- `specs/main.md`
