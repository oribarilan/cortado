# Cortado Spec

Cortado is a cross-platform extensible watcher that lives in the macOS menubar.
It gives developers a persistent, glanceable view of things they care about.

## Terminology

| Term | Definition |
|------|-----------|
| **Feed** | A configured data source that discovers and tracks related items. Example: "GitHub PRs for personal/cortado". |
| **Activity** | An individual tracked item within a feed, discovered and managed by the feed's lifecycle. Example: PR #42 "Add feed scaffold". |
| **Field** | A typed, structured piece of data on an activity. Fields have a name, label, value, and type. Example: `review: awaiting` (status field). |
| **Status** | The overall state of an activity, derived from its fields. |

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

Current + planned dependency requirements:

- `github-pr`: requires `gh` installed and authenticated.
- `ado-pr` (future): requires `az` CLI, `azure-devops` extension, and authenticated access (logged-in state and/or PAT/env-based auth as supported by the implementation).

#### Future `ado-pr` dependency checks (contract)

Any future `ado-pr` feed implementation should follow this dependency preflight order and normalize failures to concise feed-level poll errors:

1. Verify `az` CLI is installed/invocable.
2. Verify `azure-devops` extension is installed (`az extension show --name azure-devops` or equivalent).
3. Verify authentication is available (logged-in state and/or PAT/env strategy used by the implementation).

Failures in any step should not crash the app globally; they should surface on that feed while other feeds continue polling/rendering.

### Default intervals

Each feed type defines a default poll interval (in seconds) used when `interval` is omitted from config:

| Feed type | Default interval |
|-----------|-----------------|
| `github-pr` | 120 |
| `shell` | 30 |

### Config loading

Config is loaded once at app launch. Changes to `feeds.toml` require restarting the app to take effect. (Hot-reload may be added later.)

### Error handling

Errors are surfaced per-feed in the UI, never silently swallowed.

- **Bad config** (missing required field, unknown type, duplicate name): the feed card renders the config error message instead of activities. Other valid feeds still load normally.
- **Poll failure** (network error, `gh` not installed, command failed): the feed card shows a feed-level error status. Previous activities may still be displayed if available.

### Field types

| Type | Description | Example |
|------|-------------|---------|
| `text` | Plain string | `labels: "wip, draft"` |
| `status` | String with severity (success, warning, error, pending, neutral) | `review: awaiting (pending)` |
| `number` | Numeric value | `response_time: 142` |
| `url` | Clickable link | `link: https://github.com/...` |

### Curated feed types (Phase 1)

| Feed type | Activities | Key fields |
|-----------|-----------|------------|
| `github-pr` | Open PRs per user/repo | review (status), checks (status), mergeable (status), draft (status), labels (text) |
| `shell` | Single activity (the command output) | User-defined |

### Future feed types (not in Phase 1)

- `github-actions` — CI workflow runs. Fields: status, duration, branch, trigger.
- `ado-pr` — Azure DevOps pull requests. Fields: review (status), mergeable (status), labels (text). Initial implementation may defer checks/build policy details to avoid N+1 API calls.
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
interval = 60

# Optional: override field display
[feed.fields.labels]
visible = false

[[feed]]
name = "Disk usage"
type = "shell"
command = "df -h / | tail -1 | awk '{print $5}'"
interval = 30
```

### Config rules

- `name` and `type` are required on every feed.
- Type-specific fields (e.g., `repo`, `command`) are flat, not nested.
- `interval` is an integer (seconds). Defaults to a sane value per feed type if omitted.
- `[feed.fields.<name>]` allows overriding visibility, label, etc.
- The base feed entity defines the field override contract; curated types (like `github-pr`) provide defaults.

## Tech stack

| Layer | Technology |
|-------|-----------|
| Shell | Tauri v2 (Rust) |
| UI | React + TypeScript + Vite |
| Menubar | tauri-nspanel + tauri-toolkit (macOS panel behavior) |
| Config | TOML (`~/.config/cortado/feeds.toml`) |
| Package manager | pnpm |
| Dev shell | Nix flake |
| Task runner | Just |

## Platform

Phase 1 is macOS only. The app runs as an `Accessory` (no dock icon), with a tray icon that opens a native macOS menubar menu.

### Menubar UX (native menu)

- Top level groups by **Feed**.
- Each **Activity** is a submenu item prefixed by a derived status dot.
- Activity title rows are compact and do not include full field details inline.
- Expanding an activity submenu reveals all **Field** entries (`label: value`).
- Dot color/severity is derived from status fields using this precedence:
  1. `error` → red
  2. else `warning` → yellow
  3. else `pending` → blue
  4. else `success` → green
  5. else neutral/no status → gray
- Feed-level config and poll errors are shown at the feed submenu level.

## Non-goals (Phase 1)

- External plugin system (WASM, JS, or otherwise).
- Windows/Linux support.
- Push notifications (polling only).
- Persistent storage beyond the config file.
