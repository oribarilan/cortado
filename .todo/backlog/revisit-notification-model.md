---
status: pending
---

# Revisit notification model across all feed types

## Context

The current notification model is built around **rollup kind changes** — when an activity's highest-priority `StatusKind` changes between polls, that can trigger a notification. The `NotificationMode` options are:

- `All` — any kind change fires
- `EscalationOnly` — only when new kind has higher priority than previous
- `SpecificKinds` — only when new kind is in a configured set

The priority order is: `AttentionNegative > Waiting > Running > AttentionPositive > Idle`.

### The problem

None of the current modes match user intent:

- **`All`** is noisy — `Idle → Running` fires when you just triggered the agent or pushed code.
- **`EscalationOnly`** misses completions — `Working → Idle` (agent finished), `Running → Idle` (build passed), `AttentionNegative → Idle` (site recovered) are all silent because they're de-escalations.
- **`SpecificKinds`** requires thinking in StatusKind terms, not user intent.

### Root cause

`EscalationOnly` uses the severity priority axis, but user interest doesn't align with severity. "Agent finished" is low-severity but high-interest. "Agent started" is medium-severity but low-interest (you just triggered it).

### Transition audit (all feed types)

**Coding agents (harness feeds):**

| Transition | `escalation_only` | User wants |
|---|---|---|
| Idle → Working (Running) | Notifies | **No** — I just triggered it |
| Working → Idle | Silent | **Yes** — it finished! |
| Working → Question (AttentionPositive) | Notifies | **Yes** — needs my input |
| Question → Idle | Silent | **Yes** — session ended |
| Question → Working (Running) | Silent | **No** — it resumed |

**GitHub Actions:**

| Transition | `escalation_only` | User wants |
|---|---|---|
| Idle → Running | Notifies | **No** — I just pushed |
| Running → Idle | Silent | **Yes** — build passed |
| Running → AttentionNegative | Notifies | **Yes** — build failed |
| AttentionNegative → Running | Silent | **No** — re-run started |
| AttentionNegative → Idle | Silent | **Yes** — fixed! |

**HTTP Health:**

| Transition | `escalation_only` | User wants |
|---|---|---|
| Idle → AttentionNegative | Notifies | **Yes** — site down |
| AttentionNegative → Idle | Silent | **Yes** — site recovered |

**PR feeds (GitHub/ADO):**

| Transition | `escalation_only` | User wants |
|---|---|---|
| Idle → Waiting | Notifies | **No** — I opened the PR |
| Waiting → AttentionPositive | Notifies | **Yes** — approved, my turn |
| Waiting → AttentionNegative | Notifies | **Yes** — checks failed |
| Waiting → Running | Silent | **No** — checks started |
| Running → Waiting | Silent | **No** — checks done, still waiting |
| Running → Idle | Silent | **Yes** — checks passed |
| AttentionPositive → Idle | Silent | **Yes** — PR merged |
| AttentionNegative → Idle | Silent | **Yes** — problem resolved |

### The pattern

Users care about **where things land**, not where they start. Notify when something reaches a resting state (Idle) or needs attention (AttentionPositive, AttentionNegative). Don't notify when entering a transient in-progress state (Running, Waiting).

This aligns with the StatusKind "who has the ball?" model:
- Idle = ball landed, nobody's turn → something resolved
- AttentionPositive/AttentionNegative = my turn → I need to act
- Running = machine working → ball is in the air
- Waiting = someone else's turn → ball is in the air

## Design

### New notification modes

| Mode | Config value | Rule | Use case |
|------|-------------|------|----------|
| **Worth Knowing** (NEW, **default**) | `worth_knowing` | Notify when `new_kind ∈ {Idle, AttentionPositive, AttentionNegative}` | Most feeds — "tell me when things finish or need me" |
| **Need Attention** (NEW) | `need_attention` | Notify when `new_kind ∈ {AttentionPositive, AttentionNegative}` | High-traffic feeds — "only when it's my turn" |
| All | `all` | Any kind change | Small, low-traffic feeds where every change matters |
| Specific kinds | `specific_kinds` | `new_kind` in configured set | Power-user escape hatch (keep, don't promote in UI) |

**Remove `escalation_only`** — it's conceptually wrong for every feed type. No migration needed (no existing users). If the config contains `escalation_only`, treat it as an invalid value and raise a parse error, same as any other unexpected format.

The `worth_knowing` definition is **global**, operating on the StatusKind semantic layer. Per-feed differences are already handled one layer down — each feed maps its domain concepts to StatusKinds. The notification rule doesn't need feed-specific logic.

### Specific Kinds UI

When "Specific kinds" is selected (in either the global or per-feed radio group), a row of kind chip toggles appears:

- **Attention** — maps to both `AttentionPositive` and `AttentionNegative` internally (always toggled together)
- **Waiting**
- **Running**
- **Idle**

Four chips, not five. The Attention+/Attention- distinction is an implementation detail users don't need to manage.

### Verification: `worth_knowing` mode against all feed types

| Feed type | Transition | new_kind | Notifies? | Correct? |
|---|---|---|---|---|
| Harness | Idle → Working | Running | No | ✓ |
| Harness | Working → Idle | Idle | Yes | ✓ |
| Harness | Working → Question | AttentionPositive | Yes | ✓ |
| Harness | Question → Working | Running | No | ✓ |
| Actions | Idle → Running | Running | No | ✓ |
| Actions | Running → Idle | Idle | Yes | ✓ |
| Actions | Running → AttentionNegative | AttentionNeg | Yes | ✓ |
| Actions | AttentionNegative → Idle | Idle | Yes | ✓ |
| Health | Idle → AttentionNegative | AttentionNeg | Yes | ✓ |
| Health | AttentionNegative → Idle | Idle | Yes | ✓ |
| PR | Idle → Waiting | Waiting | No | ✓ |
| PR | Waiting → AttentionPositive | AttentionPos | Yes | ✓ |
| PR | Waiting → AttentionNegative | AttentionNeg | Yes | ✓ |
| PR | Running → Idle | Idle | Yes | ✓ |
| PR | AttentionNegative → Idle | Idle | Yes | ✓ |
| PR | AttentionPositive → Idle | Idle | Yes | ✓ |

One model, all feed types, no per-feed special casing.

### Per-feed mode override

Extend `notify` from `bool` to `bool | mode_name`:

```toml
[[feed]]
name = "Production health"
type = "http-health"
notify = "all"              # override: every change matters

[[feed]]
name = "Agent sessions"
type = "copilot"
notify = true               # uses global mode (worth_knowing)

[[feed]]
name = "Noisy monorepo PRs"
type = "github-pr"
repo = "org/mono"
notify = "need_attention"   # only when it's my turn

[[feed]]
name = "Logs feed"
type = "http-health"
notify = false              # suppress entirely
```

Backward compatible — `true`/`false` still work. `notify = true` means "use global mode"; `notify = "worth_knowing"` is an explicit per-feed override (distinct from `true` when the global mode is something else). In Rust, model as an untagged serde enum.

### Runtime resolution

The dispatch pipeline resolves the effective mode for each feed:

1. `feed_notify_map` in `NotificationContext` changes from `HashMap<String, bool>` to `HashMap<String, FeedNotifyOverride>` where `FeedNotifyOverride` is `Off | Global | Mode(NotificationMode)`.
2. `process_feed_update` resolves: `effective_mode = match per_feed { Off => skip, Global => global_mode, Mode(m) => m }`.
3. `matches_mode()` evaluates against the resolved effective mode.

### First-poll behavior

The new modes only apply to `KindChanged` events. On first poll (or when a feed is added), activities arrive as `NewActivity` events, which are controlled by the separate `notify_new_activities` toggle, not the notification mode. The mode has no effect on first-poll behavior.

### Settings UI

**Global mode** (Notifications tab): Radio group with descriptions for each mode. Same visual pattern as the current UI, with updated mode names and descriptions. Selecting "Specific kinds" reveals kind chip toggles below it.

**Per-feed** (feed edit form): Two toggles + expandable radio group:
1. **Notifications** — on/off toggle (existing). Controls whether this feed sends notifications at all.
2. **Use specific notification settings for this feed** — toggle (default: off). When off, the hint reads "Uses global mode (Worth Knowing)". When on, reveals the same 4-option radio group as the global mode section. Selecting "Specific kinds" reveals kind chips.

The "Notifications" toggle disables the feed-specific toggle when off (grayed out, no interaction).

Showcase: `showcases/notification-mode-showcase.html`

## Goal

Replace the notification mode system so the default behavior matches user intent across all feed types: notify on completion and attention, not on start.

## Value delivered

Users get notified about the events they actually care about (agent finished, build completed, site recovered) without having to use `All` mode and tolerate noise.

## Related files

- `src-tauri/src/notification/dispatch.rs` — dispatch pipeline, `matches_mode()`
- `src-tauri/src/notification/change_detection.rs` — change detection logic
- `src-tauri/src/notification/content.rs` — notification formatting
- `src-tauri/src/app_settings.rs` — NotificationSettings, NotificationMode, DeliveryPreset
- `src-tauri/src/feed/config.rs` — per-feed `notify` toggle (currently bool)
- `src/settings/SettingsApp.tsx` — notification settings UI (global mode + per-feed toggle)
- `specs/main.md` (lines 374-451) — notification spec section
- `specs/status.md` — StatusKind semantics and precedence

## Acceptance criteria

- [ ] `worth_knowing` mode added and set as default
- [ ] `need_attention` mode added
- [ ] `escalation_only` removed (parse error on unknown mode values)
- [ ] Per-feed `notify` accepts mode name in addition to bool
- [ ] Per-feed runtime resolution: `FeedNotifyOverride` enum, `process_feed_update` resolves effective mode
- [ ] Specific Kinds UI uses 4 chips (Attention, Waiting, Running, Idle) — Attention maps to both +/-
- [ ] Settings UI: global radio group + per-feed two-toggle design with expandable radios
- [ ] Spec updated to reflect the new model
- [ ] Existing tests updated, new tests for new modes
- [ ] `just check` passes

## Scope Estimate

Medium

## Notes

- This may subsume or interact with `optional-notification-digest.md` (digest delivery preset).
- `specific_kinds` is kept as an advanced option but not promoted in the UI.
- Settings UI changes are included in this task scope (documented in showcase).
