# Glossary

Canonical definitions for project terminology. When introducing a new term, define it here first.

## Core Model

| Term | Definition |
|------|-----------|
| **Feed** | A configured data source that discovers and tracks related items. Example: "GitHub PRs for personal/cortado". Not to be confused with a single item -- that's an Activity. |
| **Activity** | An individual tracked item within a feed, discovered and managed by the feed's lifecycle. Example: PR #42 "Add feed scaffold". |
| **Field** | A typed, structured piece of data on an activity. Fields have a name, label, value, and field type. Example: `review: awaiting` (a status field). |
| **Retained Activity** | An activity no longer returned by a feed's latest poll, kept visible for a configured retention duration. |
| **Harness** | A terminal-based AI coding agent (e.g., GitHub Copilot CLI, OpenCode). The `HarnessProvider` trait abstracts session discovery; `HarnessFeed` is the generic Feed impl. All harness feeds use `GenericProvider` backed by the interchange format (`~/.config/cortado/harness/`, see `specs/harness-interchange.md`). Agent-specific logic lives in plugins that write interchange files (OpenCode plugin, Copilot CLI plugin). Adding a new agent = one line of Rust (`GenericProvider::new("name")`) + a plugin that writes state files. |

## Status Model

See `specs/status.md` for the full status model spec.

| Term | Definition |
|------|-----------|
| **Status Kind** | Semantic classification of a status field -- answers "who has the ball?" One of: `AttentionNegative`, `AttentionPositive`, `Waiting`, `Running`, `Idle`. Controls dot color and animation. In code: `StatusKind` enum, `kind` field. |
| **Status Value** | Feed-specific display text for a status field (e.g., "approved", "failing", "awaiting"). Each feed defines its own vocabulary and maps values to status kinds in code. In code: `value` field on `FieldValue::Status`. |

## UI

| Term | Definition |
|------|-----------|
| **Tray** | The menu opened by left-clicking the menubar icon. Shows feeds and activities in a compact list with inline disclosure. |
| **Panel** | The main app window -- a floating, non-activating NSPanel opened by global hotkey. Split layout: list pane + detail pane. View-specific settings live under `[panel]` in `settings.toml`. |
| **Settings** | The standard decorated window for configuring feeds, notifications, and general preferences. Opened from the panel footer or `⌘,`. |

## Deprecated Terms

| Old Term | Replacement |
|----------|------------|
| Bean | Feed |
| Watch | Activity |
| Severity | Status Kind |
| Main Screen | Panel |
| Menubar | Tray |
