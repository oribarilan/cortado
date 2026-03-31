# Contributing to Cortado

## Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org) (LTS)
- [pnpm](https://pnpm.io/) (`npm install -g pnpm`)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (Xcode Command Line Tools on macOS)
- [just](https://github.com/casey/just) command runner

## Getting started

```bash
git clone https://github.com/oribarilan/cortado.git
cd cortado
just install   # install JS dependencies
just dev       # run the app in dev mode
```

## Commands

| Command | What it does |
|---------|-------------|
| `just` | List all commands |
| `just install` | Install JS dependencies (pnpm) |
| `just dev` | Run the app locally (dev identity) |
| `just build` | Build a release DMG (production identity) |
| `just check` | Format + lint + test (run before committing) |
| `just lint` | TypeScript type check + Cargo clippy |
| `just format` | Cargo fmt |
| `just test` | Cargo test |

## Dev vs. production builds

The app uses two identities to avoid collisions:

| | Dev | Production |
|---|---|---|
| **Bundle ID** | `sh.oribi.cortado.dev` | `sh.oribi.cortado` |
| **App name** | Cortado Dev | Cortado |
| **Config dir** | `~/.config/cortado-dev/` | `~/.config/cortado/` |
| **Global hotkey** | Not registered | Registered |

- `just dev` uses the dev identity via `tauri.dev.conf.json` config overlay.
- `just build` produces a **production** DMG (no overlay).
- The `tauri-plugin-single-instance` plugin prevents running two instances with the same bundle ID.
- A red "DEV" badge is shown in the main screen panel for dev builds.

You can run a dev instance and a production install side-by-side. You cannot run two dev instances simultaneously.

## Building a DMG locally

```bash
just build
```

The DMG is output to `src-tauri/target/release/bundle/dmg/Cortado_<version>_aarch64.dmg`.

The build is unsigned and unnotarized — macOS will show a Gatekeeper warning.
To open it: right-click the app > Open, or run `xattr -cr` on the `.app` bundle.

## Version

The version lives in two files and must be kept in sync:

- `src-tauri/Cargo.toml` — `version` under `[package]`
- `src-tauri/tauri.conf.json` — `version` at the top level

### Releasing

To release a new version:

1. Bump version in both `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`
2. Move `## Unreleased` entries in `CHANGELOG.md` to a new `## [X.Y.Z] - YYYY-MM-DD` section
3. Commit: `release: vX.Y.Z`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push && git push origin vX.Y.Z`

The tag push triggers the CD workflow to build and publish the release.

Version levels: MAJOR (breaking), MINOR (features), PATCH (fixes). Pre-1.0, MINOR may include breaking changes.

### Changelog

Maintain `CHANGELOG.md` incrementally using [Keep a Changelog](https://keepachangelog.com/) format. Add entries under `## Unreleased` as you work. The release script promotes them to a versioned section.

## Code quality

Always run `just check` before committing. It must pass with zero warnings:

- `cargo fmt` — formatting
- `tsc --noEmit` — TypeScript type checking
- `cargo clippy -D warnings` — all Clippy warnings are errors
- `cargo test` — all tests must pass

## Package manager

Use **pnpm**, not npm or yarn. The Tauri CLI is a local devDependency — run it via `pnpm exec tauri`, not `pnpm tauri`.

## Commit style

- Summarize the "why" in 1-2 sentences.
- Use conventional-ish prefixes when natural: `add`, `fix`, `update`, `remove`, `refactor`.
