# US-ux-polish: Better Defaults and User Feedback

## Theme

A bundle of small-to-medium improvements that make Cortado smarter out of the box and give users better signals about what's happening. Covers default settings, auto-generated names, data freshness, and connectivity awareness.

## Task Sequencing

```
01-notify-removed-default-off ──┐
02-default-feed-names ──────────┤
03-activity-last-refreshed ─────┼── independent, parallelizable
06-terminal-tab-icon ───────────┤
07-hide-interval-file-watching ─┤
08-remove-em-dashes ────────────┘
04-offline-indicator ─────────────── standalone, largest feature task
```

Tasks 01-03, 06-08 are independent, small/tiny, and can be done in any order or in parallel. Task 04 is medium-sized -- design decisions are captured in the task file.

## Tasks

| # | File | Size | Summary |
|---|------|------|---------|
| 01 | `01-notify-removed-default-off.md` | Tiny | Default "Removed activities" notification toggle to OFF |
| 02 | `02-default-feed-names.md` | Small | Auto-populate feed names from type + config via `defaultNamePattern` |
| 03 | `03-activity-last-refreshed.md` | Small | Show live-ticking "3m ago" last-refreshed timestamp in detail panes |
| 04 | `04-offline-indicator.md` | Medium | Detect offline via feed failure rollup + ping, show single indicator, pause polling |
| 06 | `06-terminal-tab-icon.md` | Tiny | Replace `▸` with `>_` in Settings nav |
| 07 | `07-hide-interval-file-watching-feeds.md` | Tiny | Hide interval badge on feed cards for file-watching feed types |
| 08 | `08-remove-em-dashes.md` | Small | Replace all em dashes with `--` across the codebase |

## Deferred

- `05-split-bloated-files` -- moved to `.todo/backlog/optional-split-bloated-files.md`
