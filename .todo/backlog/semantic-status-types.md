# Task: Semantic status types

## Context

The current `StatusKind` enum (`Success`, `Error`, `Pending`, `Warning`, `Neutral`) maps directly to 5 UI colors. It works but has two problems:

1. **`Pending` is overloaded.** "Waiting for a reviewer" and "CI is running" are both `Pending` (blue). They mean different things — one requires a human to act, the other just needs time.

2. **Inconsistent vocabulary across feeds.** Same semantic meaning gets different display text (`"passing"` vs `"succeeded"`, `"pending"` vs `"running"`, `"failing"` vs `"failed"`). The current model doesn't normalize this.

A full "semantic type system" was considered but rejected as premature abstraction for 3 feed types. Instead, the simpler approach: **replace `StatusKind` with a new 5-variant enum that answers "who has the ball?"**, plus normalize status strings across feeds.

**Value delivered**: At-a-glance clarity on whether something needs my attention, someone else's attention, or is just a machine doing its thing.

## Design Decisions

### The Model: "Who Has the Ball?"

Each status variant answers one question: **who needs to act next?**

```rust
enum StatusKind {
    AttentionNegative,  // My turn — something's wrong
    AttentionPositive,  // My turn — go do the thing
    Waiting,            // Someone else's turn
    Running,            // Machine working
    Idle,               // Nothing happening
}
```

### Color & Visual Mapping

5 variants, 5 distinct visuals:

| Variant | Color | Visual |
|---|---|---|
| `AttentionNegative` | red | solid dot |
| `Waiting` | yellow | solid dot |
| `Running` | blue | **pulsing** dot |
| `AttentionPositive` | green | solid dot |
| `Idle` | gray | solid dot |

Key change from current model: `Waiting` (was `Pending`) moves from blue to yellow. Blue is now exclusively for machine-in-progress, with a pulse animation to reinforce "actively working."

### Aggregation Precedence

Activity-level dot uses highest-priority variant across all status fields:

```
AttentionNegative > Waiting > Running > AttentionPositive > Idle
```

### Why Not a "Neutral" Sub-flavor?

An `AttentionNeutral` variant was considered for cases like "draft PR" or "agent asked a question" — situations where the user needs to act but nothing is good or bad. It was dropped because:

- It would need a 6th color or share blue with `Running`, which is confusing (one means "sit tight," the other means "your turn").
- These scenarios map cleanly to `AttentionPositive` — "your move, nothing's wrong" is effectively green/"go."
- The label text disambiguates the nature of the action needed.

### Scenario Mapping

Validated against PR and coding agent workflows:

| Scenario | Variant | Dot |
|---|---|---|
| **PR: GitHub / ADO** | | |
| Draft, working on it | `AttentionPositive` | 🟢 |
| Pushed, CI running | `Running` | 🔵💫 |
| CI failed | `AttentionNegative` | 🔴 |
| CI passing, awaiting reviewer | `Waiting` | 🟡 |
| Reviewer requested changes | `AttentionNegative` | 🔴 |
| CI passing, approved, ready to merge | `AttentionPositive` | 🟢 |
| Merge conflict | `AttentionNegative` | 🔴 |
| Approved but blocked by policy | `Waiting` | 🟡 |
| CI running, review approved | `Running` | 🔵💫 |
| Merged / closed | `Idle` | ⚪ |
| **Coding Agent** | | |
| Agent thinking / generating | `Running` | 🔵💫 |
| Agent asked me a question | `AttentionPositive` | 🟢 |
| Agent hit error, needs intervention | `AttentionNegative` | 🔴 |
| Agent finished, ready for review | `AttentionPositive` | 🟢 |
| Agent idle, no task | `Idle` | ⚪ |
| Agent waiting on external API | `Waiting` | 🟡 |

### Resolved Questions

- **Field-level vs activity-level**: Feeds assign semantics per-field as-is. Activity-level dot uses simple highest-priority-wins aggregation (same as current approach). No cross-field reasoning.
- **Shell feed**: Shell feed changes are **out of scope** for this task. The shell feed will keep its current `StatusKind` mapping until a dedicated shell feed effort is done. This task updates `github-pr` and `ado-pr` only.

### Existing Bugs Found During Analysis

These should be fixed as part of (or before) this work:

- GitHub `draft` field is in `provided_fields()` but excluded from tray menu (ADO shows it, GitHub doesn't)
- `github_pr_url_for_id` handles ADO URLs too — misleading function name
- GitHub `mergeable: "unknown"` maps to either blue or gray depending on API source — same display text, different color

### What This Does NOT Include

- **Normalizing status display strings** (e.g., `"passing"` vs `"succeeded"`) — this is a small follow-up, not part of the enum redesign
- **Full "semantic type system"** with per-feed-type vocabulary declarations — rejected as over-engineering for current needs

## Related Files
- `src-tauri/src/feed/mod.rs` — `StatusKind` enum, `FieldValue::Status`
- `src-tauri/src/feed/ado_pr.rs` — ADO PR status field mappings
- `src-tauri/src/feed/github_pr.rs` — GitHub PR status field mappings
- `src-tauri/src/feed/shell.rs` — Shell feed status parsing, `status_severity_from_output`
- `src/App.tsx` — `severityPriority`, `deriveActivitySeverity`, dot rendering
- `src/styles.css` — severity color CSS variables, dot styles
- `specs/main.md` — field type definitions, severity precedence

## Dependencies
- None

## Acceptance Criteria
- [ ] New `StatusKind` enum with 5 variants (`AttentionNegative`, `AttentionPositive`, `Waiting`, `Running`, `Idle`) replacing the current 5
- [ ] Feed implementations (`ado_pr`, `github_pr`) updated to use new variants with correct semantic mappings
- [ ] UI rendering (`App.tsx`, `styles.css`) updated: new colors, pulsing animation for `Running`, updated precedence
- [ ] Spec (`specs/main.md`) updated with the new status model
- [ ] Shell feed continues to work (maps old keywords to new enum, no behavioral changes)
- [ ] Existing behavior preserved (no regressions for current feeds)
- [ ] Bugs fixed: draft field consistency, mergeable "unknown" severity
- [ ] `just check` passes

## Scope Estimate
Medium

## Notes
- The implementation is straightforward: rename the enum variants, update match arms in each feed, update color/symbol mapping and precedence in the frontend.
- The `Running` pulsing animation is a CSS addition (keyframe on the dot when `severity-running`).
