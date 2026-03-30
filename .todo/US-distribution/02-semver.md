---
status: pending
---

# Semantic Versioning

## Goal

Establish a single source of truth for the app version, align existing mismatches, and create an automated release command.

## Acceptance criteria

- [ ] Version aligned between `src-tauri/Cargo.toml` (currently `0.2.0`) and `src-tauri/tauri.conf.json` (currently `0.0.0`)
- [ ] `just release <patch|minor|major>` command that:
  1. Bumps version in both `Cargo.toml` and `tauri.conf.json`
  2. Updates `CHANGELOG.md` header (moves "Unreleased" to the new version with date)
  3. Creates a git commit: `release: vX.Y.Z`
  4. Creates a git tag: `vX.Y.Z`
  5. Pushes commit and tag
- [ ] `CHANGELOG.md` created with "Keep a Changelog" format
- [ ] `CONTRIBUTING.md` created with release process documentation
- [ ] `AGENTS.md` updated to reference `CONTRIBUTING.md` for release workflow

## Notes

- Consider a small script (bash or Python) for the version bump logic rather than a complex tool. It needs to edit two files: `Cargo.toml` (`version = "X.Y.Z"`) and `tauri.conf.json` (`"version": "X.Y.Z"`).
- The `CHANGELOG.md` format should follow [Keep a Changelog](https://keepachangelog.com/): sections for Added, Changed, Fixed, Removed under each version.
- Start `CHANGELOG.md` at the current version with a summary of existing functionality, plus an "Unreleased" section for ongoing work.
- Semantic versioning: MAJOR (breaking), MINOR (features), PATCH (fixes). Pre-1.0, MINOR can include breaking changes.

## Relevant files

- `src-tauri/Cargo.toml` — `version = "0.2.0"`
- `src-tauri/tauri.conf.json` — `"version": "0.0.0"`
- `Justfile` — add `release` command
- `CHANGELOG.md` (to create)
- `CONTRIBUTING.md` (to create)
