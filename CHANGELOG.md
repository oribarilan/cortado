# Changelog

All notable changes to Cortado are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added
- Terminal focus: cmux support -- clicking an agent activity focuses the exact terminal by working directory

### Changed
- README updated with screenshot and demo video

## [0.10.0] - 2026-04-06

### Added
- Feed filters: author/creator/actor filters now use a 3-option control (All / Me / User) instead of a plain text input
- Feed filters: "All" option shows activities from every author, not just yours
- Feed filters: workflow and branch filters for GitHub Actions now explain that empty means "all"

### Changed
- Feed filters: empty author/creator config now means "no filter" (previously defaulted to current user)

## [0.9.0] - 2026-04-06

### Added
- Settings: "Terminals" tab replaces "Agents" -- each terminal emulator gets its own expandable section showing capabilities, version, and settings
- Terminal detection: the app now detects which terminal emulators are installed on your system
- Config directory respects `$XDG_CONFIG_HOME` when set (default remains `~/.config/cortado/`)
- Terminal icons: each terminal in Settings shows its brand icon (Ghostty, iTerm2, WezTerm, tmux)
- Each terminal's expanded section explains what the integration does for you, not just technical details
- Non-detected terminals link to the official download page ("Get WezTerm", etc.)
- Default feed names: new feeds auto-populate a descriptive name from their type and config (e.g., "owner/repo PRs")
- Last refreshed: each feed shows when it was last polled (e.g., "Updated 3m ago"), ticking live
- Offline detection: when the network is down, affected feeds show "disconnected" and polling pauses until connectivity is restored -- local feeds are unaffected
- Retry connection: a single button in the footer lets you trigger an immediate connectivity check

### Changed
- Panel: arrow keys wrap around when navigating past the first or last activity
- Panel: "Needs Attention" section header restyled to match feed headers, with a subtle accent border
- Copilot and OpenCode feed icons now use the official brand logos
- Terminals sorted by availability: detected first, non-detected dimmed at the bottom
- tmux visually separated under its own "Multiplexer" section header
- "Removed activities" notifications now default to off -- most removals (e.g., merged PRs) are expected
- File-watching feeds no longer show an interval badge in Settings since they update via filesystem events
- Replaced all em dashes with double dashes for consistent rendering across terminals and editors

## [0.8.0] - 2026-04-05

### Added
- Copilot CLI session tracking: see your Copilot CLI sessions with live status, repo, and branch -- install the Cortado plugin with one click in Settings
- Question detection for Copilot: when Copilot asks you a question, the session shows an attention indicator (same as OpenCode)
- Plugin uninstall: safely remove the Cortado plugin from Copilot CLI or OpenCode via Settings
- Empty feeds are now hidden by default in both the tray and panel; toggle "Show empty feeds" in Settings to reveal them

## [0.7.0] - 2026-04-05

### Added
- Config change detection: edits to feeds.toml or settings.toml are detected automatically and a one-click restart prompt appears in the tray, panel, and Settings sidebar
- Settings save feedback now tells you whether changes applied immediately or require a restart
- "Focus session" works for all coding agent feeds, not just Copilot
- Reorder feeds in Settings: hover over a feed card to reveal up/down arrows that rearrange the list and persist the new order

### Fixed
- "Focus session" now correctly switches to the right terminal tab when using tmux inside Ghostty

## [0.6.0] - 2026-04-04

### Added
- OpenCode session tracking: see your OpenCode coding sessions with status, repo, and branch at a glance
- One-click plugin install and update from Settings
- Plugin update notifications in the Cortado Updates feed
- Near-instant session detection for coding agent feeds via filesystem event watching

### Fixed
- When multiple coding sessions share the same repo, the most actionable status now surfaces (e.g., "question" beats "working")

## [0.5.1] - 2026-04-02

### Added
- Panel height adapts to screen size instead of a fixed 400px

## [0.5.0] - 2026-04-02

### Added
- Built-in update awareness feed: Cortado checks for new versions and surfaces "vX.Y.Z available" as an activity with an "Install update" button
- In-app auto-update (download, verify signature, install, restart)
- Tray icon status indicator: a colored dot in the bottom-right corner reflects the global rollup status across all feeds (red for attention, yellow for waiting, blue for running, green for action needed). When idle, the icon reverts to its native monochrome template.

## [0.4.0] - 2026-03-31

### Added
- Shimmer loading placeholders while feeds are polling for the first time
- Version shown in tray and panel footers
- Branded DMG installer with drag-to-Applications guide
- New app icon

## [0.3.0] - 2026-03-31

### Added
- Signed and notarized DMG, no more Gatekeeper warnings
- CD pipeline: push a version tag to auto-publish a GitHub Release
- Version shown in tray menu
- Panels close when opening Settings (Cmd+,)

### Fixed
- CLI tools (az, gh) not found in packaged app (PATH resolution from login shell)

## [0.2.0] - 2026-03-31

### Added
- Feed system: GitHub PR, GitHub Actions, Azure DevOps PR, HTTP health, shell, Copilot session feeds
- Main screen panel with split layout (list + detail)
- Menubar panel (tray dropdown)
- Settings window with live config editing
- Notification system with configurable delivery modes
- Terminal focus: tmux pane switching, Ghostty tab focus
- Global hotkey (Cmd+Shift+Space) to toggle main screen
- Autostart via launch agent
- Dev/release build isolation (separate bundle ID, config dir)
- Local DMG packaging (`just build`)
- CI pipeline (GitHub Actions)
