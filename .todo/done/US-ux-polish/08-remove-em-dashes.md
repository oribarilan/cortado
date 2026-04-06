---
status: done
---

# Remove all em dashes from the codebase

## Goal

Replace every em dash (`--`, U+2014) with a plain double dash (`--`). Em dashes render inconsistently across terminals, editors, and fonts. Double dashes are universally safe and readable.

## Scope

~1,900 occurrences across ~207 files (excluding `.agents/skills/` and `node_modules/`). Breakdown by category:

| Category | Files | Examples |
|----------|-------|---------|
| Specs/docs | `specs/*.md`, `AGENTS.md`, `CONTRIBUTING.md` | Prose, table cells, inline explanations |
| Rust comments | `src-tauri/src/**/*.rs` | Doc comments (`///`), inline comments (`//`) |
| UI text | `src-tauri/src/notification/content.rs:24` | `format!("{} -- {}", ...)` -- user-visible notification subtitle |
| Plugin comments | `plugins/copilot/cortado-hook.sh`, `plugins/opencode/src/plugin-bundle.ts` | Inline comments |
| SVG comment | `src-tauri/icons/app-icon.svg` | XML comment |

## Notes

- The UI text occurrence in `notification/content.rs` is user-facing (notification subtitle). Replace with ` -- ` to preserve readability.
- Skip any files under `src-tauri/gen/` (auto-generated).
- This is a mechanical find-and-replace -- no behavioral changes.

## Acceptance criteria

- [ ] Zero occurrences of `--` (U+2014) in the codebase (excluding `src-tauri/gen/`)
- [ ] Notification subtitle still reads naturally with `--`
- [ ] `just check` passes
