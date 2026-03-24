# Task: Semantic status types

## Context

The current `StatusKind` enum (`Success`, `Error`, `Pending`, `Warning`, `Neutral`) maps directly to 5 UI colors. It works but has two problems:

1. **`Pending` is overloaded.** "Waiting for a reviewer" and "CI is running" are both `Pending` (blue). They mean different things — one requires a human to act, the other just needs time.

2. **Inconsistent vocabulary across feeds.** Same semantic meaning gets different display text (`"passing"` vs `"succeeded"`, `"pending"` vs `"running"`, `"failing"` vs `"failed"`). The current model doesn't normalize this.

A full "semantic type system" was considered but rejected as premature abstraction for 3 feed types. Instead, the simpler approach: **replace `StatusKind` with a new enum that has ~6-7 variants capturing who-needs-to-act**, plus normalize status strings across feeds.

**Value delivered**: At-a-glance clarity on whether something needs my attention, someone else's attention, or is just a machine doing its thing.

## Design Discussion Summary

### The Model: "Who Needs to Act?"

Four categories were identified. The key insight is that each status answers: **who has the ball?**

| # | Category | Who acts? | Temporal feel |
|---|----------|-----------|---------------|
| 1 | **Blocked / waiting on someone else** | Another human or external system | Indefinite — may need nudging |
| 2 | **Waiting on me** | Me | Depends on sub-flavor |
| 3 | **Process running** | A machine | Finite — just wait |
| 4 | **Idle / nothing happening** | Nobody | Stable |

### Category 2 needs sub-flavors

"Waiting on me" is the only category that needs sub-flavors because the *nature* of what's waiting matters:

- **Positive**: Something succeeded and I can take the next step (e.g., review approved → go merge)
- **Negative**: Something failed and I need to fix it (e.g., CI broken, merge conflicts)
- **Neutral**: Just needs doing, no strong signal (e.g., PR is draft, ticket assigned to me)

Categories 1, 3, and 4 don't need sub-flavors. When I'm blocked on someone else, the valence doesn't change what I can do (nothing). Same for process running and idle.

### Scenario Mapping (Pressure Test)

These categories were validated against a wide range of scenarios:

| Scenario | Category |
|----------|----------|
| PR review: awaiting reviewer | 1 — blocked |
| PR review: approved → ready to merge | 2 — me (positive) |
| PR review: changes requested | 2 — me (negative) |
| PR checks: failing | 2 — me (negative) |
| PR checks: running | 3 — process |
| PR checks: passing (review still pending) | 1 — blocked |
| PR checks: passing (everything green) | 2 — me (positive) |
| PR: merge conflict | 2 — me (negative) |
| PR: draft | 2 — me (neutral) |
| Deployment: rolling out | 3 — process |
| Deployment: succeeded | 4 — idle |
| Deployment: failed | 2 — me (negative) |
| Jira ticket: assigned to me, "To Do" | 2 — me (neutral) |
| Jira ticket: blocked by another team | 1 — blocked |
| Jira ticket: closed/done | 4 — idle |
| Build queued, waiting for runner | 3 — process |
| System outage blocking my PR | 1 — blocked |
| PR merged, all done | 4 — idle |

### Open Questions

- **Field-level vs activity-level**: A single field like `checks: passing` could be idle (nothing to do) or contribute to "me-positive" (all green, go merge). The semantic category depends on context. Current thinking: feeds assign semantics per-field, the UI aggregates across fields for the activity-level dot. But this needs more thought.
- **Idle sub-types**: "Idle — everything is fine" vs "idle — completed/archived" — probably not worth distinguishing for a menubar app, but flagged.
- **Shell feed compatibility**: Users already pick values + severities for shell feeds. The new enum replaces `StatusKind` 1:1 in config, so shell feeds just get more options. No third concept layer needed.

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
- `src-tauri/src/feed/shell.rs` — Shell feed status parsing
- `src-tauri/src/tray.rs` — tray rendering of status fields, dot color logic
- `specs/main.md` — field type definitions

## Dependencies
- None

## Acceptance Criteria
- [ ] New `StatusKind` enum with ~6-7 variants replacing the current 5
- [ ] Feed implementations (`ado_pr`, `github_pr`, `shell`) updated to use new variants
- [ ] UI rendering (`tray.rs`) updated with distinct visual treatment per variant
- [ ] Spec (`specs/main.md`) updated with the new status model
- [ ] Shell feed config supports the new variants (backward compatible)
- [ ] Existing behavior preserved (no regressions for current feeds)
- [ ] Bugs fixed: draft field consistency, mergeable "unknown" severity
- [ ] `just check` passes

## Scope Estimate
Medium

## Notes
- Terminology for the enum variants is TBD — the semantic categories are agreed, naming is next.
- The implementation is straightforward: expand the enum, update the match arms in each feed, update the color/symbol mapping in tray.rs.
- Consider whether the activity-level dot aggregation logic (`infer_status` in tray.rs) needs a new precedence order for the expanded variants.
