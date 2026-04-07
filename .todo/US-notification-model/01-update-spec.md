---
status: pending
---

# Update spec with new notification model

## Goal

Update `specs/main.md` notification section to reflect the new mode names, per-feed override semantics, and removal of `escalation_only`. The spec is the source of truth -- it must be updated before implementation begins.

## Acceptance criteria

- [ ] Notification mode table updated: `worth_knowing` (default), `need_attention`, `all`, `specific_kinds`
- [ ] `escalation_only` removed from spec entirely
- [ ] Per-feed `notify` field documented as `bool | mode_name` with semantics for each value
- [ ] Runtime resolution behavior documented (FeedNotifyOverride: Off / Global / Mode)
- [ ] Specific Kinds described with 4-chip UI (Attention maps to both +/-)
- [ ] First-poll behavior clarified (governed by `notify_new_activities`, not mode)

## Related files

- `specs/main.md` (lines 374-451) -- notification spec section
- `.todo/backlog/revisit-notification-model.md` -- full design doc with transition audit
