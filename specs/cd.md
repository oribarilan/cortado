# CD Pipeline

## Workflow

The CD pipeline runs when a version tag (`v*`) is pushed to the repository.

**File:** `.github/workflows/cd.yml`

## What it does

1. **Check** — runs `just check` (same as CI, ensures the tagged commit is clean)
2. **Build** — runs `just build` (release DMG for macOS aarch64)
3. **Release notes** — extracts the relevant section from `CHANGELOG.md`
4. **Publish** — creates a GitHub Release with the DMG attached

## Trigger

Push a tag matching `v*` (e.g., `v0.3.0`). See `CONTRIBUTING.md` for the full release process.

## Artifacts

The release includes:

- `Cortado_X.Y.Z_aarch64.dmg` — macOS Apple Silicon disk image

## Runner

macOS ARM (`macos-latest`) — same as CI.

## Future additions (US-distribution-part2)

- Code signing + notarization (Gatekeeper-clean installs)
- Tauri updater artifacts (`.app.tar.gz`, `.app.tar.gz.sig`, `latest.json`)
