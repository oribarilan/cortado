# Cortado

Cortado is a cross-platform extensible watcher that lives in the macOS menubar.

Users configure **feeds** (data sources like "GitHub PRs for repo X"), and each feed automatically discovers and tracks **activities** (e.g., individual PRs). Each activity has structured **fields** showing its current state.

Phase 1 focuses on:

- macOS menubar + panel experience
- Developer-focused workflows
- GitHub PR feed (first curated feed type)
- Shell feed (user-defined commands as an escape hatch)

## Core terms

- **Feed**: a configured data source that discovers and tracks related items
- **Activity**: one tracked item within a feed (e.g., PR #42)
- **Field**: a typed data point on an activity (e.g., `review: awaiting`)

See `specs/main.md` for the full spec.

## Development

### Prerequisites

- [Node.js](https://nodejs.org)
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Commands

```bash
just          # list commands
just install  # install JS deps
just dev      # run app locally
just test     # run Rust tests
just check    # format + lint + test
```

## Feed configuration (`~/.config/cortado/feeds.toml`)

Cortado reads feed config from:

- `~/.config/cortado/feeds.toml`

Config is loaded at startup. If the file changes while Cortado is running, the tray shows a restart-required warning.

### Top-level format

Use one `[[feed]]` table per feed.

```toml
[[feed]]
name = "My feed"
type = "github-pr" # or "shell"
```

### Shared feed keys (all types)

| Key | Required | Type | Notes |
|---|---|---|---|
| `name` | Yes | string | Feed display name; must be unique across all feeds |
| `type` | Yes | string | Supported: `"github-pr"`, `"ado-pr"`, `"shell"` |
| `interval` | No | duration string | Poll interval parsed by `jiff` (examples: `"30s"`, `"5m"`, `"1.5m"`); must be > 0 |
| `retain` | No | duration string | Retain disappeared activities for this long; omitted = no retention |

Duration strings must be strings (not integers). Example: use `"60s"`, not `60`.

Default `interval` values when omitted:

- `github-pr`: `"120s"`
- `ado-pr`: `"120s"`
- `shell`: `"30s"`

Retention is currently in-memory only (retained activities are cleared on app restart).

Each feed is capped to at most **20 activities** after retention and ordering are applied.

### `github-pr` feed

Required keys:

- `repo` (string): target repo in `owner/repo` form (example: `"oribarilan/cortado"`)

Optional type-specific keys:

- `user` (string, default `"@me"`): GitHub author filter (login like `"octocat"` or `"@me"`)

Example:

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "oribarilan/cortado"
user = "@me"
interval = "30s"
retain = "2h"
```

### `shell` feed

Required keys:

- `command` (string): shell command executed via `sh -c`

Optional type-specific keys:

- `field_name` (string, default `"output"`): output field key
- `field_type` (string, default `"text"`): one of `text`, `status`, `number`, `url`
- `label` (string): custom display label for the shell output field

Example:

```toml
[[feed]]
name = "Disk usage"
type = "shell"
command = "df -h /"
interval = "15s"
field_name = "output"
field_type = "text"
label = "Output"
```

### `ado-pr` feed

Required keys:

- `org` (string): full Azure DevOps org URL (example: `"https://dev.azure.com/my-org"`)
- `project` (string): Azure DevOps project name
- `repo` (string): repository name

Optional type-specific keys:

- `user` (string, default `"me"`): PR creator filter; prefer email/UPN, supports `"me"`

Behavior:

- Polls active PRs via `az repos pr list`.
- Requires:
  - `az` CLI installed
  - `azure-devops` extension installed
  - authenticated `az login` session

Example:

```toml
[[feed]]
name = "ADO PRs"
type = "ado-pr"
org = "https://dev.azure.com/my-org"
project = "my-project"
repo = "my-repo"
user = "me"
interval = "120s"
retain = "2h"
```

### Current custom-feed primitive

Today, Cortado's custom-feed escape hatch is the `shell` feed.

What it gives you:

- run any shell command on an interval
- map command output into one typed field (`text`, `status`, `number`, `url`)
- apply shared retention and field overrides

Current limits:

- one activity per shell feed poll
- single primary output field model
- no built-in JSON/object extraction pipeline yet (use command-line tools to shape output)

### Field overrides (optional)

Override visibility/label for any provided field using:

```toml
[feed.fields.<field_name>]
visible = false # optional boolean
label = "Custom label" # optional string
```

Example:

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "oribarilan/cortado"

[feed.fields.labels]
visible = false
label = "Tags"
```

## License

MIT. See [LICENSE.md](./LICENSE.md).
