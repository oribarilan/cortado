---
status: done
---

# 03 — Notification config types

## Goal

Define the Rust types that represent notification configuration and wire them into the `settings.toml` parsing from task 01.

## Context

Notification behavior is controlled by several settings:

1. **Master toggle** — enable/disable all notifications
2. **Notification mode** — which status changes trigger notifications
3. **Delivery preset** — how notifications are batched
4. **New activity toggle** — notify when new activities appear
5. **Removed activity toggle** — notify when activities disappear

## Types to define

```rust
enum NotificationMode {
    All,                           // Any rollup kind change
    EscalationOnly,                // Only when rollup moves higher in priority
    SpecificKinds(Vec<StatusKind>), // Only when rollup lands on these kinds
}

enum DeliveryPreset {
    Immediate,  // One notification per activity change
    Grouped,    // At most one per feed per poll cycle (default)
}

struct NotificationSettings {
    enabled: bool,                   // Master toggle (default: true)
    mode: NotificationMode,          // Default: All
    delivery: DeliveryPreset,        // Default: Grouped
    notify_new_activities: bool,     // Default: true
    notify_removed_activities: bool, // Default: true
}
```

## Acceptance criteria

- [ ] `NotificationMode`, `DeliveryPreset`, `NotificationSettings` types defined
- [ ] Serde Serialize/Deserialize implemented with sensible defaults
- [ ] Integrated into `AppSettings` from task 01 (nested under `[notifications]` in TOML)
- [ ] Defaults: enabled=true, mode=All, delivery=Grouped, notify_new=true, notify_removed=true
- [ ] Round-trip test: serialize → deserialize preserves all values
- [ ] Missing `[notifications]` section in TOML → all defaults applied
- [ ] `just check` passes

## Notes

- `SpecificKinds` needs careful TOML representation. Consider a list of kind names:
  ```toml
  [notifications]
  mode = "specific"
  specific_kinds = ["attention-negative", "attention-positive"]
  ```
- The `EscalationOnly` mode uses the existing priority order: `AttentionNegative > Waiting > Running > AttentionPositive > Idle`
- These types will be consumed by the dispatch logic (task 06) and exposed in the settings UI (task 08).
- Digest delivery preset is deferred to backlog — only Immediate and Grouped are in scope.

## Relevant files

- `src-tauri/src/feed/mod.rs` — `StatusKind` enum (reuse for `SpecificKinds`)
- Task 01 output — `AppSettings` struct and settings.toml parser
