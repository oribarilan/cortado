# Glossary

Canonical definitions for project terminology. When introducing a new term, define it here first.

## Core Model

| Term | Definition |
|------|-----------|
| **Feed** | A configured data source that discovers and tracks related items. Example: "GitHub PRs for personal/cortado". Not to be confused with a single item — that's an Activity. |
| **Activity** | An individual tracked item within a feed, discovered and managed by the feed's lifecycle. Example: PR #42 "Add feed scaffold". |
| **Field** | A typed, structured piece of data on an activity. Fields have a name, label, value, and field type. Example: `review: awaiting` (a status field). |
| **Retained Activity** | An activity no longer returned by a feed's latest poll, kept visible for a configured retention duration. |

## Status Model

See `specs/status.md` for the full status model spec.

| Term | Definition |
|------|-----------|
| **Status Kind** | Semantic classification of a status field — answers "who has the ball?" One of: `AttentionNegative`, `AttentionPositive`, `Waiting`, `Running`, `Idle`. Controls dot color and animation. In code: `StatusKind` enum, `kind` field. |
| **Status Value** | Feed-specific display text for a status field (e.g., "approved", "failing", "awaiting"). Each feed defines its own vocabulary and maps values to status kinds in code. In code: `value` field on `FieldValue::Status`. |

## Deprecated Terms

| Old Term | Replacement |
|----------|------------|
| Bean | Feed |
| Watch | Activity |
| Severity | Status Kind |
