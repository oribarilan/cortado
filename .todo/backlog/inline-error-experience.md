---
status: pending
---

# Inline error experience for feeds

## Problem

Feed errors (poll failures, config issues, auth problems) currently display as dedicated banner-style blocks (`.feed-error`) that break the visual rhythm of the activity list. They look foreign compared to the rest of the UI and disrupt the layout.

## Goal

Errors should be displayed inline, using the same visual language as activities. An errored feed should show an activity-like row (dot, title, chip) that communicates the error state. Expanding it reveals error details in the same detail region used for normal activity fields.

## Design principles

- No banners, toasts, or layout-breaking elements.
- Error rows look like activity rows — same grid, same spacing, same interaction (click to expand).
- The error state is communicated through the status dot color, chip text, and possibly a subtle row background tint.
- Expanded detail shows: error type, message, timestamp, retry info, and any actionable guidance.
- Multiple errors in the same feed could show as multiple error "activities" or a single consolidated row — to be decided via showcase.

## Workflow

1. **Showcase first** — use the `showcase` skill to explore alternatives before any implementation:
   - How should the error row look vs a normal activity row? (dot color, title text, chip style)
   - Should there be one error row per feed or one per error occurrence?
   - What goes in the expanded detail area? (raw message, structured fields, retry button?)
   - How does this interact with the panel vs the main-screen layout?
2. **Pick a direction** based on showcase review.
3. **Implement** the chosen design.

## Questions to explore in showcase

- Should errored feeds still show their last-known activities alongside the error row?
- What StatusKind maps to errors? `AttentionNegative` seems natural but that's already used for failing CI etc. — is that fine or do errors need their own visual treatment?
- Should the error row be pinned to the top of the feed section?

## Relevant files

- `src/styles.css` — `.feed-error` (current banner implementation)
- `src/main-screen/main-screen.css` — `.ms-feed-error`
- `src/App.tsx` — panel error rendering
- `specs/status.md` — StatusKind definitions
