---
status: pending
---

# Install Script

## Goal

Create a one-liner install script that users can run to download and install Cortado from GitHub Releases.

## Acceptance criteria

- [ ] `install` script at repo root (no extension, executable)
- [ ] Hosted at raw GitHub URL: `curl -fsSL https://raw.githubusercontent.com/oribarilan/cortado/main/install | bash`
- [ ] Detects OS (macOS only — exits with clear message on Linux/Windows)
- [ ] Detects architecture (aarch64 for now — exits with message on x86_64 until supported)
- [ ] Downloads the latest `.app.tar.gz` from GitHub Releases
- [ ] Extracts to `~/Applications/Cortado.app`
- [ ] Creates `~/Applications/` if it doesn't exist
- [ ] Handles upgrade: replaces existing `Cortado.app` if present
- [ ] Prints clear success message with how to launch
- [ ] Supports `VERSION=v0.3.0 curl ... | bash` for pinning a specific version
- [ ] Clean error messages for: no internet, GitHub rate limit, missing dependencies (curl, tar)
- [ ] README.md updated with install instructions

## Notes

- Pattern follows opencode.ai: minimal bash script, no package manager dependency.
- The script is for first-time install only. Subsequent updates are handled by the Tauri updater plugin in-app.
- Since the app is notarized (task 04), no `xattr` workaround needed.
- Keep the script simple and auditable — users should be able to read it before piping to bash.
- Consider printing a one-line command to add Cortado to Login Items for auto-start (or mention the `tauri-plugin-autostart` handles this).

## Relevant files

- `install` (to create, at repo root)
- `README.md` — install instructions
