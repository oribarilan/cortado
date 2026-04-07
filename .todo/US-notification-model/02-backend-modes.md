---
status: done
---

# Backend: new notification modes

## Goal

Replace the `NotificationMode` enum with the new modes and update the dispatch pipeline to match against them.

## Acceptance criteria

- [ ] `NotificationMode` enum has variants: `WorthKnowing`, `NeedAttention`, `All`, `SpecificKinds { kinds: Vec<StatusKind> }`
- [ ] `WorthKnowing` is the default
- [ ] `EscalationOnly` variant removed -- unknown mode values produce a parse error
- [ ] `matches_mode()` in dispatch.rs implements the new rules:
  - `WorthKnowing`: `new_kind ∈ {Idle, AttentionPositive, AttentionNegative}` (false when `new_kind` is `None`)
  - `NeedAttention`: `new_kind ∈ {AttentionPositive, AttentionNegative}` (false when `new_kind` is `None`)
  - `All`: any kind change
  - `SpecificKinds`: `new_kind` in configured set
- [ ] Serde serialization uses `worth_knowing`, `need_attention`, `all`, `specific_kinds`
- [ ] `app_settings.rs` default updated
- [ ] Existing notification tests updated, new tests for `WorthKnowing` and `NeedAttention`
- [ ] `just check` passes

## Related files

- `src-tauri/src/app_settings.rs` -- `NotificationMode` enum, `NotificationSettings` defaults
- `src-tauri/src/notification/dispatch.rs` -- `matches_mode()` function
- `src-tauri/src/notification/` -- test files
