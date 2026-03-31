---
status: in-progress
---

# Apple Notarization

## Goal

Code-sign and notarize the macOS app bundle so users can install without Gatekeeper warnings.

## Acceptance criteria

- [x] CD workflow handles signing and notarization automatically (workflow code done)
- [x] Documented in `specs/cd.md`
- [ ] GitHub Actions secrets configured
- [ ] Test release verifies signed + notarized DMG

## Setup steps

### Step 1: Create the signing certificate

1. On your Mac, open **Keychain Access** > Certificate Assistant > Request a Certificate from a Certificate Authority (save to disk)
2. Go to [Apple Developer > Certificates](https://developer.apple.com/account/resources/certificates/list)
3. Click **+** > choose **Developer ID Application** > upload your CSR
4. Download the `.cer` file and double-click to install it in your keychain

### Step 2: Export the certificate as `.p12`

1. In Keychain Access, click **My Certificates** tab
2. Find your **Developer ID Application** entry, expand it
3. Right-click the **private key** underneath > Export
4. Save as `.p12`, set a strong password
5. Convert to base64: `base64 -A -in certificate.p12 | pbcopy`

### Step 3: Find your signing identity

Run:
```
security find-identity -v -p codesigning
```
Copy the full string like: `Developer ID Application: Your Name (XXXXXXXXXX)`

### Step 4: Create an App Store Connect API key

1. Go to [App Store Connect > Users and Access > Integrations > Keys](https://appstoreconnect.apple.com/access/integrations/api)
2. Click **+**, name it (e.g., "Cortado CI"), select **Admin** or **App Manager** role
3. Note the **Issuer ID** (above the table) and **Key ID** (in the table)
4. Download the `.p8` private key (can only be downloaded once!)
5. Convert to base64: `base64 -A -in AuthKey_XXXXXXXX.p8 | pbcopy`

### Step 5: Add GitHub Actions secrets

Go to [github.com/oribarilan/cortado/settings/secrets/actions](https://github.com/oribarilan/cortado/settings/secrets/actions) and add:

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE` | Base64 of your `.p12` file (from step 2) |
| `APPLE_CERTIFICATE_PASSWORD` | Password you set when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | Full identity string (from step 3) |
| `APPLE_API_ISSUER` | Issuer ID from App Store Connect (from step 4) |
| `APPLE_API_KEY` | Key ID from App Store Connect (from step 4) |
| `APPLE_API_KEY_PATH` | Base64 of the `.p8` file (from step 4) |
| `KEYCHAIN_PASSWORD` | Any strong password (e.g., generate with `openssl rand -base64 32`) |

### Step 6: Test with a release

After secrets are set, do a test release to verify the full pipeline end-to-end.
