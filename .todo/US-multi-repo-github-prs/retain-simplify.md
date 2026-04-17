# retain-simplify

## Context

The retain field is currently an optional duration input (number + unit). Most users either want completed items cleared immediately or kept for some time. The current UI doesn't make the common case (clear immediately) obvious — it's just an empty optional field.

**Value delivered**: Retain behavior is immediately understandable with a clear toggle, reducing cognitive load.

## Related Files

- `src/settings/SettingsApp.tsx` — retain field rendering (~lines 1919-1930)
- `src/shared/feedTypes.ts` — feed catalog (retain is not per-type, it's generic)
- `src-tauri/src/feed/config.rs` — retain config parsing

## Dependencies

- None

## Design Exploration

### Current UX

- Optional `DurationInput` labeled "Retain" with hint "Keep completed items for"
- Empty = items cleared immediately (verified: `retain = None` in backend means no retention)
- If set, items are kept for the specified duration after completion

### Proposed UX

**"Clear completed items immediately"** toggle, ON by default.

- **Toggle ON** (default): Completed items are removed immediately. No duration input shown. Config: `retain` is absent.
- **Toggle OFF**: A duration input appears below the toggle. User sets how long to keep completed items. Config: `retain = "2h"` etc.

This makes the default behavior explicit and the progressive disclosure natural. Toggle ON = common case = simpler state.

### Config mapping

| Toggle state | Duration | TOML |
|---|---|---|
| ON (clear immediately) | — | `retain` absent |
| OFF | user-specified | `retain = "2h"` |

No backend changes needed — the existing `retain: Option<Duration>` handles both cases. The change is purely frontend UX.

## Acceptance Criteria

- [ ] Retain field is replaced with a "Clear completed items immediately" toggle
- [ ] Toggle is ON by default (for new feeds)
- [ ] When toggle is OFF, a duration input appears for specifying retain duration
- [ ] When toggle is ON, no duration input is shown, and `retain` is omitted from config
- [ ] Existing feeds with `retain` set load correctly (toggle OFF, duration populated)
- [ ] Existing feeds without `retain` load correctly (toggle ON)
- [ ] No backend changes required
- [ ] `just check` passes

## Verification

- **Ad-hoc**: Add new feed → verify toggle is ON, no duration input → toggle OFF → verify duration input appears → set "2h" → save → reopen → verify toggle is OFF with "2h" populated
- **Ad-hoc**: Edit existing feed with `retain = "2h"` → verify toggle is OFF with "2h" shown → toggle ON → save → verify `retain` is removed from config
