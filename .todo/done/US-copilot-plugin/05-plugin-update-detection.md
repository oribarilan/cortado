---
status: done
---

# Add copilot extension update detection to CortadoUpdateFeed

## Goal

Extend the Cortado update feed to detect when the on-disk Copilot CLI extension is outdated compared to the version embedded in the binary, and surface an update activity. Follows the same pattern as the existing OpenCode plugin update detection.

## Acceptance criteria

- [ ] `cortado_update.rs` checks for outdated Copilot extension when a `copilot-session` feed is configured
- [ ] Uses `copilot_extensions_dir()` to locate the on-disk extension file
- [ ] Compares versions via `parse_plugin_version()` / `is_plugin_outdated()` (same helpers as OpenCode)
- [ ] When outdated, surfaces an activity like "Copilot CLI extension update available" with `AttentionPositive` status
- [ ] When up-to-date or not installed, no activity is surfaced
- [ ] `CortadoUpdateFeed` constructor updated to accept copilot extension check flag (currently only takes `check_opencode_plugin: bool`)
- [ ] `main.rs` updated: checks if any configured feed has type `copilot-session`, passes the flag to `CortadoUpdateFeed::new()`
- [ ] Pattern matches the existing OpenCode update detection logic in `cortado_update.rs`

## Notes

### Wiring in main.rs

The current code passes a single boolean:
```rust
let has_opencode_feed = feed_configs.iter().any(|c| c.feed_type == "opencode-session");
feed_registry.register(Arc::new(CortadoUpdateFeed::new(has_opencode_feed)));
```

This needs to expand to also check for `copilot-session`:
```rust
let has_opencode_feed = feed_configs.iter().any(|c| c.feed_type == "opencode-session");
let has_copilot_feed = feed_configs.iter().any(|c| c.feed_type == "copilot-session");
feed_registry.register(Arc::new(CortadoUpdateFeed::new(has_opencode_feed, has_copilot_feed)));
```

Consider whether `CortadoUpdateFeed::new()` should take a struct or just add a second bool. Keep it simple -- a second parameter is fine for two cases. If a third agent is added later, refactor to a struct then.

### Existing pattern

Look at how `cortado_update.rs` handles OpenCode plugin updates. The Copilot extension check should follow the same structure:

1. Check if any configured feed has type `copilot-session`
2. If yes, check if `~/.copilot/extensions/cortado/extension.mjs` exists
3. If exists, compare version headers
4. If outdated, add an update activity to the feed snapshot
