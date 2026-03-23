# Task: Semantic status types

## Context

The current `StatusKind` enum (`Success`, `Error`, `Pending`, `Warning`, `Neutral`) is hardcoded and maps directly to UI rendering. Feed implementations choose a `StatusKind` for each status value, but there's no way for different feed types to express richer semantics that the UI could render consistently.

For example, an ADO PR with a pending reviewer shows `review: awaiting` with `StatusKind::Pending`, and a GitHub PR shows `review: awaiting` similarly. But "pending review" and "CI running" are both `Pending` — they mean different things to the user, and a feed author has no way to express that distinction.

The idea: introduce **semantic status types** (e.g., `waiting`, `in-progress`, `blocked`, `action-needed`) that feed types can assign to their status values. The UI maps these semantic types to visual treatments. This lets feeds express intent ("this is waiting on someone else" vs "this is actively running") while the UI decides how to render each semantic category consistently across all feed types.

This would replace or layer on top of `StatusKind`, making status rendering more flexible without requiring UI changes every time a new feed type is added.

**Value delivered**: Richer, more consistent status rendering across feed types; feed authors get expressive primitives without coupling to UI details.

## Related Files
- `src-tauri/src/feed/mod.rs` — `StatusKind` enum, `FieldValue::Status`
- `src-tauri/src/feed/ado_pr.rs` — ADO PR status field mappings
- `src-tauri/src/feed/github_pr.rs` — GitHub PR status field mappings
- `src-tauri/src/feed/shell.rs` — Shell feed status parsing
- `src-tauri/src/tray.rs` — tray rendering of status fields, dot color logic
- `specs/main.md` — field type definitions

## Dependencies
- None

## Acceptance Criteria
- [ ] Design decided: what semantic types exist, how feeds declare them, how UI maps them
- [ ] `StatusKind` (or its replacement) supports the new semantic types
- [ ] Feed implementations updated to use semantic types
- [ ] UI rendering updated to distinguish semantic types visually
- [ ] Spec updated with the new status type system
- [ ] Existing behavior preserved (no visual regressions for current feeds)
- [ ] `just check` passes

## Scope Estimate
Medium

## Notes
- This is exploratory — the exact set of semantic types and the configuration surface need discussion before implementation.
- Consider whether feeds should declare their status vocabulary in config (per-feed-type flexibility) or whether a fixed set of semantic types is sufficient.
- The shell feed's user-defined statuses add a wrinkle: users already pick status values and severities. Any new system needs to remain compatible with that.
