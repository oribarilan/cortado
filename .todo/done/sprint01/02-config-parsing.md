---
status: done
---

# Config parsing

## Goal

Parse `~/.config/cortado/feeds.toml` into structured Rust types. The config module reads the file and produces a list of `FeedConfig` values that can be used to instantiate feed implementations.

## Acceptance criteria

- [ ] `src-tauri/src/feed/config.rs` exists with `FeedConfig` and `load_feeds_config()` function
- [ ] `FeedConfig` captures: `name`, `type`, `interval` (optional), type-specific fields (as `toml::Table`), field overrides
- [ ] `load_feeds_config()` reads from `~/.config/cortado/feeds.toml`
- [ ] Missing config file returns an empty list (not an error)
- [ ] Invalid TOML returns a meaningful error
- [ ] Duplicate feed names return a meaningful error
- [ ] Config can represent both github-pr and shell feed definitions
- [ ] `just check` passes

## Notes

- Config path: `~/.config/cortado/feeds.toml`. Use `dirs::home_dir()` + `.config/cortado/feeds.toml`. Don't use platform-specific app support dirs -- `~/.config` is the intended location (dev-tool convention).
- Type-specific fields are flat in the TOML (not nested under `[feed.config]`). The config parser collects known common fields (`name`, `type`, `interval`) and passes the rest as a `toml::Table` for the feed implementation to interpret.
- Field overrides use `[feed.fields.<name>]` with optional `visible` and `label`.

## Example config

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "personal/cortado"
interval = 60

[feed.fields.labels]
visible = false

[[feed]]
name = "Disk usage"
type = "shell"
command = "df -h / | tail -1 | awk '{print $5}'"
interval = 30
```

## Relevant files

- `src-tauri/src/feed/config.rs` -- new file
- `src-tauri/src/feed/mod.rs` -- add `pub mod config;`
