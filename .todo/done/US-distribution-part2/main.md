---
status: done
---

# US: App Distribution (Part 2 -- Auto-Update)

## Theme

In-app auto-update pipeline for Cortado: Tauri updater for seamless updates, and a built-in feed to surface update availability.

## Prerequisites

- US-distribution (Part 1) must be complete: CI, CD (DMG on GitHub Releases), notarization, semver.

## Context

- **Current state (after Part 1):** Users download a signed, notarized DMG from GitHub Releases and install manually. No auto-update mechanism.
- **Target:** The app knows when a new version is available (via a built-in feed), and Tauri's updater plugin handles the actual update download, verification, and installation.
- **Tauri updater:** Tauri has a built-in `tauri-plugin-updater` that checks a `latest.json` endpoint (e.g., on GitHub Releases), verifies cryptographic signatures, downloads the `.app.tar.gz` bundle, and replaces the running app. It requires a signing keypair generated via `tauri signer generate`.

## Sequencing

```
01-update-feed     Built-in update awareness feed (Tauri updater + activity feed)
```

Single task. Depends on updater infrastructure (signing, `latest.json`) which builds on the CD pipeline from Part 1.

## Tasks

| # | Task | Description |
|---|------|-------------|
| 01 | Update awareness feed | Built-in Cortado feed using Tauri updater plugin. Checks `latest.json`, surfaces "v0.X.0 available" as an activity. User clicks to install. Always visible in tray and panel. |

## Cross-cutting notes

### Signing keys
- The Tauri updater requires a signing keypair. The private key must be stored as a GitHub Actions secret (`TAURI_SIGNING_PRIVATE_KEY`). The public key goes into `tauri.conf.json`.
- Generate once with `pnpm exec tauri signer generate -w ~/.tauri/cortado.key`.
- **Never commit the private key.** If lost, existing users cannot receive updates.

### CD enhancements needed
- The CD pipeline (from Part 1) will need to be extended to also produce updater artifacts:
  - `createUpdaterArtifacts: true` in tauri.conf.json
  - `.app.tar.gz` + `.app.tar.gz.sig` alongside the DMG
  - `latest.json` with version, platform URL, signature, and release notes

### Tauri updater plugin
- Plugin: `tauri-plugin-updater` (already in the Tauri ecosystem, not yet added to this project).
- Endpoint: static `latest.json` on GitHub Releases (simplest approach, no server needed).
- The updater checks `latest.json`, compares versions, downloads + verifies signature, installs, and optionally relaunches.
- Formal docs: https://v2.tauri.app/plugin/updater/

### Update feed UX
- The update feed is a standard feed implementing the existing `Feed` trait -- no new event feed architecture needed.
- Activities use `AttentionPositive` StatusKind and participate in normal rollup.
- No dismiss option -- the update activity stays visible until installed.
- Expanding the activity shows release notes; an action triggers the update.

### Install script considerations
- Removed from scope -- not implementing `curl | bash` install script.
- First-time install remains manual (DMG from GitHub Releases).
- The updater handles all subsequent updates in-app.
