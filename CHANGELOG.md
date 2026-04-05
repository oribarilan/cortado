# Changelog

All notable changes to Cortado are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added
- Config change detection: edits to feeds.toml or settings.toml are detected automatically and a one-click restart prompt appears
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
- Signed and notarized DMG — no more Gatekeeper warnings
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
