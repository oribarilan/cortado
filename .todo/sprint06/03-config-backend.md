---
status: pending
---

# 03 — Config backend commands

## Goal

Expose Tauri commands that let the settings frontend read, validate, and write the `feeds.toml` config file. Reuse the existing config parser for validation.

## Acceptance criteria

- [ ] New Tauri command `get_feeds_config` returns the parsed config as structured JSON (list of feed configs with all fields)
- [ ] New Tauri command `save_feeds_config` accepts structured JSON, validates it, backs up the existing file, and writes the new config
- [ ] Validation reuses the existing config parser logic; invalid configs return descriptive errors to the frontend
- [ ] Before overwriting, the existing `feeds.toml` is copied to `feeds.toml.bak`
- [ ] If `feeds.toml` doesn't exist yet, `save_feeds_config` creates it (and its parent directory if needed)
- [ ] After a successful save, the config-change tracker detects the change (triggering the existing restart-needed flow)
- [ ] A serde-serializable struct represents a feed config entry for the frontend (name, type, interval, retain, type-specific fields, field overrides)
- [ ] Duration fields (`interval`, `retain`) are serialized as duration strings (e.g. `"5m"`) for the frontend and parsed back on save
- [ ] New Tauri command `get_config_path` returns the config file path as a string for display in the UI
- [ ] New Tauri command `open_config_file` opens `feeds.toml` in the default text editor (via `open` on macOS)
- [ ] New Tauri command `reveal_config_file` reveals `feeds.toml` in Finder (via `open -R` on macOS)
- [ ] `just check` passes cleanly

## Notes

### Data shape for frontend

```typescript
interface FeedConfig {
  name: string;
  type: string;
  interval?: string;    // duration string like "5m"
  retain?: string;      // duration string like "1h"
  fields?: Record<string, FieldOverride>;
  // type-specific keys as a flat record
  [key: string]: unknown;
}

interface FieldOverride {
  visible?: boolean;
  label?: string;
}
```

### Rust implementation approach

- `get_feeds_config`: Read the raw TOML file, parse it with the existing parser, then serialize the `FeedConfig` structs to JSON. If the file doesn't exist, return an empty array. Duration fields (`interval`, `retain`) are stored as `Option<Duration>` in Rust but must be serialized as duration strings (e.g. `"5m"`, `"30s"`) for the frontend.
- `save_feeds_config`: Accept a `Vec<FeedConfig>`, parse duration strings back to `Duration`, reconstruct a TOML document, validate by running it through the parser, back up the old file, then write.
- `get_config_path`: Return `feeds_config_path()?.display().to_string()`.
- `open_config_file`: Run `open <config_path>` to open in default text editor. Create the file (and parent dir) if it doesn't exist yet.
- `reveal_config_file`: Run `open -R <config_path>` to reveal in Finder.
- Validation should return user-friendly error messages (e.g., "Feed 'my-feed': interval must be a positive duration like '5m'").

### TOML reconstruction

Use `toml::Value` to build the TOML output (Option C — clean rewrite). The goal is to produce clean, readable TOML — not a direct serde dump. Comments and formatting from hand-edits will be lost; the old file is backed up as `feeds.toml.bak`. The output should look like hand-written config:

```toml
[[feed]]
name = "My PRs"
type = "github-pr"
repo = "owner/repo"
interval = "5m"

[feed.fields.review]
label = "Review Status"
```

### Config file location

Reuse the existing `config_path()` function from `feed/config.rs`.
