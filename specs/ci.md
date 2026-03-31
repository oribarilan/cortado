# CI Pipeline

## Workflow

The CI pipeline runs on every push to `main` and on every pull request.

**File:** `.github/workflows/ci.yml`

## What it does

1. **Check** — runs `just check` (format + lint + test):
   - `cargo fmt` — formatting verification
   - `tsc --noEmit` — TypeScript type checking
   - `cargo clippy -D warnings` — all Clippy warnings are errors
   - `cargo test` — all Rust tests

2. **Build** — runs `just build` (release compilation + DMG bundling):
   - Verifies the app compiles in release mode
   - Verifies the DMG bundles correctly

## Runner

macOS ARM (`macos-latest`) — required for the Tauri macOS build and native dependencies.

## Concurrency

Pushes to the same branch cancel in-progress runs to avoid wasted CI minutes.

## Toolchain

- Rust stable (via `dtolnay/rust-toolchain`)
- Node.js LTS (via `actions/setup-node`)
- pnpm (version from `packageManager` in `package.json`, via `pnpm/action-setup`)
- just (installed via Homebrew on macOS runner)
