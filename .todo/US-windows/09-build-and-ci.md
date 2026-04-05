---
status: pending
---

# Task: Build configuration and CI/CD for Windows

## Goal

Configure the build system, Tauri config, and CI/CD pipeline to produce Windows artifacts (NSIS installer) alongside existing macOS artifacts.

## Acceptance criteria

- [ ] `tauri.conf.json` bundle targets updated: `"targets": ["dmg", "app", "nsis"]` — Tauri auto-selects per platform (dmg/app on macOS, nsis on Windows)
- [ ] `tauri.conf.json` has a `"windows"` section with NSIS config (installer name, icon, etc.)
- [ ] Windows `.ico` icon verified for correct sizes (16x16, 32x32, 48x48, 256x256) — already at `icons/icon.ico`
- [ ] `justfile` commands audited for cross-platform compatibility. Shell commands that use Unix-isms (`rm`, pipes, etc.) either work on Windows or have platform-conditional recipes.
- [ ] CI pipeline (`.github/workflows/ci.yml`) includes a Windows build job on `windows-latest` runner producing NSIS installer artifacts
- [ ] CI Windows build job includes code signing step (with placeholder secrets if no certificate yet)
- [ ] Updater (`tauri-plugin-updater`): `latest.json` generation includes Windows platform entry (`windows-x86_64`) with NSIS download URL and signature
- [ ] `macOSPrivateApi: true` verified to not break Windows builds (documented as no-op)
- [ ] WebView2 runtime bootstrapping: verify Tauri NSIS template auto-downloads WebView2 Runtime if missing (default behavior). Confirm in `tauri.conf.json` bundle config or NSIS settings that the bootstrapper is enabled.
- [ ] Copilot plugin hook scripts: `#[cfg(unix)]` block in `settings_config.rs:573` has a Windows counterpart using `.bat` or `.ps1` hook scripts (or document that this feature is macOS/Linux-only)
- [ ] Release workflow produces both macOS (DMG, .app.tar.gz) and Windows (NSIS .exe) artifacts
- [ ] Both platforms' artifacts are downloadable from GitHub Releases

## Notes

- Tauri v2 NSIS is the recommended Windows installer format (over MSI). It supports auto-update, custom install paths, and silent install.
- Windows code signing uses Authenticode (`.pfx` certificate). Azure SignTool or `signtool.exe` via the `tauri-plugin-sign` or Tauri's built-in signing.
- The updater plugin handles Windows `.nsis.zip` artifacts if `latest.json` includes them with the correct platform key.
- GitHub Actions `windows-latest` runners include the Windows SDK and MSVC toolchain.
- The `justfile` uses shell commands — `just` supports `[windows]` and `[unix]` attributes for platform-conditional recipes.
- Existing CI secret variables for Apple signing won't interfere with Windows builds, but Windows-specific secrets need to be added.
- The `#[cfg(unix)]` in `settings_config.rs` for copilot hook file permissions (`set_permissions(0o755)`) — Windows doesn't use Unix permissions. Either use `.bat`/`.ps1` hooks on Windows or skip the permission step.

## Dependencies

- All feature tasks (01-08) should be complete before CI/CD produces release artifacts

## Related files

- `src-tauri/tauri.conf.json` (bundle config)
- `.github/workflows/ci.yml` (release pipeline — currently macOS-only)
- `.github/workflows/ci-build.yml` (build smoke test — currently macOS-only)
- `justfile` (build commands)
- `src-tauri/icons/` (icon assets)
- `src-tauri/src/settings_config.rs:573` (`#[cfg(unix)]` copilot hook permissions)
- `specs/ci_cd.md` (CI/CD pipeline spec — update in task 10)
