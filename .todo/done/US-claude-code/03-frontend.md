---
status: pending
---

# Frontend catalog entry

## Context

Add a `claude-code-session` entry to `FEED_CATALOG` in `feedTypes.ts` so the Settings UI knows how to present and configure Claude Code feeds.

**Value delivered**: Users can add and configure Claude Code session feeds from the Settings UI.

## Related Files

- `src/shared/feedTypes.ts` -- `FEED_CATALOG` definition and `FeedType` union

## Dependencies

- `02-backend.md` (Tauri command names must match)

## Acceptance Criteria

- [ ] `"claude-code-session"` is added to the `FeedType` union type
- [ ] A `CatalogFeedType` entry exists in the `"coding-agents"` provider group with:
  - `feedType`: `"claude-code-session"`
  - `name`: `"Claude Code Sessions"`
  - `label`: `"Claude Code Session"`
  - `description`: `"Track active Claude Code coding sessions"`
  - `icon`: Claude Code brand SVG from thesvg.org (already fetched and adapted):
    ```
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" fill-rule="evenodd" xmlns="http://www.w3.org/2000/svg"><path clip-rule="evenodd" d="M20.998 10.949H24v3.102h-3v3.028h-1.487V20H18v-2.921h-1.487V20H15v-2.921H9V20H7.488v-2.921H6V20H4.487v-2.921H3V14.05H0V10.95h3V5h17.998v5.949zM6 10.949h1.488V8.102H6v2.847zm10.51 0H18V8.102h-1.49v2.847z"/></svg>
    ```
  - `defaultInterval`: `"30s"`
  - `hideInterval`: `true` (file-watching, not timer-based)
  - `defaultNamePattern`: `"Claude Code"`
  - `fields`: `[]` (no user-configurable fields)
  - `dependency`: binary `"claude"`, name `"Claude Code"`, installUrl `"https://code.claude.com"`
  - `setup`: with `checkCommand`, `installCommand`, `uninstallCommand` matching the Tauri commands from task 02
  - `badge`: `"beta"` -- displays a beta tag on the feed type in the UI
  - `notes`: should include a note that this feed type is in early preview and feedback is welcome
- [ ] `CatalogFeedType` type has a new optional `badge?: string` field
- [ ] The badge is rendered in the Settings UI wherever feed types are listed (e.g., a small tag/pill next to the feed type name)
- [ ] `tsc --noEmit` passes

## Verification

- **Automated**: `just lint` passes (tsc --noEmit)
- **Ad-hoc**: Verify the entry appears in the correct provider group and all required fields are present

## Notes

### Badge

`CatalogFeedType` doesn't have a `badge` field yet. Add `badge?: string` to the type and render it as a small tag/pill in the Settings UI where feed types are shown (catalog cards, feed edit header, etc.). Keep the rendering generic -- other feed types may use it in the future.
