# AGENTS.md

## Specs

The app spec lives in `specs/main.md`. Read it before starting any work.

UX design patterns, animation tokens, and feedback conventions are in `specs/ux_design.md`. Read it before any UI or UX work — it captures consistency rules (inline save feedback, reset-to-defaults, reduced-motion coverage) that are easy to miss.

The spec is the source of truth. If during implementation you notice the code diverging from the spec (or vice versa), stop and ask the user whether to update the spec or the implementation. Do not silently let them drift apart.

## Terminology

Canonical definitions live in `specs/glossary.md`. Read it before starting any work. Quick reference:

| Term | What it is |
|------|-----------|
| **Feed** | A configured data source (e.g., "GitHub PRs for repo X") |
| **Activity** | One tracked item within a feed (e.g., PR #42) |
| **Field** | A typed data point on an activity (e.g., `review: awaiting`) |
| **Status Kind** | Semantic classification of a status field (e.g., `AttentionNegative`, `Waiting`) — see `specs/status.md` |
| **Status Value** | Feed-specific display text for a status field (e.g., "approved", "failing") |
| **Panel** | The main app window — a floating NSPanel opened by global hotkey (⌘⇧Space). Split layout with list + detail panes. Config section: `[panel]` |
| **Tray** | The menu opened by left-clicking the menubar icon. Shows feeds and activities in a compact list with inline disclosure. |

### Terminology discipline

- Use established terms consistently. Don't introduce synonyms (e.g., don't call a status kind a "severity" or "status type").
- If a concept needs a new term, define it in `specs/glossary.md` first, then use it in code and docs.

## Task Management

Work is organized in **user stories**. Each user story is a directory under `.todo/` named `US-<story-name>/`. Active work lives directly in `.todo/US-<name>/`. Completed work is moved to `.todo/done/US-<name>/`.

Legacy sprints (`sprint01`–`sprint13`) live in `.todo/done/` and follow the old naming convention.

### User story structure

```
.todo/
  backlog/
    optional-some-idea.md  # Ideas and optional work, not tied to a story
  done/
    US-some-story/         # Completed user stories
      main.md
      01-first-task.md
      ...
    sprint01/              # Legacy completed sprints
      ...
  US-active-story/
    main.md                # Story overview: theme, sequencing, cross-task notes
    01-first-task.md       # Individual task
    02-second-task.md      # Individual task
    ...
```

### Backlog

`.todo/backlog/` holds ideas and optional work not yet assigned to a story. Prefix optional items with `optional-`. These can be pulled into a story when relevant.

### User story workflow

1. Before starting a story, read `.todo/US-<name>/main.md` to understand the theme and sequencing.
2. Tasks within a story are numbered for suggested ordering but may be parallelizable — `main.md` clarifies.
3. Each task file describes the goal, acceptance criteria, and relevant files.
4. Mark tasks done by adding `status: done` to the task frontmatter, then move completed story files to `.todo/done/US-<name>/`.
5. Do not skip ahead to the next story without completing (or explicitly deferring) the current one.

### Task file format

```markdown
---
status: pending | in-progress | done | deferred
---

# Task title

## Goal
What this task accomplishes.

## Acceptance criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Notes
Any context, decisions, or gotchas.
```

## Core Principles

### Plan Before You Code
- Understand the task fully before writing code.
- Break complex tasks into smaller steps.
- Use todo lists to track multi-step work.
- If requirements are unclear, ask first.

### Ask, Don't Assume
- When uncertain about requirements, ask the user.
- When multiple approaches exist, present options.
- Don't guess at business logic or user preferences.

### KISS
- Prefer simple solutions over clever ones.
- Avoid premature abstraction.
- If a solution feels complex, step back and reconsider.
- If deviating from simplicity, explain why.

### DRY
- Extract shared logic into reusable functions.
- Don't over-abstract — wait for the Rule of Three.
- If duplicating code intentionally, explain why.

### Single Responsibility
- Each file, struct, class, and function does one thing well.
- Keep files under ~500 lines.
- Prefer composable primitives over monoliths.
- If a struct or file is accumulating multiple responsibilities, split it.

### Performance
- This app must use minimal resources and be extremely responsive. These are non-negotiable.
- If a proposed requirement or change would degrade performance or resource usage, inform the user and warn against it before proceeding.
- Prefer efficient data structures and algorithms. Avoid unnecessary allocations, copies, and blocking operations.
- Profile before optimizing — but never ignore obvious inefficiencies.

### Security
- Never store secrets in code or logs.
- Validate and sanitize all inputs — user-facing and internal boundaries.
- Follow the principle of least privilege.
- When in doubt, choose the more secure option.

### Testing

**Unit tests** — write many, with good coverage:
- Every non-trivial function and struct should have unit tests.
- Tests must be well-isolated — no shared mutable state, no dependency on external services.
- Test edge cases, error paths, and boundary conditions, not just the happy path.

**Integration tests** — write fewer, scoped carefully:
- Integration tests verify that components work together correctly.
- Isolate only the components under test — mock or stub everything else.
- Keep integration tests focused; they should not turn into end-to-end tests.

### Code Comments
- Write doc comments for all public APIs (Rust: `///`, TypeScript: `/** */`).
- Comments should explain *why*, not *what* — the code itself should be readable enough to show *what*.
- Don't over-comment obvious code. Don't under-comment non-obvious decisions.

### Don't Reinvent
- Use established, well-maintained libraries instead of writing your own implementation.
- Always ask the user before introducing a new dependency.
- When evaluating a dependency, consider maintenance status, community adoption, and security track record.

### No Emoji
- Never use emoji characters in code, UI text, or comments.
- Use plain Unicode text symbols for icons (e.g. `⚙`, `◉`, `♪`, `▸`, `⚠`, `✓`, `✕`).
- These render consistently across OS versions and themes, unlike emoji which are colorful bitmaps.

## Code Organization

```
src/                     # Frontend (React + TypeScript)
  App.tsx                # Main UI
  main.tsx               # React entry point
  styles.css             # Panel styles

src-tauri/               # Backend (Rust + Tauri)
  src/
    main.rs              # App entry, Tauri builder
    command.rs            # Tauri commands (invokable from frontend)
    fns.rs               # Menubar panel logic (NSPanel swizzling)
    tray.rs              # Tray icon setup
    feed/                # Feed system (being built)
      mod.rs             # Core types, Feed trait, registry
      config.rs          # TOML config parsing
      github_pr.rs       # GitHub PR feed implementation
      shell.rs           # Shell feed implementation

specs/                   # App specification
  main.md                # Main spec (terminology, architecture, config format)

.todo/                   # Sprint-based task management
  done/
    sprintNN/
      main.md            # Completed sprint overview
      NN-task-name.md    # Completed task
  sprintNN/
    main.md              # Sprint overview
    NN-task-name.md      # Individual tasks
```

## Development

### Prerequisites

See `CONTRIBUTING.md` for prerequisites (Rust, Node.js, pnpm, just).

### Commands

```bash
just              # List all commands
just install      # Install JS deps (pnpm)
just dev          # Run the app locally
just check        # Format + lint + test
just lint         # tsc --noEmit + cargo clippy
just format       # cargo fmt
just test         # cargo test --no-default-features
```

### Verification

Always run `just check` before considering work done. It must pass cleanly (no warnings).

### Package manager

Use `pnpm`, not npm or yarn. The Tauri CLI is a local devDependency — run it via `pnpm exec tauri`, not `pnpm tauri`.

## Code Style

### Rust
- `cargo fmt` for formatting (runs via `just format`).
- `cargo clippy` with `-D warnings` — all warnings are errors.
- The `cargo-clippy` feature in `Cargo.toml` is a workaround for transitive dep warnings — don't remove it.

### TypeScript
- `tsc --noEmit` for type checking.
- No linter configured beyond tsc yet.

## Commit Guidelines

- **Never commit or push unless the user explicitly asks.** Wait for an explicit "commit", "commit and push", or similar instruction.
- Summarize the "why" in 1-2 sentences.
- Use conventional-ish prefixes when natural: `add`, `fix`, `update`, `remove`, `refactor`.
- Don't commit generated files in `src-tauri/gen/schemas/` manually — they're auto-generated by Tauri.

## Changelog

- Entries must be **brief and user-facing**. Write them like release notes for end users.
- Describe what changed from the user's perspective, not how it was implemented.
- **No internal details**: don't mention plugin names (e.g., "Tauri"), pipeline changes (e.g., "CD"), infrastructure, or implementation specifics.
- Good: "Auto-update: the app checks for new versions and lets you install them in one click"
- Bad: "Built-in update feed via tauri-plugin-updater polling latest.json from GitHub Releases"

## PR Finalization

After a PR is merged, clean up local/remote git state in this order:

1. `git switch main`
2. `git pull`
3. `git branch -d <merged-branch>`
4. `git push origin --delete <merged-branch>` (skip if already auto-deleted)

Then verify with `git status --short --branch`.

## Dependencies

- Prefer existing deps over adding new ones.
- If a new dep is needed, it should be well-maintained and necessary.
- Always ask the user before introducing a new dependency.
- Rust deps go in `src-tauri/Cargo.toml`. JS deps via `pnpm add`.

## References

- [awesome-tauri](https://github.com/tauri-apps/awesome-tauri) — curated list of Tauri examples, plugins, and apps. Refer to this when stuck on Tauri-specific issues or looking for implementation patterns.

## Gotchas

### No `block_on` inside Tauri `setup()`

Never use `tauri::async_runtime::block_on()` inside the `setup()` closure. Tauri's setup runs on the main thread within an active tokio runtime. Calling `block_on` from inside a tokio context will deadlock or panic — especially when the awaited future spawns its own tokio tasks (process I/O, timers, etc.). The app will compile fine but silently hang at launch with no tray icon and no visible error.

**Instead**, use `tauri::async_runtime::spawn()` for any async work in setup. If the UI depends on the result (e.g., populating the tray), set up a watch channel or callback so the spawned task can notify the UI when data is ready, rather than blocking the main thread to wait for it.

### Packaged apps get a minimal PATH

macOS apps launched from Finder/Spotlight/Raycast inherit PATH from `launchd`, which only includes `/usr/bin:/bin:/usr/sbin:/sbin`. Tools installed via Homebrew, cargo, pyenv, etc. are missing. This breaks any feed that shells out to a CLI (`az`, `gh`, etc.).

**Fix:** At startup (before any feed polling), resolve the user's login shell PATH with `$SHELL -l -c 'printf "%s" "$PATH"'` and apply it via `std::env::set_var`. This runs in `main()` before Tauri is initialized.

**Do NOT use the `-i` (interactive) flag.** Packaged apps have no TTY, and `-i` causes zsh to fail with "not a terminal". The `-l` (login) flag is sufficient — it sources `/etc/zprofile`, `~/.zprofile`, and `~/.zshrc` on zsh.

**Do NOT use `fix-path-env-rs`.** The Tauri crate `fix-path-env` uses `-ilc` (interactive + login) which fails for the same reason. Our hand-rolled approach with `-l` only works correctly.

### Installing packaged builds over a running app

`cp -R` over a running `.app` bundle **silently fails** to replace the binary — macOS locks the running executable. Always quit the app first, `rm -rf` the old bundle, then copy the new one. The `just build` output goes to `src-tauri/target/release/bundle/`, not `/Applications/`.

## Known Quirks

- `src-tauri/gen/schemas/` files contain "template" and "example" in Tauri's own doc strings — don't try to rename them.
- `tauri-nspanel` and `tauri-toolkit` come from git branches (`v2`), not crates.io.
