---
status: done
---

# Apple Notarization

## Goal

Code-sign and notarize the macOS app bundle so users can install without Gatekeeper warnings.

## Acceptance criteria

- [ ] App is signed with a valid Apple Developer ID certificate
- [ ] App is notarized with Apple's notary service
- [ ] Signed + notarized app passes `spctl --assess --verbose` and `codesign --verify --deep`
- [ ] CI/CD pipeline handles signing and notarization automatically
- [ ] Install script downloads a notarized build — no `xattr -cr` needed

## Notes

- Requires an Apple Developer account ($99/year) and a "Developer ID Application" certificate.
- Notarization submits the app to Apple's servers for malware scanning. Typically takes 1-5 minutes.
- In CI, the signing identity and credentials are stored as GitHub Actions secrets:
  - `APPLE_CERTIFICATE` — base64-encoded `.p12` certificate
  - `APPLE_CERTIFICATE_PASSWORD` — certificate password
  - `APPLE_ID` — Apple ID email
  - `APPLE_TEAM_ID` — Apple Developer team ID
  - `APPLE_PASSWORD` — app-specific password for notarization
- Tauri's build system supports notarization natively via environment variables. See [Tauri code signing docs](https://v2.tauri.app/distribute/sign/macos/).
- This task integrates into the CD workflow (task 03) — the signing and notarization steps happen during the release build.
- Consider: should the dev build (`just dev`) also be signed, or only release builds?

## Relevant files

- `.github/workflows/cd.yml` — add signing/notarization steps
- `src-tauri/tauri.conf.json` — signing identity config
- `specs/cd.md` — document the signing/notarization process
