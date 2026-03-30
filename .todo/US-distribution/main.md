---
status: pending
---

# US: App Distribution

## Theme

End-to-end distribution pipeline for Cortado: from CI checks to release artifacts to user installation to in-app update awareness. Apple Silicon macOS only for now.

## Context

- **Current state:** No CI, no CD, no release process. Version is 0.2.0 in Cargo.toml, 0.0.0 in tauri.conf.json (mismatch). No GitHub Actions workflows exist. Bundle config is active with `targets: "all"`.
- **Target:** Users can install Cortado via a one-liner (`curl | bash`), the app knows when a new version is available (via a built-in feed), and Tauri's updater plugin handles the actual update.
- **Tauri updater:** Tauri has a built-in `tauri-plugin-updater` that checks a `latest.json` endpoint (e.g., on GitHub Releases), verifies cryptographic signatures, downloads the `.app.tar.gz` bundle, and replaces the running app. It requires a signing keypair generated via `tauri signer generate`.

## Sequencing

```
01-ci              CI pipeline (lint, test, build verification)
    |
02-semver          Semantic versioning + release workflow + CHANGELOG
    |
03-cd              CD pipeline (build, sign, release to GitHub)
    |
04-notarization    Apple code signing + notarization
    |
05-install-script  curl | bash install script
    |
06-update-feed     Built-in update awareness feed (Tauri updater + activity feed)

07-dev-isolation   Dev / release build isolation (can start anytime, no deps)

08-remove-nix      Remove Nix flake (can start anytime, no deps)
```

Tasks 01–06 are sequential. Tasks 07 and 08 are independent — can start anytime.

## Tasks

| # | Task | Description |
|---|------|-------------|
| 01 | CI pipeline | GitHub Actions: lint (tsc + clippy), test (cargo test), build verification on push/PR. macOS ARM runner. Document in `specs/ci.md`. |
| 02 | Semantic versioning | Single source of truth for version. Align Cargo.toml and tauri.conf.json. `just release` command. `CONTRIBUTING.md` + `CHANGELOG.md`. |
| 03 | CD pipeline | GitHub Actions: on `v*` tag, build signed + notarized release artifacts, create GitHub Release with `.app.tar.gz` + `latest.json`. Document in `specs/cd.md`. |
| 04 | Notarization | Apple Developer account setup, code signing identity, notarization in CI. Gatekeeper-clean installs. |
| 05 | Install script | `curl -fsSL <raw-github-url>/install \| bash` — detect arch, download from GitHub Releases, install to `~/Applications/`. |
| 06 | Update awareness feed | Built-in Cortado feed using Tauri updater plugin. Checks `latest.json`, surfaces "v0.X.0 available" as an activity. User clicks to install. Always visible in tray and panel. |
| 07 | Dev / release isolation | Separate bundle ID, config dir, and app name for dev builds so `just dev` doesn't collide with an installed release. |
| 08 | Remove Nix | Remove `flake.nix`, `flake.lock`, `.envrc`, `.direnv/` — not used, simplifies repo and CI. |

## Cross-cutting notes

### Signing keys
- The Tauri updater requires a signing keypair. The private key must be stored as a GitHub Actions secret (`TAURI_SIGNING_PRIVATE_KEY`). The public key goes into `tauri.conf.json`.
- Generate once with `pnpm exec tauri signer generate -w ~/.tauri/cortado.key`.
- **Never commit the private key.** If lost, existing users cannot receive updates.

### Bundle ID + Version mismatch
- The current bundle ID is `com.cortado.app` — must be changed to `sh.oribi.cortado` as part of this story. Dev builds use `sh.oribi.cortado.dev`.
- Cargo.toml says `0.2.0`, tauri.conf.json says `0.0.0`. These must be aligned as part of task 02.

### Tauri updater plugin
- Plugin: `tauri-plugin-updater` (already in the Tauri ecosystem, not yet added to this project).
- Endpoint: static `latest.json` on GitHub Releases (simplest approach, no server needed).
- The updater checks `latest.json`, compares versions, downloads + verifies signature, installs, and optionally relaunches.
- Formal docs: https://v2.tauri.app/plugin/updater/

### Update feed UX
- The update feed is a standard feed implementing the existing `Feed` trait — no new event feed architecture needed.
- Activities use `AttentionPositive` StatusKind and participate in normal rollup.
- No dismiss option — the update activity stays visible until installed.
- Expanding the activity shows release notes; an action triggers the update.

### Install script considerations
- Pattern: `curl -fsSL <url> | bash` (same as opencode.ai).
- Repo: `oribarilan/cortado` on GitHub.
- Script detects OS (macOS only for now) and arch (aarch64 only for now, but future-proof for x86_64).
- Downloads `.app.tar.gz` from GitHub Releases, extracts to `~/Applications/Cortado.app`.
- First-time install only — subsequent updates handled by the Tauri updater plugin in-app.
- The updater replaces the `.app` in-place wherever it's running from (user may move it to `/Applications/`).

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
| Install location | `~/Applications/Cortado.app` (user-local, no sudo) |
| Install script hosting | Raw GitHub URL for now (`raw.githubusercontent.com/...`) |
| Homebrew tap | Skip for now |
| Update feed behavior | Enabled by default, notify only. User clicks to install. Update activity always visible in tray and panel. |
| Gatekeeper / notarization | Include notarization in this story (requires Apple Developer account). |
| Version bump workflow | Automated: `just release patch/minor/major` bumps both files, commits, tags, pushes. Documented in `CONTRIBUTING.md`, referenced from `AGENTS.md`. |
| Release notes source | `CHANGELOG.md`, maintained incrementally. Extracted by CD workflow for GitHub Release description and update feed body. |
