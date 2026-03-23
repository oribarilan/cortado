---
status: done
---

# Spec contract for `ado-pr`

## Goal

Define `ado-pr` behavior and config contract in `specs/main.md` before implementation.

## Acceptance criteria

- [x] `ado-pr` is promoted from future/planned to current curated feed in spec.
- [x] Required config keys are documented: `org`, `project`, `repo`.
- [x] Auth contract is documented as `az login` only for sprint05.
- [x] Dependency preflight contract is explicit (`az` present, `azure-devops` extension present, authenticated).
- [x] Initial field set is documented: `review`, `mergeable`, `draft`, `labels`.
- [x] Poll scope documented as active PRs only, with retention handled by shared `retain` primitive.
- [x] Terminology remains Feed/Activity/Field.

## Relevant files

- `specs/main.md`
