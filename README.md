<p align="center">
  <img src="art/cortado-readme.png" width="96" alt="Cortado icon" />
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

Cortado is a lightweight app that keeps you in the know. Configure **feeds**, GitHub PRs, CI runs, health checks, etc, and glance at their status without leaving your flow.

## Install

Download the latest `.dmg` from [Releases](https://github.com/oribarilan/cortado/releases/latest), open it, and drag Cortado to your Applications folder.

## Getting started

Cortado reads feeds from `~/.config/cortado/feeds.toml`. Create the file and add your first feed:

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "owner/repo"
```

Launch Cortado — your feed appears in the menubar tray and the main panel (toggle with **Cmd+Shift+Space**).

## Feed types

| Feed type | What it tracks |
|-----------|---------------|
| `github-pr` | Open pull requests with review status, checks, and mergeability |
| `github-actions` | CI/CD workflow runs |
| `ado-pr` | Azure DevOps pull requests |
| `http-health` | Endpoint availability and response time |
| `shell` | Any shell command — the escape hatch for custom data sources |
| `copilot-session` | Active GitHub Copilot coding agent sessions |

## Configuration

Each feed is a `[[feed]]` block in `~/.config/cortado/feeds.toml`.

### Shared keys

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `name` | Yes | — | Display name (must be unique) |
| `type` | Yes | — | One of the feed types above |
| `interval` | No | varies | Poll interval (e.g., `"30s"`, `"5m"`) |
| `retain` | No | off | Keep disappeared activities for this duration |

### `github-pr`

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "oribarilan/cortado"
user = "@me"          # optional, default "@me"
interval = "120s"
```

### `github-actions`

```toml
[[feed]]
name = "CI"
type = "github-actions"
repo = "oribarilan/cortado"
interval = "120s"
```

### `ado-pr`

Requires `az` CLI with `azure-devops` extension and `az login`.

```toml
[[feed]]
name = "ADO PRs"
type = "ado-pr"
org = "https://dev.azure.com/my-org"
project = "my-project"
repo = "my-repo"
user = "me"
```

### `http-health`

```toml
[[feed]]
name = "API"
type = "http-health"
url = "https://api.example.com/health"
interval = "60s"
```

### `shell`

Run any command. Output maps to a single typed field.

```toml
[[feed]]
name = "Disk usage"
type = "shell"
command = "df -h / | tail -1"
interval = "30s"
```

### `copilot-session`

Discovers active sessions automatically — no config needed beyond the block.

```toml
[[feed]]
name = "Copilot"
type = "copilot-session"
```

### Field overrides

Customize field visibility or labels for any feed:

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "oribarilan/cortado"

[feed.fields.labels]
visible = false
label = "Tags"
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup, build commands, and release process.

## License

MIT. See [LICENSE.md](./LICENSE.md).
