# Status Model

Cortado's status system has two layers: **status kinds** (semantic, global) and **status values** (display text, per-feed).

## Status Kinds (`StatusKind`)

A status kind answers one question: **who has the ball?**

| Type | Meaning | Color |
|------|---------|-------|
| `AttentionNegative` | My turn -- something's wrong | Red |
| `AttentionPositive` | My turn -- go do the thing | Green |
| `Waiting` | Someone else's turn | Yellow |
| `Running` | Machine working | Pulsing blue |
| `Idle` | Nothing happening | Gray |

Status kinds are global. Their rendering (colors, animation) is defined once in the theme and varies only by light/dark mode. No feed controls how a status kind looks.

Activity-level dot precedence: `AttentionNegative > Waiting > Running > AttentionPositive > Idle` (highest wins across all status fields, no cross-field reasoning).

### Rollup

The same highest-kind-wins algorithm applies at three levels:

- **Fields → Activity dot** -- highest status kind across an activity's status fields. The status chip text also shows the highest-priority field, not a fixed field order.
- **Activities → Tray icon** -- highest rollup kind across every activity in every feed. This is the global at-a-glance signal.

Retained activities always roll up as Idle -- they are no longer actively monitored.
Errored or empty feeds contribute Idle to the global rollup.

## Status Values

A status value is the human-readable text shown alongside a status kind. Values are per-feed -- each feed defines its own vocabulary in code. For example, `github-pr` maps `"approved"` → `AttentionPositive` and `"failing"` → `AttentionNegative`.

The UI renders the value as text (in chips, field rows) and uses the type to determine color and animation.

## Design Rationale

The original model used severity levels (`Success`, `Error`, `Warning`, `Pending`, `Neutral`). This didn't answer the question a developer actually asks when glancing at a menubar: **do I need to do something?** The key problem was `Pending` -- "waiting for a reviewer" and "CI is running" both showed as blue, but mean different things.

The two-layer separation lets each feed use its own domain language while the UI renders a consistent, glanceable color system.

See `.todo/backlog/semantic-status-types.md` for the full design discussion, scenario mapping, and rejected alternatives.

## Implementation

- `src-tauri/src/tray_icon.rs` -- Tray icon compositing, theme detection, global rollup → dot overlay
- `src-tauri/src/feed/mod.rs` -- `StatusKind` enum, `FieldValue::Status { value, kind }`, `rollup_for_activity`, `rollup_for_feeds`
- `src-tauri/src/feed/github_pr.rs` -- GitHub PR value→kind mappings
- `src-tauri/src/feed/ado_pr.rs` -- ADO PR value→kind mappings
- `src-tauri/src/feed/harness/` -- Harness session status inference (working, question, approval, idle)
- `src/App.tsx` -- `kindPriority`, `deriveActivityKind`, rendering
- `src/styles.css` -- Status kind colors, pulse animation, theme variables
