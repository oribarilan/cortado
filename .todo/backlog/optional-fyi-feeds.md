---
status: pending
---

# FYI Feeds (e.g., GitHub Release)

## Concept

Some feeds are informational rather than actionable -- they surface awareness ("React 19.1 dropped") rather than demanding action ("your CI is failing"). These feeds don't map naturally to cortado's status-driven attention model because their activities have no meaningful StatusKind progression.

This task covers two coupled ideas:

1. **Dismiss-until-update** -- a general-purpose mechanism that lets users dismiss an activity from the panel and tray, with the activity resurfacing only when its underlying data changes.
2. **FYI feeds** -- feed types (like GitHub Releases) that are primarily informational and benefit from dismiss-until-update as their core interaction model.

## Dismiss-until-update

### Behavior

- User dismisses an activity (UI gesture TBD -- swipe, button, keyboard shortcut).
- Dismissed activities are hidden from panel list and tray.
- Dismissed activities do NOT affect tray icon rollup.
- On next poll, if the activity's fields have changed, it resurfaces (becomes visible again).
- If the activity is unchanged, it stays dismissed.
- If the activity disappears from the feed entirely, the dismissal is discarded (nothing to track).

### What counts as "changed"?

- Any field value change (text, status, number, url) on the activity.
- For FYI feeds without status fields: a new activity appearing in the feed is inherently "new" and visible. Dismiss only applies to specific activities the user has already seen.

### Scope questions (to be decided)

- **Persistence:** Should dismissals survive app restart? If stored in memory only, restarting the app resurfaces everything. If persisted, need a storage mechanism (file, SQLite, etc.). Related to the existing `optional-retained-activity-persistence.md` backlog item.
- **Per-activity vs per-feed:** Dismissal is per-activity. But should there be a "dismiss all" action per feed? Probably yes for convenience.
- **Interaction with retain:** A dismissed activity that is also retained (disappeared from feed but within retain window) -- should it stay dismissed? Probably yes -- if it's retained AND dismissed, it's invisible. If it reappears in a future poll with changes, it resurfaces.
- **Interaction with notifications:** A dismissed activity that resurfaces -- should it trigger a notification? Probably yes, if the feed has `notify = true`. The user explicitly said "don't bother me until this changes," so a change is worth notifying about.
- **UI gesture:** Swipe-to-dismiss? A dismiss button? Keyboard shortcut? Context menu? This is a UX design question. Should be specced in `specs/ux_design.md` once decided.
- **Visual indicator:** Should there be any indication in the UI that dismissed activities exist? (e.g., "3 dismissed" label on the feed). Or is fully hidden the right UX?

### Implementation sketch

```rust
// In runtime or a new module
struct DismissalTracker {
    // Map of (feed_name, activity_id) -> field snapshot at time of dismissal
    dismissed: HashMap<(String, String), Vec<Field>>,
}

impl DismissalTracker {
    fn dismiss(&mut self, feed: &str, activity: &Activity) { ... }
    fn is_dismissed(&self, feed: &str, activity: &Activity) -> bool {
        // True if activity exists in dismissed map AND fields haven't changed
    }
    fn sweep(&mut self, active_ids: &HashSet<(String, String)>) {
        // Remove dismissals for activities no longer in any feed
    }
}
```

The snapshot comparison should use field values only (not labels or metadata) to determine if something "changed."

## GitHub Release feed (`github-release`)

The first candidate FYI feed.

### What it tracks

New releases of repositories the user follows. Each activity is a release.

### Data source

```sh
gh release list --repo OWNER/REPO --limit 10 --json tagName,name,publishedAt,isPrerelease,url,isDraft
```

### Auth & preflight

Same as `github-pr` / `github-actions` (shared `gh` preflight).

### Config

```toml
[[feed]]
name = "react releases"
type = "github-release"
repo = "facebook/react"

# Optional
include_prereleases = true     # Include prerelease versions (default: true)
include_drafts = false         # Include draft releases (default: false)
```

### Provided fields

| Field        | Type   | Label      | Description                    |
|-------------|--------|------------|--------------------------------|
| `tag`       | text   | Tag        | Release tag (e.g., v19.1.0)   |
| `prerelease`| status | Prerelease | Whether this is a prerelease   |
| `age`       | text   | Age        | Time since publication         |

### Status kind mapping

| Condition      | Value   | StatusKind        |
|----------------|---------|-------------------|
| prerelease     | `yes`   | AttentionPositive |
| stable release | `no`    | Idle              |

Minimal status model. The `prerelease` field gives a subtle visual signal but the feed is primarily informational.

### Activity identity

Release URL.

### Activity title

Release name, falling back to tag name.

### Default interval

`3600s` (1 hour -- releases are infrequent).

### Why this pairs well with dismiss-until-update

- User sees "React v19.1.0" in their feed.
- They acknowledge it (dismiss).
- Feed keeps polling. v19.1.0 stays dismissed.
- When v19.2.0 appears, it's a new activity (new ID) -- automatically visible.
- No resurface logic needed for *new* activities; dismiss only hides *existing* ones.

This means GitHub Release works even without the full dismiss-until-update mechanism -- old releases naturally scroll out as new ones appear, and the 20-activity cap keeps the list fresh. Dismiss-until-update is a nice-to-have for manually clearing noise, not a hard requirement.

## Other potential FYI feeds

Feeds that would benefit from this pattern in the future:

- **RSS/Atom** -- new blog posts, changelogs, security advisories
- **GitHub Notifications** -- unread notifications (dismiss = mark as read equivalent)
- **Dependency updates** -- new versions available (Renovate/Dependabot PRs, npm outdated)
- **Changelog/Release notes** -- similar to GitHub Release but for non-GitHub sources

## Open questions

1. Should dismiss-until-update be a prerequisite for shipping any FYI feed, or can FYI feeds ship independently (with dismiss as a later enhancement)?
2. Is "FYI feed" a formal concept in the type system (e.g., a trait method `fn is_fyi(&self) -> bool`), or is it just a convention that some feeds happen to have minimal/no status fields?
3. Should the tray rollup completely ignore FYI feeds, or should they contribute `Idle` at most?

## Relevant files

- `src-tauri/src/feed/mod.rs` -- possibly add dismiss tracking
- `src-tauri/src/feed/runtime.rs` -- filter dismissed activities from snapshots
- `src-tauri/src/feed/github_release.rs` -- new file (when implemented)
- `specs/main.md` -- document FYI feed concept and dismiss behavior
- `specs/ux_design.md` -- dismiss gesture and visual design
