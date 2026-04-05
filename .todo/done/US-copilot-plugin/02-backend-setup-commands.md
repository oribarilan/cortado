---
status: done
---

# Add check/install Tauri commands and embed extension source

## Goal

Add backend support for checking whether the Cortado extension is installed in Copilot CLI, installing it on demand, and embedding the extension source in the Cortado binary at compile time. Follows the exact same pattern as the OpenCode plugin commands.

## Acceptance criteria

- [ ] `COPILOT_EXTENSION_SOURCE` constant in `settings_config.rs` via `include_str!("../../plugins/copilot/src/extension.mjs")`
- [ ] `COPILOT_EXTENSION_FILENAME` constant: `"extension.mjs"`
- [ ] `copilot_extensions_dir()` function returns `~/.copilot/extensions/cortado/` path
- [ ] `check_copilot_extension` Tauri command:
  - Checks if `~/.copilot/extensions/cortado/extension.mjs` exists on disk
  - If exists, compares version via `// cortado-plugin-version: N` header (reuses `parse_plugin_version()` and `is_plugin_outdated()`)
  - Returns `SetupCheckResult { ready: bool, outdated: bool }`
  - No slow-path fallback needed (unlike OpenCode, there's no `copilot debug config` equivalent to check)
- [ ] `install_copilot_extension` Tauri command:
  - Creates `~/.copilot/extensions/cortado/` directory if needed
  - Writes embedded source to `~/.copilot/extensions/cortado/extension.mjs`
  - Idempotent (overwrites existing file)
  - Returns `SetupInstallResult { success: bool, error: Option<String> }`
- [ ] Both commands registered in the Tauri builder (`main.rs` or wherever commands are registered)
- [ ] `cargo build` succeeds with the embedded extension source (build depends on task 01 — `include_str!` will fail at compile time if the file doesn't exist)
- [ ] Reuses existing `SetupCheckResult`, `SetupInstallResult`, `parse_plugin_version()`, `is_plugin_outdated()` -- no duplication

## Notes

### Differences from OpenCode

The OpenCode `check_opencode_plugin` has a slow-path that runs `opencode debug config` to check if the plugin is configured via npm or the config file. For Copilot, there's no equivalent CLI command to query loaded extensions. The file existence check is sufficient.

### Directory structure

OpenCode plugin path: `~/.config/opencode/plugins/cortado-opencode.ts`
Copilot extension path: `~/.copilot/extensions/cortado/extension.mjs`

Note the extra directory level -- Copilot extensions live in a subdirectory named after the extension, with a fixed `extension.mjs` entry point.

### Existing helpers

`parse_plugin_version()` and `is_plugin_outdated()` in `settings_config.rs` are already generic (they just look for `// cortado-plugin-version:` headers). Reuse them directly.
