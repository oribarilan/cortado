---
status: done
---

# Remove Nix

## Goal

Remove the Nix flake and related files from the repo. They are not used and add unnecessary complexity.

## Acceptance criteria

- [ ] Remove `flake.nix`, `flake.lock`
- [ ] Remove `.envrc`, `.direnv/`
- [ ] Update `.gitignore` if it references Nix/direnv paths
- [ ] Document prerequisites (Rust, Node.js, pnpm, cargo-tauri) in `CONTRIBUTING.md`
- [ ] Update `AGENTS.md` -- remove the "Nix flake provides the dev shell" line in the Development section

## Notes

- The Justfile commands (`just dev`, `just check`, etc.) don't depend on Nix -- they call pnpm and cargo directly.
- CI will use direct toolchain setup, not Nix.

## Relevant files

- `flake.nix` (to delete)
- `flake.lock` (to delete)
- `.envrc` (to delete)
- `.direnv/` (to delete)
- `.gitignore`
- `AGENTS.md` -- Development section
- `CONTRIBUTING.md` -- prerequisites
