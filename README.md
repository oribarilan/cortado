# Cortado

Cortado is a cross-platform, extensible watcher app.

Users add **Beans** (items they care about), and each Bean can have one or more **Watches**.

Phase 1 focuses on:

- macOS menubar + panel experience
- developer-focused workflows
- `status` watch type first (for example, GitHub PR status)

## Core terms

- **Bean**: a user-defined watch item (for example, a repo or PR stream)
- **Watch**: behavior attached to a Bean (starting with status)

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

## Recommended IDE setup

- [VS Code](https://code.visualstudio.com/)
- [Tauri extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT. See [LICENSE.md](./LICENSE.md).
