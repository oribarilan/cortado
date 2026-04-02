# CI Pipeline

## Workflows

### CI (`ci.yml`)

Runs on every push to `main` and on every pull request.

1. **Check** — runs `just check` (format + lint + test):
   - `cargo fmt` — formatting verification
   - `tsc --noEmit` — TypeScript type checking
   - `cargo clippy -D warnings` — all Clippy warnings are errors
   - `cargo test` — all Rust tests

2. **Frontend build** — runs `pnpm build` (Vite bundle):
   - Verifies the frontend compiles and bundles correctly

### CI Build (`ci-build.yml`)

Runs only when packaging-related files change (Tauri config, Cargo deps, capabilities, icons, JS deps). Triggered on pushes to `main` and pull requests.

1. **Build smoke test** — runs `tauri build` without updater signing:
   - Verifies the app compiles in release mode
   - Verifies the `.app` and DMG bundle correctly
   - Skips updater artifact signing (no secrets needed)

Monitored paths: `src-tauri/tauri.conf.json`, `src-tauri/tauri.dev.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/capabilities/**`, `src-tauri/icons/**`, `package.json`, `pnpm-lock.yaml`.

## Runner

macOS ARM (`macos-latest`) — required for the Tauri macOS build and native dependencies.

## Concurrency

Pushes to the same branch cancel in-progress runs to avoid wasted CI minutes.

## Toolchain

- Rust stable (via `dtolnay/rust-toolchain`)
- Node.js LTS (via `actions/setup-node`)
- pnpm (version from `packageManager` in `package.json`, via `pnpm/action-setup`)
- just (installed via Homebrew on macOS runner — CI only, not needed for CI Build)
