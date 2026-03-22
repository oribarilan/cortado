---
status: pending
---

# Optional: error handling standards in AGENTS.md

## Goal

Define error handling conventions and add them to AGENTS.md so agents produce consistent error handling across the codebase.

## Notes

- Rust: when to use `Result` vs `panic!`, custom error types vs `anyhow`, error propagation with `?`.
- Tauri boundary: how errors cross from Rust commands to the TypeScript frontend.
- TypeScript: try/catch patterns, when to surface errors to the user vs log silently.
- User-facing error messages: should they be human-readable? Localized? Include context?
- Decide on a crate for error handling (e.g., `thiserror` for typed errors, `anyhow` for ad-hoc) if not already chosen.
