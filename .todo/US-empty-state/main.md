# US-empty-state: Empty State

## Problem

When Cortado has zero configured feeds — whether on first launch or after a user deletes all feeds — the panel and tray show blank content with no guidance. There's nothing explaining what the app does, what feeds are, or how to get started.

## Goal

Provide a clear, welcoming empty state in the panel that helps users understand Cortado and navigate to feed creation with minimal friction.

## Trigger

The empty state shows whenever **zero feeds are configured**. It's not a one-time event — if a user deletes all feeds, the empty state reappears. The same welcome copy is used regardless of whether this is a first launch or a re-entry.

## Chosen Approach: Panel Empty State with Settings Deep Links

The panel shows a welcoming empty state with feed type discovery. Actions navigate the user to the existing Settings add-feed flow. No new windows, modals, or inline creation forms.

### Panel behavior

The panel auto-opens when the app launches (regardless of feed count — this is a general behavior, not empty-state-specific).

When zero feeds are configured, the panel's split layout shows:

**List pane (left):**
- Welcome headline ("Welcome to Cortado")
- Short explanation of what a feed is
- Prominent CTA button ("+ Add your first feed") that opens Settings to the add-feed flow
- Secondary link ("or edit ~/.config/cortado/feeds.toml") for power users
- Hotkey hint (Cmd+Shift+Space to toggle panel)

**Detail pane (right):**
- Feed types list with icons and one-line descriptions
- Each feed type is clickable — acts as a deep link that opens Settings to the add-feed form with that feed type pre-selected

Once a feed exists, the empty state is replaced by the normal feed/activity list.

### Tray behavior

Unchanged from current behavior. No special empty state items in the tray for now.

### Navigation flow

1. User sees empty state in panel
2. User clicks CTA or a specific feed type
3. Settings window opens to the Feeds section, add-feed flow (pre-filled if a type was clicked)
4. User creates a feed in Settings
5. Panel updates to show the new feed (via existing `feeds-updated` event)

### Scope

- Panel empty state UI (list pane + detail pane content)
- Deep link from panel to Settings add-feed flow (with optional pre-selected feed type)
- Panel auto-open on app launch

### Out of scope

- Tray changes
- Inline feed creation in the panel
- Any wizard or multi-step flow

## Open Questions

- Does Settings currently support being opened to a specific feed type pre-selected? If not, a Tauri command needs to be added.

## Showcase

See `showcases/empty-state-showcase.html` — variant C2 is closest to the chosen approach, though the final implementation uses Settings for creation rather than inline forms.

## Task Sequencing

Tasks are sequential — each builds on the previous:

1. **Auto-open panel** — make the panel open on app launch (general improvement, not empty-state-specific)
2. **Settings deep-link** — extend `open_settings` to accept section + feed type params, so the panel can deep-link into the add-feed flow
3. **Panel empty state UI** — build the rich empty state with welcome message, feed type cards, and deep links to Settings
