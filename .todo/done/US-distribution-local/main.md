---
status: done
---

# US: Local Distribution (Build & Install)

## Theme

Get Cortado building into a distributable DMG locally, installable and runnable side-by-side with dev builds. Validate the artifact end-to-end on the developer's machine before automating in CI/CD.

## Context

- **Current state:** No build command exists. Bundle config uses `targets: "all"` (builds every format). Version mismatch: Cargo.toml says `0.2.0`, tauri.conf.json says `0.0.0`. Bundle ID is `com.cortado.app` (should be `sh.oribi.cortado`).
- **Target:** `just build` produces a DMG. The developer can install it, launch it, and run it alongside `just dev` without collisions.

## Sequencing

```
01-bundle-config   Fix bundle ID, version, targets
    |
02-build-command   Add `just build`, verify DMG output
    |
03-dev-isolation   Dev / release build isolation
```

All three are sequential.

## Tasks

| # | Task | Description |
|---|------|-------------|
| 01 | Bundle config | Align version across Cargo.toml and tauri.conf.json. Change bundle ID to `sh.oribi.cortado`. Set bundle targets to `["dmg", "app"]`. |
| 02 | Build command | Add `just build` recipe that runs `pnpm exec tauri build`. Verify it produces a working DMG in `src-tauri/target/release/bundle/dmg/`. |
| 03 | Dev isolation | Separate bundle ID, config dir, and app name for dev builds so `just dev` and an installed release don't collide. |
