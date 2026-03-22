# Cortado

Cortado is a cross-platform extensible watcher that lives in the macOS menubar.

Users configure **feeds** (data sources like "GitHub PRs for repo X"), and each feed automatically discovers and tracks **activities** (e.g., individual PRs). Each activity has structured **fields** showing its current state.

Phase 1 focuses on:

- macOS menubar + panel experience
- Developer-focused workflows
- GitHub PR feed (first curated feed type)
- Shell feed (user-defined commands as an escape hatch)

## Core terms

- **Feed**: a configured data source that discovers and tracks related items
- **Activity**: one tracked item within a feed (e.g., PR #42)
- **Field**: a typed data point on an activity (e.g., `review: awaiting`)

See `specs/main.md` for the full spec.

## Development

### Prerequisites

- [Node.js](https://nodejs.org)
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Commands

```bash
just          # list commands
just install  # install JS deps
just dev      # run app locally
just check    # format + lint
```

## License

MIT. See [LICENSE.md](./LICENSE.md).
