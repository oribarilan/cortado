# US-ux-polish: Better Defaults and User Feedback

## Theme

A bundle of small-to-medium improvements that make Cortado smarter out of the box and give users better signals about what's happening. Covers default settings, auto-generated names, data freshness, and connectivity awareness.

## Task Sequencing

```
01-notify-removed-default-off ──┐
02-default-feed-names ──────────┼── independent, parallelizable
03-activity-last-refreshed ─────┘
04-offline-indicator ─────────────── standalone, largest task
```

Tasks 01–03 are independent, small, and can be done in any order. Task 04 is medium-sized with open design questions — tackle it last.

## Tasks

| # | File | Size | Summary |
|---|------|------|---------|
| 01 | `01-notify-removed-default-off.md` | Small | Default "Removed activities" notification toggle to OFF |
| 02 | `02-default-feed-names.md` | Small | Auto-populate feed names from type + config |
| 03 | `03-activity-last-refreshed.md` | Small | Show "3m ago" last-refreshed timestamp in detail panes |
| 04 | `04-offline-indicator.md` | Medium | Detect offline state, show single indicator instead of N feed errors |
