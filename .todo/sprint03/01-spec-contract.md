---
status: pending
---

# Spec contract update for notifications and reload

## Goal

Align `specs/main.md` with sprint03 scope so implementation and spec do not diverge.

## Acceptance criteria

- [ ] `specs/main.md` defines per-feed notification configuration in the feed config format.
- [ ] `specs/main.md` states that config is loaded at startup and also on explicit tray `Reload`.
- [ ] `specs/main.md` clarifies that sprint03 adds manual reload (not file-watcher/hot reload).
- [ ] `specs/main.md` updates/removes the non-goal that currently excludes notifications.
- [ ] Notification scope is explicitly documented as MVP: new-activity notifications only.
- [ ] Error-handling contract for reload failures is documented (last-known-good state remains active).
- [ ] Terminology remains Feed/Activity/Field throughout.

## Notes

- This task is spec-only; no runtime behavior changes here.
- Keep wording concise and implementation-agnostic where possible.

## Relevant files

- `specs/main.md`
