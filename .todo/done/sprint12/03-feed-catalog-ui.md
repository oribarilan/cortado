---
status: pending
---

# Feed Catalog UI

## Goal

Replace the current "+ New feed" flow (which opens a form with a feed type dropdown) with a two-step catalog: provider grid → feed type selection → pre-filled form. Implements Variant A from `showcases/feed-catalog-showcase.html`.

## Flow

1. User clicks "+ New feed" (or "Add your first feed" in empty state).
2. **Step 1 — Provider grid:** Cards in a responsive grid. Each card shows the provider SVG icon, name, feed type count, and a list of feed type names. Clicking a card advances to step 2.
   - **Auto-skip:** Providers with only one feed type (Azure DevOps, HTTP, Shell) skip step 2 and go directly to the form with the feed type pre-selected.
3. **Step 2 — Feed type list:** Cards for each feed type under the selected provider. Each card shows a representative SVG icon, name, and short description. Clicking a card advances to the form.
4. **Form:** The existing feed edit form, but with the feed type pre-selected and the type dropdown removed (or disabled/hidden). Breadcrumb shows the full path: Feeds → New Feed → {Provider} → {Feed Type}.

## Providers and feed types

Show only implemented (registered in `instantiate_feed()`) feed types. As of this sprint:

| Provider    | Feed Types                     | Icon      |
|-------------|--------------------------------|-----------|
| GitHub      | Pull Requests, Actions         | Octocat   |
| Azure DevOps| Pull Requests                  | ADO mark  |
| HTTP        | Health Check                   | Globe     |
| Shell       | Custom Command                 | Terminal  |

GitHub Issues is not implemented and should not appear in the catalog.
Vercel is in backlog and should not appear until implemented.

When future feeds are added, they register in the catalog by adding an entry to a provider/feed-type registry data structure in the frontend. This should be a simple static data structure (array of objects), not dynamic.

## Frontend data structure

```typescript
type FeedTypeCatalogEntry = {
  provider: string;                    // Display name: "GitHub", "Azure DevOps", etc.
  providerIcon: string;                // SVG markup string or component
  types: {
    feedType: FeedType;                // "github-pr", "github-actions", etc.
    name: string;                      // "Pull Requests", "Actions", etc.
    description: string;               // One-line description
    icon: string;                      // SVG markup string or component
    defaultInterval?: string;          // Per-type default: "2m", "1m", etc.
  }[];
};
```

## SVG icons

Use the SVGs from the showcase (`showcases/feed-catalog-showcase.html`):

**Provider icons (fill-based brand marks):**
- GitHub: Octocat mark (viewBox 0 0 16 16)
- Azure DevOps: Angular DevOps logo (viewBox 0 0 18 18)

**Provider icons (stroke-based, Lucide style):**
- HTTP: Globe icon
- Shell: Terminal icon

**Feed type icons (stroke-based, Lucide style):**
- Pull Requests: git-pull-request (two nodes with merge path)
- Actions: play-circle (circle with play triangle)
- Health Check: activity/pulse icon
- Custom Command: terminal chevron icon

All icons use `currentColor` for theme adaptation.

## Navigation

- Breadcrumb replaces the title area during the catalog flow:
  - Step 1: `New Feed` (no back link — this is the root)
  - Step 2: `New Feed > GitHub` (clicking "New Feed" returns to step 1)
  - Form: `Feeds > New Feed` remains as-is (existing breadcrumb pattern)
- The existing slide-in animation (`feed-slide-in-right` / `feed-slide-in-left`) should be used for transitions between steps.

## Changes to existing code

### Type system updates

The `FeedType` union must be expanded to include new feed types:

```typescript
type FeedType = "github-pr" | "github-actions" | "ado-pr" | "http-health" | "shell";
```

### Registry maps — add entries for new feed types

**`FEED_TYPE_LABELS`** — add display names:
- `"github-actions"`: `"GitHub Actions"`
- `"http-health"`: `"HTTP Health Check"`

**`FEED_TYPE_FIELDS`** — add type-specific config fields:
- `"github-actions"`:
  - `repo` (required, mono, placeholder: `owner/repo`)
  - `branch` (optional, mono, placeholder: `main`)
  - `workflow` (optional, mono, placeholder: `ci.yml`)
  - `user` (optional, mono, placeholder: `@me`)
- `"http-health"`:
  - `url` (required, mono, placeholder: `https://api.example.com/health`)
  - `method` (optional, mono, placeholder: `GET`, hint: `GET or HEAD`)
  - `expected_status` (optional, mono, placeholder: `200`)
  - `timeout` (optional, mono, placeholder: `10s`)

**`FEED_TYPE_DEPS`** — add dependency info:
- `"github-actions"`: same as `"github-pr"` (gh CLI, `gh auth login`)
- `"http-health"`: no entry (no external CLI dependency)

### Form changes

- Remove or hide the feed type `<select>` dropdown. The type is now pre-determined by the catalog selection.
- `emptyFeed()` receives the selected feed type from the catalog flow instead of defaulting to `"github-pr"`. Use `defaultInterval` from the catalog entry (e.g., `"2m"` for GitHub Actions, `"1m"` for HTTP Health) instead of the hardcoded `"5m"`.
- `startAdd()` no longer immediately calls `check_feed_dependency` for `"github-pr"` — it opens the catalog instead. Dependency checking happens after the feed type is selected, only if the type has a `FEED_TYPE_DEPS` entry.
- `validateFeed()` works unchanged — it already validates based on `FEED_TYPE_FIELDS[feed.type]`, so adding entries to that map is sufficient.
- For feed types with no external dependency (e.g., `http-health`), the dependency banner is not shown.

## Acceptance criteria

- [ ] `FeedType` union expanded with `"github-actions"` and `"http-health"`
- [ ] `FEED_TYPE_LABELS`, `FEED_TYPE_FIELDS` entries added for new feed types
- [ ] `FEED_TYPE_DEPS` entry added for `github-actions`; `http-health` has no entry
- [ ] "+ New feed" opens the provider grid (step 1), not the form directly
- [ ] Provider cards show SVG icon, name, feed type count, and type names
- [ ] Clicking a multi-type provider (GitHub) shows its feed type list (step 2)
- [ ] Clicking a single-type provider skips to the form with type pre-selected
- [ ] Feed type cards show SVG icon, name, and description
- [ ] Clicking a feed type card opens the form with that type pre-selected
- [ ] Feed type dropdown is removed from the form
- [ ] `emptyFeed()` uses per-type default interval from catalog, not hardcoded `"5m"`
- [ ] `startAdd()` opens catalog; dependency check runs after type selection, only for types with deps
- [ ] Feeds with no external dependency (http-health) skip the dependency banner
- [ ] Breadcrumb navigation works for back-navigation at each step
- [ ] Slide transitions between steps match existing animation patterns
- [ ] Catalog only shows implemented feed types (no Vercel, no GitHub Issues)
- [ ] Grid is responsive within the settings main area
- [ ] Dark and light theme work correctly (icons use currentColor)
- [ ] Reduced motion: slide transitions disabled under `prefers-reduced-motion`
- [ ] Empty state ("Add your first feed") also opens the catalog
- [ ] `just check` passes (tsc --noEmit)

## Notes

- This task is frontend-only (React + CSS in `src/settings/`).
- No backend changes needed — the catalog is a static data structure in the frontend.
- The provider/feed-type catalog data structure should be easy to extend when new feeds are added. Adding a new feed type should require: (1) adding a `FEED_TYPE_FIELDS` entry, (2) adding a `FEED_TYPE_DEPS` entry if the feed has an external CLI, (3) adding a catalog entry with icon/description/defaultInterval.
- Keep the catalog data co-located with the existing `FEED_TYPE_LABELS` and `FEED_TYPE_FIELDS` maps in `SettingsApp.tsx` (or extract to a separate file if `SettingsApp.tsx` is getting too large).
- Per-type default intervals should mirror the backend defaults: `github-pr` = `2m`, `github-actions` = `2m`, `ado-pr` = `2m`, `http-health` = `1m`, `shell` = `30s`.

## Relevant files

- `src/settings/SettingsApp.tsx` — main changes (catalog state, rendering, data)
- `src/settings/settings.css` — catalog grid, cards, breadcrumb styles
- `showcases/feed-catalog-showcase.html` — reference implementation for Variant A
