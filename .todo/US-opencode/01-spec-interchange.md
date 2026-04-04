---
status: done
---

# Define generic harness interchange format

## Goal

Create a spec for the harness interchange format -- the contract between coding agent plugins (like the OpenCode plugin) and Cortado's generic harness provider. This format should be agent-agnostic so any future coding tool can adopt it.

## Acceptance criteria

- [ ] `specs/harness-interchange.md` exists with a complete spec
- [ ] JSON schema is defined with all fields, types, and constraints
- [ ] File location (`~/.config/cortado/harness/`), naming, and lifecycle rules are documented
- [ ] Status values are defined and mapped to existing `SessionStatus` enum
- [ ] Atomic write requirement is documented (write to temp file, then `rename`)
- [ ] PID liveness semantics are documented (including PID recycling as a known limitation)
- [ ] Stale file cleanup behavior is specified
- [ ] Versioning strategy is defined: `version` field is required, provider skips files with unrecognized version (logs warning)

## Notes

Key design points:

- **Location:** `~/.config/cortado/harness/<session-id>.json` (consistent with Cortado's config directory)
- **Format:** Flat JSON object, one file per active session
- **Status values:** `working`, `question`, `approval`, `idle` (maps to `SessionStatus`). OpenCode only uses `working` and `idle`; `question` and `approval` exist for future agents.
- **Required fields:** `version`, `harness`, `id`, `pid`, `cwd`, `status`, `last_active_at`
- **Optional fields:** `repository`, `branch`, `summary`
- **Atomic writes:** All file writes must use write-to-temp-then-rename to prevent partial reads
- **Version handling:** Provider must skip files with unrecognized `version` (log warning, don't error). Current version: `1`.
- **PID recycling:** Acknowledged limitation -- `kill(pid, 0)` can return true for a recycled PID. Same as Copilot provider. Accepted risk.
- **Lifecycle:** Plugin creates on session start, updates on events, deletes on session end. Cortado checks PID and cleans stale files.
- **Filename:** `<pid>.json` -- using the process PID as the filename. Simple, always available, unique per running process. The agent's internal session ID can be stored as a field inside the JSON (`id`), but the filename is always PID-based. Avoids rename dances when the session ID isn't immediately available.
- **Harness directory is always production path:** `~/.config/cortado/harness/` regardless of whether Cortado is running in dev mode. The harness directory is a cross-app contract -- agent plugins don't know about Cortado's dev/prod distinction. Both dev and production Cortado read from the same path.

The spec should be clear enough that someone could write a plugin for a new agent without reading Cortado source code.
