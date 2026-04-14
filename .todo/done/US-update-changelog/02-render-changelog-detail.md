---
status: done
---

# Frontend: render changelog in detail pane

## Goal

Display the changelog entries in the update activity's detail pane (panel) and expanded detail (tray) so the user can see what changed before installing.

## Context

The detail pane currently renders fields as flat `label: value` pairs. Changelog data has structure: version headers, section headers (Added, Changed, Fixed), and bullet-point entries. The pane needs to render this in a readable way without introducing a full markdown renderer.

## Design decision

**Variant B (Collapsible per-version), all expanded by default.** Based on showcase evaluation (`showcases/update-changelog-showcase.html`).

Each version gets a collapsible section with a disclosure chevron. All versions are expanded by default so the user can scan everything at a glance. They can collapse versions they've already read. This gives the scannable structure of collapsible sections (version headers as landmarks, section groupings) without requiring clicks for the common case.

Section headings (Added/Changed/Fixed) are color-coded: Added = green, Changed = yellow, Fixed = blue (using existing status token colors).

## Implementation

The backend provides changelog data as a JSON-serialized `FieldValue::Text` field named `changelog`. The frontend parses it into `ChangelogVersion[]`:

```typescript
type ChangelogVersion = {
  version: string;
  date: string | null;
  sections: { heading: string; entries: string[] }[];
};
```

Create a single shared `Changelog` component used by both surfaces. Render below the existing fields, above "Updated X ago":
1. **Panel detail pane** (`DetailPane` component in `MainScreenApp.tsx`)
2. **Tray expanded detail** (detail-body in `App.tsx`)

Both surfaces use the same collapsible per-version rendering. If nested disclosure in the tray feels wrong in practice, the chevrons can be hidden with a CSS-only change.

Filter the `changelog` field out of normal field rendering (it's not a key-value pair).

No markdown library needed. Consult `specs/ux_design.md` for spacing and typography conventions.

## Acceptance criteria

- [ ] Update activity detail pane shows changelog entries below the existing fields
- [ ] Changelog renders in both panel (DetailPane) and tray (App.tsx detail-body)
- [ ] Each version has a collapsible disclosure section, all expanded by default
- [ ] Section headers (Added, Changed, Fixed) are color-coded and visually distinct
- [ ] Multiple versions are shown in order (newest first)
- [ ] Long changelogs scroll within the detail pane (no layout overflow)
- [ ] Empty changelog (fetch failed or no entries) shows no changelog section (not an error)
- [ ] Looks correct in both light and dark themes
- [ ] Respects prefers-reduced-motion for disclosure animations

## Dependencies

- Task 01 (backend must provide changelog data as a field)

## Related files

- `src/main-screen/MainScreenApp.tsx` -- `DetailPane` component
- `src/main-screen/main-screen.css` -- detail pane styles
- `src/App.tsx` -- tray panel detail rendering
- `src/styles.css` -- tray panel styles
- `src/shared/types.ts` -- `FieldValue` type
- `specs/ux_design.md` -- design conventions
- `showcases/update-changelog-showcase.html` -- reference showcase
