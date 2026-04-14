# Rework notification formatting

## Context

Notification body text uses `--` as a separator and has inconsistent formatting across event types. This task reworks all notification formatting for clarity and consistency.

Depends on `01-status-value-in-notifications.md` (uses the status value it threads through).

**Value delivered**: Notifications are cleaner, more readable at a glance, and consistent across all event types.

## Related Files

- `src-tauri/src/notification/content.rs` -- `format_single()`, `format_grouped()`, and their tests

## Dependencies

- `01-status-value-in-notifications.md` must be done first

## Formatting Spec

### Status changed (single)

**Before:** `Add notifications -- needs attention`
**After:** `Add notifications → working`

- Use `→` (Unicode right arrow) as separator between activity title and status value
- Status value comes from task 01; fall back to `StatusKind::human_name()` if absent

### New activity (single)

**Before:** `New: Fix bug`
**After:** `+ Fix bug`

- Replace `New:` prefix with `+` (implies addition, visually distinct from `→` used for status changes)

### Removed activity (single)

**Before:** `Gone: Old PR`
**After:** `Removed: Old PR`

- Replace `Gone:` with `Removed:` (neutral -- doesn't imply a specific outcome like merged or closed)

### Grouped (multiple changes in one feed)

**Before:**
```
3 activities changed
PR Alpha, PR Beta, PR Gamma
```

**After (example):**
```
Add notifications → working (+2 more)
```

- Format the first event in the slice as a normal single notification
- Append ` (+N more)` if there are additional events
- If only 1 event, delegate to `format_single()` (unchanged behavior)

## Acceptance Criteria

- [ ] `format_single()` uses `→` separator for `KindChanged` events
- [ ] `format_single()` uses `+` prefix for `NewActivity` events
- [ ] `format_single()` uses `Removed:` prefix for `RemovedActivity` events
- [ ] `format_grouped()` shows the first event formatted normally with `(+N more)` suffix when there are multiple events
- [ ] All existing tests updated to match new formatting and pass
- [ ] `just check` passes cleanly

## Verification

- **Automated**: `just check` passes
- **Ad-hoc**: Trigger various notification types and confirm formatting matches spec above
