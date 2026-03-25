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
| `ado-pr` | Active Azure DevOps PRs per org/project/repo | review (status), checks (status), mergeable (status), draft (status), labels (text) |
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
org = "https://dev.azure.com/your-org"
project = "your-project"
repo = "your-repo"
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
- Type-specific fields (e.g., `repo`, `org`, `project`, `command`) are flat, not nested.
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
| Config | TOML (`~/.config/cortado/feeds.toml`) |
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

## Non-goals (Phase 1)

- External plugin system (WASM, JS, or otherwise).
- Windows/Linux support.
- Advanced notification policies (digesting, scheduling, status-change alerts, channels/actions).
- Persistent storage beyond the config file.
