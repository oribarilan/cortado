---
status: done
---

# Register opencode-session feed type

## Goal

Wire the `GenericProvider` into Cortado's feed system so users can configure `opencode-session` feeds in their `feeds.toml`. Also integrate the plugin's build/test into the root justfile.

## Acceptance criteria

- [ ] `instantiate_harness_feed()` in `feed/mod.rs` handles `"opencode-session"` feed type
- [ ] Creates `GenericProvider::new("opencode")` wrapped in `HarnessFeed::from_config()`
- [ ] `build_feed_registry_from_configs()` routes `opencode-session` through harness path
- [ ] Default interval is appropriate (30s, matching copilot-session)
- [ ] Users can configure the feed in `feeds.toml`:
  ```toml
  [[feed]]
  name = "OpenCode Sessions"
  type = "opencode-session"
  ```
- [ ] Feed appears in the frontend feed catalog (`src/shared/feedTypes.ts`)
- [ ] Root `justfile` updated: `check`, `test`, `format` cascade into `plugins/opencode/justfile`
- [ ] `just check` passes (including plugin check)

## Notes

This should be minimal wiring -- the `GenericProvider` and `HarnessFeed` do all the heavy lifting. The main work is adding match arms and a catalog entry.

### Harness type routing

Currently `build_feed_registry_from_configs()` has an explicit `if feed_type == "copilot-session"` check to route through the harness path. Rather than adding another `||` arm for `"opencode-session"`, consider making this data-driven (e.g., `instantiate_harness_feed()` returns `Option`, and the caller uses that to decide routing). This avoids growing a list of hardcoded harness type strings.

Follow the pattern used for `copilot-session` in `instantiate_harness_feed()`.

### Justfile cascade

The root `justfile` should call the plugin's justfile for the corresponding recipe. Example:

```just
check: format lint test
    just -f plugins/opencode/justfile check

test:
    cargo test --no-default-features
    just -f plugins/opencode/justfile test
```

This ensures `just check` at the root validates everything, including the plugin.
