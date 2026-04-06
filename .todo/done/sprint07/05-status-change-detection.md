---
status: done
---

# 05 -- Status change detection

## Goal

Implement a diff engine that compares previous and new feed snapshots to detect notifiable status changes. This is the core detection logic -- no notification dispatch yet.

## Change types to detect

1. **Rollup kind change** -- an existing activity's derived rollup status kind changed (e.g., Waiting → AttentionNegative)
2. **New activity** -- an activity appears that wasn't in the previous snapshot
3. **Removed activity** -- an activity disappears from the snapshot (and isn't retained)

## Design

The diff engine should:
- Accept the previous snapshot and new snapshot for a feed
- Compare activities by `id`
- For each existing activity, compute rollup kind (highest-kind-wins across status fields) and compare with previous rollup
- Return a list of `StatusChangeEvent` structs:

```rust
struct StatusChangeEvent {
    feed_name: String,
    activity_id: String,
    activity_title: String,
    change_type: ChangeType,
    previous_kind: Option<StatusKind>,  // None for new activities
    new_kind: Option<StatusKind>,       // None for removed activities
}

enum ChangeType {
    KindChanged,
    NewActivity,
    RemovedActivity,
}
```

## Acceptance criteria

- [ ] `StatusChangeEvent` and `ChangeType` types defined
- [ ] Diff function: `detect_changes(prev: &FeedSnapshot, new: &FeedSnapshot) -> Vec<StatusChangeEvent>`
- [ ] Correctly detects rollup kind changes for existing activities
- [ ] Correctly detects new activities (present in new, absent in previous)
- [ ] Correctly detects removed activities (present in previous, absent in new)
- [ ] Retained activities are excluded from change detection (their status is Idle and stable)
- [ ] Unit tests covering: no changes, kind change, new activity, removed activity, multiple changes, retained activities ignored
- [ ] `just check` passes

## Notes

- The rollup kind computation already exists in the frontend (`deriveActivityKind` in `App.tsx`). This needs a Rust equivalent for the backend. Consider extracting the priority logic into `feed/mod.rs` as a shared function.
- Hook point in `runtime.rs`: after building a new snapshot but before `cache.upsert()`, compare with the cached (previous) snapshot.
- Suppress detection during `seed_startup_best_effort()` -- the first poll establishes the baseline, not a change.

## Relevant files

- `src-tauri/src/feed/mod.rs` -- `StatusKind`, `Activity`, `FeedSnapshot`, `FieldValue`
- `src-tauri/src/feed/runtime.rs` -- poll loop, snapshot cache
- `src/App.tsx` -- `deriveActivityKind` (frontend rollup, reference implementation)
