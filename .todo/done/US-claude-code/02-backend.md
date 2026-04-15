---
status: pending
---

# Backend wiring

## Context

Wire the Claude Code feed type into the backend: add the match arm in `instantiate_harness_feed()`, create Tauri commands for plugin management, and embed the plugin files in the binary.

**Value delivered**: The backend can instantiate `claude-code-session` feeds and the Settings UI can install/uninstall the plugin.

## Related Files

- `src-tauri/src/feed/mod.rs` -- `instantiate_harness_feed()` match arms
- `src-tauri/src/settings_config.rs` -- plugin management commands (see copilot/opencode patterns)
- `src-tauri/src/main.rs` -- Tauri command registration

## Dependencies

- `01-plugin.md` (plugin files must exist to embed them)

## Acceptance Criteria

- [ ] `instantiate_harness_feed()` in `feed/mod.rs` has a `"claude-code-session"` match arm using `GenericProvider::new("claude-code")`
- [ ] `settings_config.rs` has three new Tauri commands:
  - `check_claude_code_plugin` -- checks if plugin is installed, returns `SetupCheckResult`
  - `install_claude_code_plugin` -- writes plugin files and registers in Claude Code settings
  - `uninstall_claude_code_plugin` -- removes the plugin and deregisters from settings
- [ ] Plugin files are embedded via `include_str!()` (hook script, plugin.json, hooks.json)
- [ ] Marketplace JSON is embedded or generated at install time
- [ ] Commands are registered in `main.rs` builder
- [ ] Plugin installation writes the correct directory structure for Claude Code plugins (`.claude-plugin/plugin.json`, `hooks/hooks.json`, `scripts/cortado-hook.sh`)
- [ ] `just check` passes (clippy, tsc, tests)

## Verification

- **Automated**: `just check` passes cleanly
- **Ad-hoc**: Verify the match arm compiles, verify the embedded strings are non-empty, verify command registration in main.rs

## Notes

### Plugin installation strategy

**`claude plugin install --dir <path>` does NOT exist.** The `claude plugin install` CLI only installs from marketplaces. Raw paths in `enabledPlugins` are not recognized (tested).

Use a **local marketplace**. `claude plugin marketplace add <path>` accepts a local directory, and plugins inside it can use relative paths.

#### Directory structure

Cortado writes the marketplace + plugin to a persistent directory:

```
~/.config/cortado/marketplace/
  .claude-plugin/
    marketplace.json       # marketplace catalog
  plugins/
    cortado/               # the actual plugin
      .claude-plugin/plugin.json
      hooks/hooks.json
      scripts/cortado-hook.sh
```

#### marketplace.json

```json
{
  "name": "cortado",
  "owner": { "name": "Cortado" },
  "plugins": [
    {
      "name": "cortado",
      "source": "./plugins/cortado",
      "description": "Cortado session tracking for Claude Code"
    }
  ]
}
```

#### Install flow

1. Write all embedded files to `~/.config/cortado/marketplace/` with the structure above
2. Run `claude plugin marketplace add ~/.config/cortado/marketplace/`
3. Run `claude plugin install cortado@cortado`

#### Check flow

Run `claude plugin list` and look for `cortado@cortado`. Or check that `~/.config/cortado/marketplace/plugins/cortado/` exists AND the marketplace is registered.

#### Uninstall flow

1. Run `claude plugin uninstall cortado@cortado`
2. Run `claude plugin marketplace remove cortado`
3. Remove `~/.config/cortado/marketplace/`

#### Update flow (for task 04)

On Cortado startup, overwrite the marketplace source files with the latest embedded versions. If the installed plugin's `# cortado-plugin-version` header is older than the embedded one, show an update prompt. The user runs `claude plugin update cortado@cortado` to pick up the new version.

### Embedded files

Follow the Copilot pattern in `settings_config.rs`:
```rust
pub(crate) const COPILOT_PLUGIN_JSON: &str = include_str!("../../plugins/copilot/plugin.json");
pub(crate) const COPILOT_HOOKS_JSON: &str = include_str!("../../plugins/copilot/hooks.json");
pub(crate) const COPILOT_HOOK_SCRIPT: &str = include_str!("../../plugins/copilot/cortado-hook.sh");
```

Do the same for all Claude Code plugin files. Note the different directory structure:
```rust
include_str!("../../plugins/claude-code/.claude-plugin/plugin.json")
include_str!("../../plugins/claude-code/hooks/hooks.json")
include_str!("../../plugins/claude-code/scripts/cortado-hook.sh")
```
