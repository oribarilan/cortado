<p align="center">
  <img src="art/cortado-128.png" width="96" alt="Cortado icon" />
</p>

<h1 align="center">Cortado</h1>

<p align="center">
  <em>A feed for the busy builder</em>
</p>

<p align="center">
  <a href="https://github.com/oribarilan/cortado/actions/workflows/ci.yml">
    <img src="https://github.com/oribarilan/cortado/actions/workflows/ci.yml/badge.svg" alt="CI" />
  </a>
  &nbsp;&nbsp;
  <a href="https://github.com/oribarilan/cortado/releases/latest">
    <img src="https://img.shields.io/badge/Download_for_macOS-blue?style=for-the-badge&logo=apple&logoColor=white" alt="Download for macOS" />
  </a>
</p>

Cortado is a menubar companion that keeps you in the know. Configure **feeds** — GitHub PRs, CI runs, health checks, shell commands — and glance at their status without leaving your flow.

## Feeds

Cortado ships with curated feed types for common developer workflows:

| Feed type | What it tracks |
|-----------|---------------|
| `github-pr` | Open pull requests with review status, checks, and mergeability |
| `github-actions` | CI/CD workflow runs |
| `ado-pr` | Azure DevOps pull requests |
| `http-health` | Endpoint availability and response time |
| `shell` | Any shell command — the escape hatch for custom data sources |
| `copilot-session` | Active AI coding agent sessions (see below) |

All feeds are configured in `~/.config/cortado/feeds.toml`. See [Feed configuration](#feed-configuration-configcortadofeedstoml) for details.

### Coding agent sessions

The `copilot-session` feed tracks active GitHub Copilot CLI sessions as activities. It reads local session state files — no CLI dependency, no network calls.

Each session shows:

- **Status** — working, idle, waiting for your input (question/approval)
- **Summary** — what the agent is working on
- **Last active** — how recently the session had activity

#### Terminal focus

Opening a copilot-session activity focuses the terminal containing that session. Cortado detects your terminal and picks the best focus strategy automatically:

| Setup | What happens |
|-------|-------------|
| **tmux** (any terminal) | Switches to the exact pane — even reattaches detached sessions. Best experience. |
| **Ghostty** 1.3+ | Switches to the correct tab via AppleScript. |
| **Terminal.app** | Tab focus via TTY matching (planned). |
| **iTerm2** | Session focus via TTY matching (planned). |
| **kitty** | Window focus via remote control (planned). |
| **WezTerm** | Pane focus via CLI (planned). |
| **Other terminals** | Brings the app to front. |

tmux works with any terminal and gives pane-level precision. Without tmux, terminal-specific strategies provide tab-level focus. See `specs/terminal_integration.md` for the full architecture.

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
| `type` | Yes | string | Supported: `"github-pr"`, `"github-actions"`, `"ado-pr"`, `"http-health"`, `"shell"`, `"copilot-session"` |
| `interval` | No | duration string | Poll interval parsed by `jiff` (examples: `"30s"`, `"5m"`, `"1.5m"`); must be > 0 |
| `retain` | No | duration string | Retain disappeared activities for this long; omitted = no retention |

Duration strings must be strings (not integers). Example: use `"60s"`, not `60`.

Default `interval` values when omitted:

- `github-pr`: `"120s"`
- `github-actions`: `"120s"`
- `ado-pr`: `"120s"`
- `http-health`: `"60s"`
- `shell`: `"30s"`
- `copilot-session`: `"30s"`

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

### `copilot-session` feed

Tracks active GitHub Copilot CLI sessions. No required keys — it discovers sessions automatically from `~/.copilot/session-state/`.

Example:

```toml
[[feed]]
name = "copilot sessions"
type = "copilot-session"
```

See `specs/feeds.md` for the full harness architecture and terminal focus documentation.

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
