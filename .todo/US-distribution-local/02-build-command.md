---
status: done
---

# Build Command

## Goal

Add a `just build` command that produces a DMG, and verify the artifact works end-to-end (install, launch, basic functionality).

## Acceptance criteria

- [ ] `just build` recipe added to Justfile
- [ ] Running `just build` produces a `.dmg` file in `src-tauri/target/release/bundle/dmg/`
- [ ] Opening the DMG shows Cortado.app, draggable to Applications
- [ ] Installed app launches, shows tray icon, opens panel via hotkey
- [ ] `just check` still passes

## Notes

- The recipe should be: `pnpm exec tauri build` (installs JS deps first if needed, like `just dev` does).
- First build will be slow (release mode compilation). Subsequent builds are incremental.
- The DMG is unsigned/unnotarized at this stage — macOS will show a Gatekeeper warning. The developer can bypass with right-click > Open, or `xattr -cr` on the app bundle. Proper signing comes in US-distribution.
- Verify the app works when launched from `~/Applications/` or `/Applications/`, not just from the build output directory.

## Relevant files

- `Justfile` — add `build` recipe
- `src-tauri/target/release/bundle/dmg/` — build output (gitignored)
