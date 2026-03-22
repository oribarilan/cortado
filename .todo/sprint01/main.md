---
status: in-progress
---

# Sprint 01 — Feed system foundation

## Theme

Stand up the core feed system: the Feed trait, config parsing, and wiring to the frontend. By the end of this sprint, Cortado loads feeds from `~/.config/cortado/feeds.toml` and displays them in the panel with real data flowing from Rust to React.

## Sequencing

```
01-feed-trait  ──────────┐
                         ├──> 05-registry-wiring ──> 06-frontend-update
02-config-parsing ───────┤
03-github-pr-stub ───────┤
04-shell-stub ───────────┘
```

- Task 01 must come first — everything depends on the core types.
- Tasks 02, 03, 04 are parallelizable once 01 is done.
- Task 05 depends on all of 01–04 (wires them together).
- Task 06 depends on 05 (frontend consumes the Tauri command).

## Cross-task notes

- **Terminology**: this sprint replaces all Bean/Watch references with Feed/Activity. See `specs/main.md` for definitions.
- **Dependencies**: new Rust crates needed — `toml`, `async-trait`, `anyhow`, `dirs`. Add them in task 01 so they're available for later tasks.
- **Stubs, not real IO**: github-pr and shell feeds return hardcoded data. Real API calls and command execution come in sprint 02.
- **Config file**: `~/.config/cortado/feeds.toml`. If missing, Cortado starts with no feeds (empty state).
- **Error surfacing**: errors (bad config, poll failures) are never swallowed. They surface per-feed in the UI. `FeedSnapshot` carries an `error` field for this — defined in task 01, used in tasks 05 and 06.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-feed-trait.md` | Feed trait, core types (Activity, Field, FieldValue), add Rust deps |
| 02 | `02-config-parsing.md` | TOML config parsing, load feeds from `~/.config/cortado/feeds.toml` |
| 03 | `03-github-pr-stub.md` | GitHub PR feed — implements Feed trait with hardcoded data |
| 04 | `04-shell-stub.md` | Shell feed — implements Feed trait with hardcoded data |
| 05 | `05-registry-wiring.md` | FeedRegistry as Tauri state, `list_feeds` command, wire into main.rs |
| 06 | `06-frontend-update.md` | Rename Bean/Watch to Feed/Activity in frontend, fetch from backend |
