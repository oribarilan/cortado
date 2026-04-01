---
status: pending
---

# Configurable config file location / XDG support

## Goal

Allow users to control where Cortado looks for its config file, respecting
the XDG Base Directory Specification on systems that support it.

## Context

Currently the config file location is determined by Tauri's default path
resolver. On macOS this is typically `~/Library/Application Support/`. Some
users prefer XDG-style paths (`$XDG_CONFIG_HOME/cortado/`) or want to
symlink/customize the location (e.g., to keep dotfiles in a git repo).

## Acceptance criteria

- [ ] Respect `$XDG_CONFIG_HOME` when set (fall back to `~/.config/` on
      Linux, keep macOS default when unset)
- [ ] Support an explicit env var (e.g., `CORTADO_CONFIG_DIR`) that overrides
      everything
- [ ] Document the resolution order in the spec / README
- [ ] Existing configs in the default location continue to work without
      migration

## Notes

- On macOS, `~/Library/Application Support/` is the platform convention.
  XDG is mostly relevant for Linux, but some macOS users set
  `$XDG_CONFIG_HOME` too.
- Consider whether data/cache dirs should also be configurable or just config.
