---
status: deferred
---

# Spec contract update for notifications and config-change warning

## Goal

Align `specs/main.md` with sprint03 scope so implementation and spec do not diverge.

## Acceptance criteria

- [ ] `specs/main.md` defines per-feed notification configuration in the feed config format.
- [x] `specs/main.md` states that config is loaded at startup and requires restart to apply changes.
- [x] `specs/main.md` documents runtime detection of config changes with a persistent restart-required tray warning.
- [x] `specs/main.md` updates/removes the non-goal that currently excludes notifications.
- [ ] Notification scope is explicitly documented as MVP: new-activity notifications only.
- [x] Error-handling contract for config-change detection failures is documented (detection logs errors; runtime continues).
- [ ] Terminology remains Feed/Activity/Field throughout.

## Notes

- This task is spec-only; no runtime behavior changes here.
- Keep wording concise and implementation-agnostic where possible.

## Relevant files

- `specs/main.md`
