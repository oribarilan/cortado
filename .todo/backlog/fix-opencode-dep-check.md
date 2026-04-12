---
status: done
---

# False "OpenCode not found" warning in settings (packaged app)

## Problem

The settings UI shows "OpenCode not found. This feed requires opencode to be installed. Install -->" when viewing or editing an opencode feed, even though opencode is installed and the feed works correctly. Affects the packaged app, not dev builds.

## Root cause

The app's startup PATH fix (`main.rs:37`) runs `$SHELL -l -c 'printf "%s" "$PATH"'`. This is a **login, non-interactive** shell, which sources `~/.zprofile` but **skips `~/.zshrc`** (only sourced for interactive shells).

OpenCode's installer adds `export PATH=$HOME/.opencode/bin:$PATH` to `~/.zshrc`, not `~/.zprofile`. So in packaged builds:

1. App starts with minimal launchd PATH (`/usr/bin:/bin:/usr/sbin:/sbin`)
2. PATH fix runs login non-interactive shell -- `~/.zshrc` skipped
3. `~/.opencode/bin` never added to app's PATH
4. `which opencode` fails in `check_feed_dependency`
5. Settings shows "not found"

This is a **broader issue** -- any tool that adds its PATH entry to `.zshrc` instead of `.zprofile` would have the same problem in packaged builds.

The feed itself works because it uses filesystem watching (`HarnessFeed`), not the `opencode` CLI.

## Verified fix

Explicitly source `~/.zshrc` in the PATH resolution command:

```
$SHELL -l -c '[ -f "$HOME/.zshrc" ] && . "$HOME/.zshrc" 2>/dev/null; printf "%s" "$PATH"'
```

Tested: starting with minimal PATH, this successfully picks up `~/.opencode/bin`. Output is clean (no stdout contamination from `.zshrc`).

For shell-agnosticism, detect the shell and source the appropriate RC file:
- zsh: `~/.zshrc`
- bash: `~/.bashrc`

## Implementation

In `src-tauri/src/main.rs:37-39`, change the `-l -c` command to also source the shell's interactive RC file. Add shell detection:

```rust
let rc_source = if shell.contains("zsh") {
    r#"[ -f "$HOME/.zshrc" ] && . "$HOME/.zshrc" 2>/dev/null; "#
} else if shell.contains("bash") {
    r#"[ -f "$HOME/.bashrc" ] && . "$HOME/.bashrc" 2>/dev/null; "#
} else {
    ""
};
let cmd = format!(r#"{rc_source}printf '%s' "$PATH""#);
```

Then use `cmd` as the `-c` argument instead of the current hardcoded string.

## Relevant files

- `src-tauri/src/main.rs:32-47` -- PATH fix at startup (needs change)
- `src-tauri/src/settings_config.rs:313-321` -- `check_feed_dependency` (unchanged, works correctly)
- `src/shared/feedTypes.ts:259-263` -- dependency definition (unchanged)
