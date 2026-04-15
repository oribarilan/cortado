---
status: pending
---

# Plugin update detection

## Context

Wire Claude Code plugin version checking into the `cortado-update` feed so users see an update prompt when their installed plugin is outdated.

**Value delivered**: Users are notified when their Claude Code plugin needs updating, matching the existing Copilot and OpenCode update detection.

## Related Files

- `src-tauri/src/feed/cortado_update.rs` -- `CortadoUpdateFeed` with existing copilot/opencode update checks
- `src-tauri/src/settings_config.rs` -- `is_plugin_outdated()`, `parse_plugin_version()`

## Dependencies

- `02-backend.md` (plugin embedding and directory resolution functions)

## Acceptance Criteria

- [ ] `CortadoUpdateFeed::new()` accepts a `check_claude_code_plugin: bool` parameter
- [ ] A `check_claude_code_plugin_update()` method exists, following the pattern of `check_copilot_extension_update()`
- [ ] The method finds the installed plugin's hook script, compares its version header against the embedded version using `is_plugin_outdated()`
- [ ] When outdated, it returns an `Activity` prompting the user to update
- [ ] All call sites of `CortadoUpdateFeed::new()` are updated to pass the new parameter
- [ ] `just check` passes

## Verification

- **Automated**: `just check` passes cleanly
- **Ad-hoc**: Verify the update check method follows the same pattern as `check_copilot_extension_update()`

## Notes

The plugin source lives at `~/.config/cortado/marketplace/plugins/cortado/scripts/cortado-hook.sh` (written by Cortado). On Cortado startup, overwrite these source files with the latest embedded versions. To check if the *installed* plugin is outdated, compare its `# cortado-plugin-version` header against the embedded version. The installed copy lives in the Claude Code cache (`~/.claude/plugins/cache/cortado/cortado/*/scripts/cortado-hook.sh` -- glob for the version directory). When outdated, prompt the user to run `claude plugin update cortado@cortado`.
