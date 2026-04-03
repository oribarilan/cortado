---
status: pending
---

# Shell feed type

## Context

The shell feed type was removed from the active codebase to reduce surface area. It allowed users to run arbitrary shell commands and track their output as a single typed field.

## What it did

- Ran a user-specified command via `sh -c`
- Captured stdout as a single field (text, status, number, or url)
- Config keys: `command`, `field_name`, `field_type`
- Default interval: 30s

## Why removed

- Low priority relative to other feed types
- Broad attack surface (arbitrary command execution)
- Can be revisited when there's clear demand

## To restore

- Re-add `shell.rs` in `src-tauri/src/feed/`
- Wire into `mod.rs` instantiate_feed, config parsing, and settings UI
- Historical implementation is in git history and `.todo/done/sprint01/04-shell-stub.md`, `.todo/done/sprint02/01-shell-execution.md`
