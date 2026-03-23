---
status: done
---

# Duration-string config parsing with jiff

## Goal

Migrate feed timing config parsing from integer seconds to duration strings parsed with `jiff`.

## Acceptance criteria

- [x] `interval` accepts duration strings (e.g., `"30s"`, `"5m"`, `"1.5m"`).
- [x] Integer `interval` values are rejected with clear config validation errors.
- [x] Invalid/negative/zero duration values produce actionable errors.
- [x] Feed defaults remain equivalent (`github-pr` 120s, `shell` 30s) when interval omitted.
- [x] Parsing implementation uses `jiff` (not custom parser).
- [x] Config parser tests cover valid and invalid duration-string cases.
- [x] `just check` passes.

## Notes

- No backward compatibility required for integer intervals.

## Relevant files

- `src-tauri/src/feed/config.rs`
- `src-tauri/Cargo.toml`
