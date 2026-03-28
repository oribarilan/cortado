# Cortado Spec

Cortado is a cross-platform extensible watcher that lives in the macOS menubar.
It gives developers a persistent, glanceable view of things they care about.

## Terminology

See `specs/glossary.md` for canonical definitions of all terms (Feed, Activity, Field, Status Kind, Status Value, etc.).

### Hierarchy

```
Feed: "GitHub PRs — personal/cortado"
  |-- Activity: PR #42 "Add feed scaffold"
  |     |-- Field: review    = awaiting   (status)
  |     |-- Field: checks    = passing    (status)
  |     |-- Field: mergeable = yes        (status)
  |     '-- Field: labels    = ["wip"]    (text)
  |
  '-- Activity: PR #38 "Fix tray icon"
        '-- Field: review    = approved   (status)
```

## Architecture

### Feed system

A **Feed** is the core extensibility unit. Each feed type:

1. Defines what **fields** it provides (with types and defaults).
2. Implements a **poll** method that discovers/updates activities.
3. Is configured via the user's config file.

Feed types ship as curated implementations in Rust. There is no external plugin system — extensibility comes from the `shell` feed type, which lets users run arbitrary commands.

### Feed trait contract

Every feed implementation must:

- Accept structured config from the TOML file.
- Declare its provided fields (name, label, type, description).
- Poll and return a list of activities, each with populated fields.
- Manage activity lifecycle (new, updated, gone).

### Feed identity

Feed names must be unique within the config file. The name is the feed's identity — duplicate names are a config error.

### GitHub authentication

The `github-pr` feed (and future GitHub feed types) depends on the [`gh` CLI](https://cli.github.com/) for authentication and API access. Cortado shells out to `gh` commands (for example, `gh pr list` and `gh api`) instead of managing tokens directly. This means:

- No auth config in `feeds.toml`.
- `gh` must be installed and authenticated (`gh auth login`).
- If `gh` is not available, GitHub feeds fail with this message: "GitHub feed requires `gh` CLI. Install it from https://cli.github.com/ and run `gh auth login`."
- If `gh` is available but not authenticated, GitHub feeds fail with this message: "GitHub feed requires `gh` authentication. Run `gh auth login` and retry."

### External CLI dependency contract

Feeds that rely on external CLIs must use a consistent dependency/error model:

- Dependency/auth failures are surfaced as **feed-level poll errors** (never app-global crashes).
- Errors are concise and actionable (what is missing + exact command/action to fix).
- Other valid feeds continue polling/rendering even if one feed has missing dependencies.
- Where canonical error copy is defined (as above for `github-pr`), implementations should use it verbatim.

Current dependency requirements:

- `github-pr`: requires `gh` installed and authenticated.
- `ado-pr`: requires `az` CLI, `azure-devops` extension, and authenticated access via `az login`.

#### `ado-pr` dependency checks (contract)

`ado-pr` implementations should follow this dependency preflight order and normalize failures to concise feed-level poll errors:

1. Verify `az` CLI is installed/invocable.
2. Verify `azure-devops` extension is installed (`az extension show --name azure-devops` or equivalent).
3. Verify authentication is available via `az login` state (`az account show` or equivalent).

Failures in any step should not crash the app globally; they should surface on that feed while other feeds continue polling/rendering.

Canonical `ado-pr` dependency/auth error copy:

- Missing `az` CLI: "Azure DevOps feed requires `az` CLI. Install it from https://aka.ms/install-azure-cli and run `az login`."
- Missing extension: "Azure DevOps feed requires `azure-devops` extension. Run `az extension add --name azure-devops`."
- Unauthenticated: "Azure DevOps feed requires `az` authentication. Run `az login` and retry."

### Default intervals

Each feed type defines a default poll interval used when `interval` is omitted from config:

| Feed type | Default interval |
|-----------|-----------------|
| `github-pr` | `"120s"` |
| `ado-pr` | `"120s"` |
| `shell` | `"30s"` |

Intervals use duration strings parsed by `jiff` (for example: `"30s"`, `"5m"`, `"1.5m"`, `"2h"`). Integer seconds are not supported.

### Activity retention

Feeds may opt into retention via `retain`, a duration string on each feed config.

- `retain` omitted ⇒ no retention (default)
- `retain = "2h"` ⇒ keep disappeared activities for up to 2 hours

Retention is a runtime lifecycle primitive:

- On successful poll, activities missing from the new poll result may be retained for the configured window.
- Retained activities are shown in menubar UI with a hollow dot (`◦`) marker.
- Retained activities render after active activities within each feed section.
- Retention is currently in-memory only; retained activities are cleared on app restart.

### Config loading

Config is loaded once at app launch. Changes to `feeds.toml` require restarting the app to take effect. (Hot-reload may be added later.)

If Cortado detects that `feeds.toml` changed while the app is running, it should surface a persistent menubar-level warning instructing the user to restart the app to apply updates.

### Error handling

Errors are surfaced per-feed in the UI, never silently swallowed.

- **Bad config** (missing required field, unknown type, duplicate name): the feed card renders the config error message instead of activities. Other valid feeds still load normally.
- **Poll failure** (network error, `gh` not installed, command failed): the feed card shows a feed-level error status. Previous activities may still be displayed if available.

### Field types

| Type | Description | Example |
|------|-------------|---------|
| `text` | Plain string | `labels: "wip, draft"` |
| `status` | String + status kind (see `specs/status.md`) | `review: awaiting (waiting)` |
| `number` | Numeric value | `response_time: 142` |
| `url` | Clickable link | `link: https://github.com/...` |

### Curated feed types (Phase 1)

| Feed type | Activities | Key fields |
|-----------|-----------|------------|
| `github-pr` | Open PRs per user/repo | review (status), checks (status), mergeable (status), draft (status), labels (text) |
| `ado-pr` | Active Azure DevOps PRs per org/project/repo | review (status), checks (status), mergeable (status), draft (status) |
| `shell` | Single activity (the command output) | User-defined |

Feed snapshots are capped to at most **20 activities** per feed after retention and ordering are applied.

### Future feed types (not in Phase 1)

- `github-actions` — CI workflow runs. Fields: status, duration, branch, trigger.
- `http-health` — endpoint monitoring. Fields: healthy (status), status_code (number), response_time (text).
- `docker` — running containers. Fields: state (status), health (status), uptime (text), image (text).

## Configuration

### Location

`~/.config/cortado/feeds.toml` — single file, source of truth.

The GUI can also create/edit this file. If the file doesn't exist, Cortado starts with no feeds and can guide the user to create one.

### Format

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "personal/cortado"
interval = "60s"
retain = "2h"

[[feed]]
name = "ADO PRs"
type = "ado-pr"
url = "https://dev.azure.com/your-org/your-project/_git/your-repo"
interval = "120s"

# Optional: override field display
[feed.fields.labels]
visible = false

[[feed]]
name = "Disk usage"
type = "shell"
command = "df -h / | tail -1 | awk '{print $5}'"
interval = "30s"
```

### Config rules

- `name` and `type` are required on every feed.
- Type-specific fields (e.g., `repo`, `url`, `command`) are flat, not nested.
- PR feed types support optional `user` author filter values:
  - `github-pr`: default `@me` when omitted; accepts GitHub login or `@me`
  - `ado-pr`: default `me` when omitted; accepts creator identity (prefer email/UPN) or `me`
- `interval` is a duration string (for example `"30s"`, `"5m"`, `"1.5m"`).
- `retain` is an optional duration string. When omitted, activities are not retained after they disappear from poll results.
- `[feed.fields.<name>]` allows overriding visibility, label, etc.
- The base feed entity defines the field override contract; curated types (like `github-pr`) provide defaults.

### `ado-pr` field mapping contract (initial)

`ado-pr` should map Azure DevOps CLI/REST states deterministically to Cortado status fields.

Review aggregation from reviewer votes (`10`, `5`, `0`, `-5`, `-10`):

- Any `-10` → `rejected` (attention-negative)
- Else any `-5` → `changes requested` (attention-negative)
- Else if all required reviewers have vote `>= 5` → `approved` (attention-positive)
- Else → `awaiting` (waiting)

Mergeable mapping from `mergeStatus`:

- `succeeded` → `yes` (idle)
- `conflicts` → `no` (attention-negative)
- `rejectedByPolicy` → `blocked` (waiting)
- `queued` → `checking` (running)
- `failure` → `failed` (attention-negative)
- `notSet` → `notSet (unknown)` (idle)
- Any unrecognized state `X` → `X (unknown)` (idle)

`ado-pr` polling scope for initial implementation is active PRs only (`--status active`).

Checks rollup from `az repos pr policy list --id <PR_ID>` (CI policies only — Build and Status types; reviewer/approval policies are excluded since the `review` field covers that):

- any `rejected` or `broken` → `failed` (attention-negative)
- any `queued` or `running` with expired build context (`isExpired: true`) → `failed` (attention-negative); ADO auto-requeues builds that may never run (e.g., file-pattern scoped), leaving them as `queued` indefinitely
- else any `queued` or `running` → `running` (running)
- `notApplicable` is ignored in rollup
- else → `succeeded` (idle)
- zero policies or all `notApplicable` → `succeeded` (idle)
- unknown/unexpected states are ignored in rollup; if all non-`notApplicable` policies are unknown, the result is `<state> (unknown)` (idle)
- per-PR policy-call failures produce `unknown` (idle) without failing the whole feed poll
- policy calls use bounded concurrency (max 5 in flight) with the same per-call timeout as the main poll (30s)

## Tech stack

| Layer | Technology |
|-------|-----------|
| Shell | Tauri v2 (Rust) |
| UI | React + TypeScript + Vite |
| Menubar | tauri-nspanel + tauri-toolkit (macOS panel behavior) |
| Config | TOML (`~/.config/cortado/feeds.toml`, `~/.config/cortado/settings.toml`) |
| Notifications | tauri-plugin-notification (macOS Notification Center) |
| Package manager | pnpm |
| Dev shell | Nix flake |
| Task runner | Just |

## Platform

Phase 1 is macOS only. The app runs as an `Accessory` (no dock icon), with a tray icon that opens a menubar-attached panel.

### Menubar UX (panel disclosure, Strict System)

- Top level groups by **Feed**.
- Feed headers are non-interactive and visually normal (not disabled-looking).
- Each **Activity** is a compact row prefixed by a derived status dot.
- **Retained Activities** use a hollow dot (`◦`) prefix.
- Retained activities are listed after active activities within each feed.
- Activity rows support inline disclosure.
- Expanding an activity row reveals all **Field** entries (`label: value`) inline.
- Dot color is derived from status kinds using the precedence defined in `specs/status.md`.
- Feed-level config and poll errors are shown inline within the feed section.

## Notifications

Cortado sends macOS Notification Center alerts when activity statuses change. Notification behavior is configurable at two levels: global preferences in `settings.toml` and per-feed toggles in `feeds.toml`.

### Trigger model

Notifications fire when an activity's **rollup kind** changes — the highest-priority `StatusKind` across all its status fields shifts (e.g., Waiting → AttentionNegative). New and removed activities are also optionally notifiable.

### Configuration layers

1. **Global settings** (`~/.config/cortado/settings.toml`) — master toggle, notification mode, delivery preset, new/removed activity toggles.
2. **Per-feed toggle** (`feeds.toml`) — `notify = false` disables notifications for a specific feed. Default is `true` (opt-out model).

### Notification modes

| Mode | Behavior |
|------|----------|
| `all` (default) | Any rollup kind change fires a notification |
| `escalation_only` | Only when the new kind is higher priority than the old kind |
| `specific_kinds` | Only when the new kind is in the configured set |

### Delivery presets

| Preset | Behavior |
|--------|----------|
| `grouped` (default) | At most one notification per feed per poll cycle |
| `immediate` | One notification per activity change |

### Startup suppression

Notifications are suppressed during the initial startup seed poll. The first poll establishes the baseline — it is not a "change."

### Settings reload behavior

The master `enabled` toggle takes effect immediately (read live from shared state). All other notification settings take effect on the next poll cycle.

### Click action

Clicking a notification is handled by the OS. Future: open the activity's URL.

### `settings.toml` format

```toml
[notifications]
enabled = true
mode = "all"               # "all", "escalation_only", or "specific_kinds"
# kinds = ["attention-negative", "attention-positive"]  # only when mode = "specific_kinds"
delivery = "grouped"       # "grouped" or "immediate"
notify_new_activities = true
notify_removed_activities = true
```

### Per-feed notify toggle

```toml
[[feed]]
name = "Noisy feed"
type = "github-pr"
repo = "org/mono"
notify = false  # Suppress notifications for this feed
```

## Non-goals (Phase 1)

- External plugin system (WASM, JS, or otherwise).
- Windows/Linux support.
- Notification digest mode (time-window summary batching — deferred to backlog).
- Notification scheduling / DND / quiet hours.
- Notification history / log.
- Persistent storage beyond config files.

## Main Screen

The main screen is a floating, keyboard-centric panel opened via a global hotkey. It coexists with the menubar panel — both remain accessible.

### Activation

- **Global hotkey**: ⌘+Shift+Space toggles the panel (press again to hide).
- **App reopen**: Launching Cortado while it's already running (via Spotlight, Finder, or `open -a`) also opens the main screen.

### Panel behavior

- Floating NSPanel, non-activating, centered on the monitor with the cursor.
- Hides on: Esc, clicking outside (resign key), desktop/space change, pressing the hotkey again.
- State resets on each show: scroll to top, focus first activity.
- No Dock icon — the app remains an Accessory (`ActivationPolicy::Accessory`).

### Layout

Split panel (~700×480), 60/40 flex ratio:

- **List pane (flex 3)**: Full-width rows grouped by feed. Each row shows a status dot, title, and inline status chip. Full keyboard navigation with ↑↓. Accent-soft highlight on focused row.
- **Detail pane (flex 2)**: Inset background, shows the focused activity's full un-truncated title, all field rows, and an "Open" link. No feed label or status chip — the inline chip on the list row already shows status. Updates instantly as focus moves.
- Enter opens the activity URL.

### Priority section (Needs Attention)

When enabled, a "⚑ Needs Attention" section appears at the top of the list, before feed groups. It aggregates activities with `AttentionNegative` as their derived status kind from all feeds, with a feed-hint label on each row.

- Activities in this section are **deduplicated** from their feed group below.
- Hidden when there are no attention-negative activities.
- Toggleable via `main_screen.show_priority_section` in `settings.toml` (default: `true`). Accessible from Settings > General.

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate activities |
| Enter | Open focused activity URL |
| Esc | Close panel |
| ⌘, | Open Settings |
| ⌘Q | Quit Cortado |

### Footer

Shows keyboard hints and a gear icon to open Settings.

## App Mode

The menubar (tray icon + menubar panel) is optional via the `show_menubar` setting.

- `show_menubar = true` (default): Both tray icon and menubar panel are available. The main screen is also available via hotkey.
- `show_menubar = false`: No tray icon. The app is accessed via the global hotkey (⌘+Shift+Space) or by re-launching from Spotlight/Finder.

The setting takes effect on next app launch. Settings are always accessible from the main screen footer or via ⌘, from the main screen.
