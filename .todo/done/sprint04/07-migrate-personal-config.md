---
status: done
---

# Migrate personal feeds config to duration strings

## Goal

Migrate local developer config at `~/.config/cortado/feeds.toml` to the sprint04 duration-string shape.

## Acceptance criteria

- [x] Existing feeds specify `interval` using duration strings.
- [x] `github-pr` feed config remains valid after migration.
- [x] Migration is documented in sprint notes/spec updates where relevant.

## Notes

- This is environment-specific and not a product runtime behavior change.

## Relevant files

- `~/.config/cortado/feeds.toml`
