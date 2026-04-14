---
status: done
---

# Backend: fetch and parse changelog

## Goal

During the update check, fetch `CHANGELOG.md` from GitHub and extract all entries between the user's current version and the latest available version. Attach the result to the update activity so the frontend can display it.

## Context

The update feed already makes a network request to `latest.json`. Adding a second fetch to grab the raw changelog from the same repo is low-cost and keeps the release pipeline unchanged (no need to embed changelog in `latest.json`).

The changelog follows Keep a Changelog format with `## [x.y.z]` version headers and `### Added/Changed/Fixed` section headers. Entries are single-line bullets.

The changelog parser already exists in `src-tauri/src/feed/changelog.rs` with 27 unit tests. This task wires it into the update feed.

## Design

Fetch `https://raw.githubusercontent.com/oribarilan/cortado/main/CHANGELOG.md` alongside the existing `latest.json` fetch (or concurrently). Parse the markdown using `changelog::extract_range()` to extract entries for all versions where `current < version <= latest`. Serialize the result as JSON and attach it as a `FieldValue::Text` field named `changelog` on the update activity.

Considerations:
- Changelog fetch failures should not block the update check -- fall back to no changelog (log a warning).
- The parsed result is serialized as JSON into a Text field. The frontend will parse it back into structured data.
- Strip the `[Unreleased]` section (already handled by the parser).
- If current version is 0.12.0 and latest is 0.14.0, include entries from both 0.13.0 and 0.14.0.

## Acceptance criteria

- [ ] Update feed fetches CHANGELOG.md from GitHub when an update is available
- [ ] Uses `changelog::extract_range()` to extract entries between current and latest versions
- [ ] Changelog JSON is attached to the app-update activity as a `changelog` field
- [ ] Changelog fetch failure does not break the update check
- [ ] Entries preserve section grouping (Added/Changed/Fixed) and bullet text
- [ ] `[Unreleased]` section is excluded (already handled by parser)
- [x] Unit tests cover: parsing multiple versions, single version, missing changelog, malformed markdown (done in `changelog.rs`)

## Related files

- `src-tauri/src/feed/cortado_update.rs` -- update feed poll logic (wire in here)
- `src-tauri/src/feed/changelog.rs` -- parser (already implemented and tested)
- `CHANGELOG.md` -- source format reference
