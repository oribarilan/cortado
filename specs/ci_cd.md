# CI/CD Pipeline

**File:** `.github/workflows/ci.yml`

## Jobs

### Check (always runs)

Runs on every push to `main`, every pull request, and every version tag (`v*`).

1. `just check` — format, lint (clippy + tsc), tests
2. `pnpm build` — frontend Vite bundle

### Release (only on `v*` tags, after Check passes)

1. **Sign** — imports the Apple Developer ID certificate into a temporary keychain
2. **Notarize** — writes the App Store Connect API key for Apple notarization
3. **Build** — runs `just build` with signing and notarization env vars set. Tauri handles code signing and notarization automatically during the build.
4. **Release notes** — extracts the relevant section from `CHANGELOG.md`
5. **Updater** — generates `latest.json` with version, platform URL, signature, and release notes
6. **Publish** — creates a GitHub Release with all artifacts

### CI Packaging (`ci-build.yml`)

Runs only when packaging-related files change (Tauri config, Cargo deps, capabilities, icons, JS deps). Triggered on pushes to `main` and pull requests.

- Runs `tauri build` without updater signing
- Verifies the `.app` and DMG bundle correctly
- Skips updater artifact signing (no secrets needed)

Monitored paths: `src-tauri/tauri.conf.json`, `src-tauri/tauri.dev.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/capabilities/**`, `src-tauri/icons/**`, `package.json`, `pnpm-lock.yaml`.

## Trigger

Push a tag matching `v*` (e.g., `v0.5.0`). See `CONTRIBUTING.md` for the full release process.

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

macOS ARM (`macos-latest`) — required for the Tauri macOS build and native dependencies.

## Concurrency

Pushes to the same branch cancel in-progress runs to avoid wasted CI minutes.

## Toolchain

- Rust stable (via `dtolnay/rust-toolchain`)
- Node.js LTS (via `actions/setup-node`)
- pnpm (version from `packageManager` in `package.json`, via `pnpm/action-setup`)
- just (installed via Homebrew on macOS runner)
