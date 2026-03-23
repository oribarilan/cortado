---
status: done
---

# Runtime retained-activity lifecycle

## Goal

Implement generic runtime logic that retains disappeared activities for each feed's configured retention duration.

## Acceptance criteria

- [x] On successful poll, activities missing from current results are retained when feed retention is configured.
- [x] Retained activities expire deterministically once retention duration elapses.
- [x] Omitted retain (none) preserves current behavior (disappeared activities removed immediately).
- [x] Reappearing activity IDs become active again (not duplicated).
- [x] Poll-error behavior remains unchanged (no notification/retention regressions).
- [x] `just check` passes.

## Notes

- Match activities by `Activity.id`.
- Keep implementation efficient and deterministic.

## Relevant files

- `src-tauri/src/feed/runtime.rs`
- `src-tauri/src/feed/mod.rs`
