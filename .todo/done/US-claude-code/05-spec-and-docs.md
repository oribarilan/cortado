---
status: pending
---

# Spec and docs updates

## Context

Update specs and documentation to reflect the new Claude Code feed type.

**Value delivered**: Specs and docs stay in sync with the implementation, as required by AGENTS.md.

## Related Files

- `specs/main.md` -- feed types table, config examples
- `README.md` -- feed types table, config reference
- `CHANGELOG.md` -- user-facing changelog entry

## Dependencies

- `01-plugin.md`, `02-backend.md`, `03-frontend.md` (need to know final implementation details)

## Acceptance Criteria

- [ ] `specs/main.md` lists `claude-code-session` in the feed types table with default interval `30s`
- [ ] `specs/main.md` includes a config example:
  ```toml
  [[feed]]
  name = "Claude Code"
  type = "claude-code-session"
  ```
- [ ] `README.md` feed types table includes Claude Code
- [ ] `CHANGELOG.md` has a brief user-facing entry (e.g., "Claude Code: track active coding sessions with status, repo, and branch info")
- [ ] No references to internal implementation details in user-facing docs

## Verification

- **Ad-hoc**: Read updated files and verify Claude Code is listed alongside Copilot and OpenCode in all relevant tables and examples
