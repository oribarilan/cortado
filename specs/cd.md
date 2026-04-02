# CD Pipeline

## Workflow

The CD pipeline runs when a version tag (`v*`) is pushed to the repository.

**File:** `.github/workflows/cd.yml`

## What it does

1. **Check** — runs `just check` (same as CI, ensures the tagged commit is clean)
2. **Sign** — imports the Apple Developer ID certificate into a temporary keychain
3. **Notarize** — writes the App Store Connect API key for Apple notarization
4. **Build** — runs `just build` with signing and notarization env vars set. Tauri handles code signing and notarization automatically during the build.
5. **Release notes** — extracts the relevant section from `CHANGELOG.md`
6. **Publish** — creates a GitHub Release with the signed, notarized DMG attached

## Trigger

Push a tag matching `v*` (e.g., `v0.3.0`). See `CONTRIBUTING.md` for the full release process.

## Artifacts

The release includes:

- `Cortado_X.Y.Z_aarch64.dmg` — signed, notarized macOS Apple Silicon disk image
- `Cortado.app.tar.gz` — compressed app bundle for Tauri updater
- `Cortado.app.tar.gz.sig` — Ed25519 signature for updater verification
- `latest.json` — Tauri updater endpoint (version, platform URL, signature, release notes)

## Required GitHub Actions secrets

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` export |
| `APPLE_SIGNING_IDENTITY` | e.g., `Developer ID Application: Name (TEAMID)` |
| `APPLE_API_ISSUER` | App Store Connect API issuer ID |
| `APPLE_API_KEY` | App Store Connect API key ID |
| `APPLE_API_KEY_PATH` | Base64-encoded `.p8` private key file |
| `KEYCHAIN_PASSWORD` | Any strong password for the CI-only temporary keychain |
| `TAURI_SIGNING_PRIVATE_KEY` | Ed25519 private key for Tauri updater artifact signing |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the Tauri signing key (optional) |

## Runner

macOS ARM (`macos-latest`) — same as CI.

## Future additions (US-distribution-part2)

- ~~Tauri updater artifacts (`.app.tar.gz`, `.app.tar.gz.sig`, `latest.json`)~~ Done — updater artifacts are now produced and uploaded alongside the DMG.
