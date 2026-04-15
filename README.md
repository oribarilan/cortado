<p align="center">
  <img src="art/cortado-readme.png" width="96" alt="Cortado icon" />
</p>

<h1 align="center">Cortado</h1>

<p align="center">
  <em>Attention, spent wisely.</em>
</p>

<p align="center">
  <a href="https://github.com/oribarilan/cortado/actions/workflows/ci.yml">
    <img src="https://github.com/oribarilan/cortado/actions/workflows/ci.yml/badge.svg" alt="CI" />
  </a>
  &nbsp;&nbsp;
  <a href="https://github.com/oribarilan/cortado/releases/latest">
    <img src="https://img.shields.io/badge/Download_for_macOS-blue?style=for-the-badge&logo=apple&logoColor=white" alt="Download for macOS" />
  </a>
  &nbsp;&nbsp;
  <img src="https://img.shields.io/badge/Windows-coming_soon-gray?style=for-the-badge&logo=windows&logoColor=white" alt="Windows - coming soon" />
</p>

https://github.com/user-attachments/assets/08a2f0e5-51af-4e53-b277-6cb2d020abc1

Cortado is a lightweight macOS app that tracks your PRs, CI runs, and services in one place. Set up **feeds** for the things you care about and glance at their status without leaving your flow.

> **Early beta**: You'll hit bugs. [Open an issue](https://github.com/oribarilan/cortado/issues/new) when you do and we'll fix it fast.

<p align="center">
  <img src="https://github.com/user-attachments/assets/d17302ec-af00-4b4d-aae7-cd4f28ad95b2" width="650" alt="Cortado panel" />
</p>

- **Menubar tray + floating panel**: quick-glance from the tray icon, or open the full panel with a global hotkey (**Cmd+Shift+Space**)
- **Multiple feed types**: GitHub PRs, GitHub Actions, Azure DevOps PRs, HTTP health checks, Copilot, OpenCode, and Claude Code coding sessions
- **Lightweight**: minimal CPU and memory footprint
- **Auto-updates**: checks for new versions and lets you install in one click
- **Text-based config**: everything is plain TOML under `~/.config/cortado/` (or `$XDG_CONFIG_HOME/cortado/`), editable through the built-in settings UI or by hand

## Install

Download the latest `.dmg` from [Releases](https://github.com/oribarilan/cortado/releases/latest), open it, and drag Cortado to your Applications folder.

## Getting started

Launch Cortado and open **Settings** (click the gear icon or press **Cmd+,**). Add your first feed from there, no manual file editing required.

Your feeds appear in the menubar tray and the main panel (toggle with **Cmd+Shift+Space**).

## Feeds

A **feed** is a configured data source that discovers and tracks related items. Each feed polls its source on an interval and surfaces individual **activities**: for example, a `github-pr` feed for a repo will show each open PR as a separate activity with review status, checks, and more.

| Feed type              | What it tracks                                                        |
| ---------------------- | --------------------------------------------------------------------- |
| `github-pr`            | Open pull requests with review status, checks, and mergeability       |
| `github-actions`       | CI/CD workflow runs                                                   |
| `ado-pr`               | Azure DevOps pull requests                                            |
| `http-health`          | Endpoint availability and response time                               |
| `copilot-session`      | Active GitHub Copilot coding agent sessions (requires Cortado plugin) |
| `opencode-session`     | Active OpenCode coding sessions                                       |
| `claude-code-session`  | Active Claude Code coding sessions                                    |

### Terminal focus

Coding agent feeds support **one-click terminal focus**: click a session activity to jump to the exact terminal tab and pane running that agent. Works with Ghostty, iTerm2, Terminal.app, WezTerm, and kitty. tmux is supported and gives pane-level precision with any terminal.

<details>
<summary><strong>Configuration reference</strong></summary>

Feeds are configured through the settings UI, but under the hood everything is stored as TOML in `~/.config/cortado/feeds.toml`. You can edit this file directly if you prefer. If `$XDG_CONFIG_HOME` is set, cortado uses `$XDG_CONFIG_HOME/cortado/` instead.

Each feed is a `[[feed]]` block.

### Shared keys

| Key        | Required | Default | Description                                   |
| ---------- | -------- | ------- | --------------------------------------------- |
| `name`     | Yes      |         | Display name (must be unique)                 |
| `type`     | Yes      |         | One of the feed types above                   |
| `interval` | No       | varies  | Poll interval (e.g., `"30s"`, `"5m"`)         |
| `retain`   | No       | off     | Keep disappeared activities for this duration |

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

### `copilot-session`

Tracks active Copilot CLI sessions. The Cortado plugin for Copilot CLI must be installed -- Cortado offers a one-click install button when you add this feed type.

```toml
[[feed]]
name = "Copilot"
type = "copilot-session"
```

### `opencode-session`

Tracks active OpenCode sessions. The `cortado-opencode` plugin must be installed in OpenCode -- Cortado offers a one-click install button when you add this feed type.

```toml
[[feed]]
name = "OpenCode"
type = "opencode-session"
```

### `claude-code-session`

Tracks active Claude Code sessions. The Cortado hook for Claude Code must be installed -- Cortado offers a one-click install button when you add this feed type.

```toml
[[feed]]
name = "Claude Code"
type = "claude-code-session"
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

</details>

## Privacy

Cortado collects **no telemetry, analytics, or usage data**. All data stays on your machine. When local logging is enabled (the default), the app writes log files to `~/Library/Logs/Cortado/` for troubleshooting -- you can disable this in Settings. If you report a bug, you may choose to attach logs to help diagnose the issue; this is entirely optional and up to you.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup, build commands, and release process.

## License

MIT. See [LICENSE.md](./LICENSE.md).
