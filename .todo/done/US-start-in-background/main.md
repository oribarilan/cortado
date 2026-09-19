---
status: done
---

# Start in background

## Goal
Let tray-focused users launch Cortado without opening the panel, while keeping the current default.

## Definition of done
- [x] `general.start_in_background` defaults to `false`, accepts a boolean, and survives saving and reloading settings.
- [x] Enabling it skips only the startup panel show. Panel initialization, polling, tray access, and the hotkey remain available. App reopen shows rather than toggles the panel.
- [x] General settings exposes the option, preserves it during other saves, and resets it to `false`.
- [x] General saves provide inline confirmation and visible errors, using existing reduced-motion styles.
- [x] The spec and README describe the setting and next-launch behavior.
- [x] `just check` and frontend tests pass; isolated UI/startup checks record their results and limitations.

## Plan
1. Add the settings field and startup guard, with parsing and persistence regression tests.
2. Wire the General toggle through loading, all full-settings saves, and reset. Align General feedback and app reopen with the spec.
3. Update docs, run checks, and review the diff.

## Decisions
- User approved following the existing spec for app reopen and General inline save feedback, plus regression tests using existing frameworks.
- This option is separate from launch at login. GUI changes affect the next launch without hiding current windows or prompting an immediate restart.
- Preserve the pre-existing deletion of `videos/.opencode/skills/remotion-best-practices`.

## Verification
Parent-owned results:
- `just check`: passed without warnings; 574 Rust tests passed, 10 ignored; 22 OpenCode tests passed.
- `pnpm test:ts`: 49 tests passed. `pnpm build`: passed.
- Isolated native bundle (`sh.oribi.cortado.background-test.dev`) with temporary XDG config: missing/false opens the panel; true stays hidden. Both direct binary and LaunchServices launches checked via CoreGraphics window visibility. Reopen shows the panel, repeated reopen leaves it open, and reopen works with tray and hotkey disabled.
- An isolated local HTTP feed received both startup and subsequent polls while background startup was enabled.
- Headless Chrome rendered the built Settings UI with mocked Tauri commands: default off; persisted true reload; preservation through General, Notifications, and Terminals saves; reset to false without resetting unrelated sections; failed-save rollback/error; transient inline feedback; reduced-motion duration zero. Deferred-save mouse input could not issue an overlapping write. A failed hotkey reset did not show success.
- Existing hotkey registration and tray setup were inspected and remain outside the startup visibility guard. Actual login startup, global hotkey presses, and native tray clicks were not exercised, to avoid altering the running app's login or shortcut configuration.

## Review and follow-up
A read-only reviewer identified overlapping General saves and premature reset feedback. General edits now block input until the full-settings write finishes, and reset suppresses intermediate success feedback. Both fixes passed the browser checks above.

Rust regressions are permanent tests in `src-tauri/src/app_settings.rs`. Browser/native smoke support was temporary, with no new dependencies or production diagnostic hooks. Session artifacts: `/tmp/cortado-bg-tests/`; full check log: `/tmp/cortado-background-check.log`.
