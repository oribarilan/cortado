---
status: pending
---

# Task: Spec and documentation updates

## Goal

Update the project specification and documentation to reflect cross-platform support, replacing "macOS only" language with cross-platform terminology and documenting Windows-specific behavior.

## Acceptance criteria

- [ ] `specs/main.md` "Platform" section updated: replace "Phase 1 is macOS only" with cross-platform support description, document macOS and Windows behaviors
- [ ] `specs/main.md` tech stack table: note that `tauri-nspanel` and `tauri-toolkit` are macOS-only; document Windows window management approach
- [ ] `specs/main.md` menubar UX section: document platform differences (NSPanel on macOS vs standard window on Windows)
- [ ] `specs/main.md` tray icon section: document template mode (macOS) vs colored icon (Windows)
- [ ] `specs/main.md` notifications section: reference both macOS Notification Center and Windows notification system
- [ ] `specs/main.md` non-goals: remove "Windows/Linux support" — Windows is now supported; Linux remains a non-goal
- [ ] `specs/main.md` terminal focus: document as "macOS only" with note that Windows support may come later
- [ ] `AGENTS.md` gotchas: macOS-specific gotchas (no `tokio::spawn` in setup, no `block_on`, PATH resolution, packaged app installs) are annotated as macOS-specific where applicable. Add any Windows-specific gotchas discovered during implementation.
- [ ] `AGENTS.md` code organization: document cfg-dispatched module pairs pattern (e.g., `panel_macos.rs` / `panel_windows.rs` + dispatcher `panel.rs`)
- [ ] `AGENTS.md` code organization: document that `app_env.rs` config directory uses `dirs::config_dir()` on Windows, `~/.config/cortado` on macOS
- [ ] `README.md`: update installation/download section to include Windows, update feature list, add Windows-specific notes
- [ ] `specs/glossary.md`: update Panel definition from "floating NSPanel" to "floating panel (NSPanel on macOS, always-on-top window on Windows)"
- [ ] `CHANGELOG.md`: add entry for Windows support
- [ ] `specs/ci_cd.md`: document Windows build and signing pipeline, NSIS artifacts, `latest.json` multi-platform format

## Notes

- Keep spec language platform-neutral where possible. Only call out platform differences where behavior actually differs.
- Terminal focus should be documented as "macOS only" in the spec with a note that Windows support is a future enhancement.
- The glossary entry for "Panel" currently says "floating NSPanel" — should reference both platforms.
- The "Gotchas" section in `AGENTS.md` is heavily macOS-focused. Some gotchas (like `tokio::spawn` in setup) may apply to both platforms; annotate accordingly.
- Document the config directory difference prominently — it's a cross-cutting concern that affects users and developers.

## Dependencies

- All other tasks (01-09) should be complete or nearly complete so docs reflect final behavior

## Related files

- `specs/main.md`
- `specs/glossary.md`
- `AGENTS.md`
- `README.md`
- `CHANGELOG.md`
- `specs/ci_cd.md`
