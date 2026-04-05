---
status: done
---

# Replace CopilotProvider with GenericProvider and remove copilot.rs

## Goal

Switch the `copilot-session` feed type from the native `CopilotProvider` to `GenericProvider::new("copilot")`, then remove `copilot.rs` and its dependencies. This completes the migration to the interchange-based architecture.

## Acceptance criteria

- [ ] `instantiate_harness_feed()` in `feed/mod.rs`: `"copilot-session"` now creates `GenericProvider::new("copilot")` instead of `CopilotProvider::new()`
- [ ] `src-tauri/src/feed/harness/copilot.rs` is deleted
- [ ] `mod copilot;` and `use ... copilot::CopilotProvider` removed from `harness/mod.rs`
- [ ] All references to `CopilotProvider` removed from the codebase
- [ ] `serde-saphyr` removed from `src-tauri/Cargo.toml` (confirmed only used by copilot.rs; note: hyphen in Cargo.toml, underscore in Rust source)
- [ ] `just check` passes (format + lint + test)
- [ ] No warnings from `cargo clippy`

## Notes

### GenericProvider filter

`GenericProvider::new("copilot")` filters interchange files by `harness == "copilot"`. The Copilot extension (task 01) writes `"harness": "copilot"` in its interchange JSON. The filter matches.

### What we lose

The native `CopilotProvider` could discover sessions even without the Cortado extension installed (by reading Copilot's native files). After this change, the feed produces no data unless the extension is installed. This is intentional -- the setup flow (task 03) guides users to install the extension, and the dependency check verifies the CLI is present.

### What we gain

- ~750 lines of Rust removed (copilot.rs)
- No more YAML parsing (serde_saphyr), JSONL tail-reading, lock file scanning
- Status tracking is more accurate (real-time events vs. inferring from last 2 JSONL lines)
- `question` status now comes from actual `ask_user` tool events, not heuristic inference
- Consistent architecture: both agent feeds use the same `GenericProvider` + interchange format

### serde-saphyr removal

Confirmed only used in `copilot.rs` (5 hits). Remove from `Cargo.toml` (the crate name uses a hyphen: `serde-saphyr`). Run `cargo build` after to verify no other transitive consumers.
