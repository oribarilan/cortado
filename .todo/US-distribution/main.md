---
status: pending
---

# US: App Distribution (Part 1 — DMG Release)

## Theme

CI/CD pipeline for Cortado: from code quality checks to signed, notarized DMG artifacts on GitHub Releases. Apple Silicon macOS only for now.

Auto-update (Tauri updater, install script, update feed) is deferred to US-distribution-part2.

## Context

- **Current state:** Bundle config fixed (`sh.oribi.cortado`, version `0.2.0`), `just build` produces DMG, dev isolation set up. No CI, no CD, no release process. No GitHub Actions workflows exist.
- **Target:** Users can download a signed, notarized DMG from GitHub Releases, open it, and drag Cortado to Applications. No Gatekeeper warnings.

## Sequencing

```
01-ci              CI pipeline (lint, test, build verification)
    |
02-semver          Semantic versioning + release workflow + CHANGELOG
    |
03-cd              CD pipeline (build DMG, release to GitHub)
    |
04-notarization    Apple code signing + notarization

05-remove-nix      Remove Nix flake (can start anytime, no deps)
```

Tasks 01–04 are sequential. Task 05 is independent — can start anytime.

## Prerequisites

- US-distribution-local must be complete: bundle config fixed, `just build` works, dev isolation set up.

## Tasks

| # | Task | Description |
|---|------|-------------|
| 01 | CI pipeline | GitHub Actions: lint (tsc + clippy), test (cargo test), build verification on push/PR. macOS ARM runner. Document in `specs/ci.md`. |
| 02 | Semantic versioning | Single source of truth for version. `just release` command. `CONTRIBUTING.md` + `CHANGELOG.md`. |
| 03 | CD pipeline | GitHub Actions: on `v*` tag, build DMG, create GitHub Release with DMG artifact. Document in `specs/cd.md`. |
| 04 | Notarization | Apple Developer account setup, code signing identity, notarization in CI. Gatekeeper-clean installs. |
| 05 | Remove Nix | Remove `flake.nix`, `flake.lock`, `.envrc`, `.direnv/` — not used, simplifies repo and CI. |

## Cross-cutting notes

### Bundle ID + Version
- Already aligned in US-distribution-local: `sh.oribi.cortado`, version `0.2.0`. Dev builds use `sh.oribi.cortado.dev`.

### Nix removal
- `flake.nix`, `flake.lock`, `.envrc`, `.direnv/` are not used. Remove them to simplify the repo.
- CI uses direct Rust + Node setup, not Nix.
- Prerequisites (Rust, Node, pnpm) should be documented in `CONTRIBUTING.md` instead.

### Documentation
- CI process documented in `specs/ci.md`.
- CD process documented in `specs/cd.md`.

## Open questions (resolved)

| Question | Decision |
|----------|----------|
| Install location | Users download DMG from GitHub Releases and install manually for now |
| Homebrew tap | Skip for now |
| Gatekeeper / notarization | Include notarization in this story (requires Apple Developer account) |
| Version bump workflow | Automated: `just release patch/minor/major` bumps both files, commits, tags, pushes. Documented in `CONTRIBUTING.md`, referenced from `AGENTS.md`. |
| Release notes source | `CHANGELOG.md`, maintained incrementally. Extracted by CD workflow for GitHub Release description. |
| Install script / auto-update | Deferred to US-distribution-part2 |
