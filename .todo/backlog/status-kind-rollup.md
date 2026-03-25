# Task: Three-level status kind rollup

## Goal

Implement a consistent status kind rollup at every level of the hierarchy:

```
Fields → Activity dot → Feed rollup → Tray icon
```

All levels use the same algorithm: **highest status kind wins**.

```
AttentionNegative > Waiting > Running > AttentionPositive > Idle
```

## Context

The per-activity rollup (fields → activity dot) already exists. This task adds:

1. **Feed-level rollup** — the feed header shows the worst-case status kind across its activities.
2. **Tray icon rollup** — the tray icon reflects the worst-case status kind across all feeds.

## Design Decisions

- **Same algorithm everywhere.** No special logic at any level — just highest kind wins.
- **Retained activities always roll up as Idle.** They are no longer actively monitored, so they should not affect the feed or tray rollup. This means no special-case exclusion is needed — they naturally have lowest priority.
- **Tray icon always expresses the global rollup.** When everything is Idle, the tray shows Idle (gray/neutral). The tray icon is the global at-a-glance signal.

## Open Questions

- **Feed header visual** — dot? colored text? subtle background tint?
- **Tray icon visual** — colored dot overlay? SF Symbol swap? Badge? Needs macOS-native exploration.

## Related Files

- `src/App.tsx` — `deriveActivityKind` (per-activity rollup, already exists), feed header rendering
- `src-tauri/src/main.rs` — tray icon setup
- `specs/status.md` — status model spec

## Acceptance Criteria

- [ ] Feed header shows a visual indicator of its rolled-up status kind
- [ ] Tray icon reflects the global rolled-up status kind across all feeds
- [ ] Retained activities contribute as Idle to rollup
- [ ] Rollup uses same precedence at all three levels
- [ ] `just check` passes

## Dependencies

- Semantic status kinds (done)
