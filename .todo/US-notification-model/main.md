# US: Notification Model Redesign

## Theme

Replace the notification mode system so the default behavior matches user intent across all feed types: notify on completion and attention, not on start. Add per-feed mode overrides so individual feeds can deviate from the global mode.

## Current state

The notification model offers three modes: `All` (noisy), `EscalationOnly` (misses completions), and `SpecificKinds` (requires thinking in StatusKind terms). None match what users actually want. The per-feed `notify` field is a simple bool toggle.

## Design decisions (from review)

- **Mode names**: Worth Knowing (`worth_knowing`), Need Attention (`need_attention`), All (`all`), Specific Kinds (`specific_kinds`).
- **Remove `escalation_only`** entirely (parse error, no migration -- no existing users).
- **Specific Kinds UI**: 4 chips (Attention, Waiting, Running, Idle). Attention maps to both AttentionPositive and AttentionNegative internally.
- **Per-feed override**: `notify` field accepts `bool | mode_name`. Runtime resolution via `FeedNotifyOverride` enum.
- **Per-feed UI**: Two toggles (Notifications on/off + "Use specific notification settings for this feed") with expandable radio group matching the global mode options.
- **First-poll**: unaffected -- governed by `notify_new_activities`, not the mode.

Full analysis and transition audit: `.todo/backlog/revisit-notification-model.md`
Showcase: `showcases/notification-mode-showcase.html`

## Sequencing

Tasks are sequential. Task 01 (spec) establishes the source of truth. Task 02 (backend modes) is the foundation. Task 03 (per-feed override) builds on 02. Tasks 04 and 05 (frontend) can be parallelized after 03 is done. Task 06 (verify) is last.
