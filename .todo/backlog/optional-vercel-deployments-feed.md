---
status: pending
---

# Vercel Deployments feed (`vercel`)

## Goal

Add a curated `vercel` feed type that tracks deployment status for a Vercel project. Each activity is a deployment. Uses the Vercel REST API with a personal token (since `reqwest` is already being added for `http-health`).

## Config

```toml
[[feed]]
name = "my deploys"
type = "vercel"
project = "my-project"             # Required: Vercel project name
token_env = "VERCEL_TOKEN"         # Required: env var containing the API token

# Optional
team = "my-team"                   # Vercel team slug (required for team projects)
prod_only = false                  # Only show production deployments (default: false)
```

Token creation: https://vercel.com/account/tokens

## Auth & preflight

API-based auth using a Bearer token read from an environment variable.

Preflight: make a lightweight API call to validate the token:
```
GET https://api.vercel.com/v2/user
Authorization: Bearer {token}
```

If `token_env` is not set or the env var is empty:
- "Vercel feed requires an API token. Create one at https://vercel.com/account/tokens and set the `{token_env}` environment variable."

If 401/403:
- "Vercel feed token is invalid or expired. Create a new token at https://vercel.com/account/tokens."

## Token source pattern

This feed introduces the `token_env` config pattern: read a secret from an environment variable. This is the simplest secure token pattern (no secrets in config files).

```rust
fn resolve_token(env_var: &str) -> Result<String> {
    std::env::var(env_var)
        .map_err(|_| anyhow!("environment variable `{}` is not set", env_var))
        .and_then(|v| {
            if v.is_empty() {
                Err(anyhow!("environment variable `{}` is empty", env_var))
            } else {
                Ok(v)
            }
        })
}
```

Consider placing this in a shared location (e.g., `feed/token.rs` or in `mod.rs`) so future API-backed feeds can reuse it.

## Data source

**Vercel REST API:**

```
GET https://api.vercel.com/v6/deployments?projectId={project}&limit=20&state=BUILDING,ERROR,QUEUED,READY,CANCELED
Authorization: Bearer {token}
```

If `team` is set, add `?teamId={team}` (or use the `team` slug in the URL).
If `prod_only` is true, add `&target=production`.

Response shape (relevant fields):
```json
{
  "deployments": [
    {
      "uid": "dpl_...",
      "name": "my-project",
      "url": "my-project-abc123.vercel.app",
      "state": "READY",
      "target": "production",
      "meta": {
        "githubCommitRef": "main",
        "githubCommitMessage": "fix: typo"
      },
      "created": 1679000000000,
      "inspectorUrl": "https://vercel.com/team/project/dpl_..."
    }
  ]
}
```

## Provided fields

| Field    | Type   | Label  | Description                                |
|---------|--------|--------|--------------------------------------------|
| `state` | status | State  | Deployment state (ready, building, error)  |
| `target`| text   | Target | Environment (production / preview)         |
| `branch`| text   | Branch | Git branch (from meta.githubCommitRef)     |

## Status kind mapping

| Condition          | Value        | StatusKind        |
|--------------------|--------------|-------------------|
| state = ERROR      | `error`      | AttentionNegative |
| state = CANCELED   | `cancelled`  | AttentionNegative |
| state = BUILDING   | `building`   | Running           |
| state = INITIALIZING | `building` | Running           |
| state = QUEUED     | `queued`     | Waiting           |
| state = READY      | `ready`      | Idle              |
| fallback           | `unknown`    | Idle              |

## Activity identity

Inspector URL (e.g., `https://vercel.com/team/project/dpl_...`). Fallback: deployment UID.

## Activity title

Commit message (from `meta.githubCommitMessage`) truncated to ~80 chars. Fallback: `{project} ({target})`.

## Default interval

`60s`

## Acceptance criteria

- [ ] `src-tauri/src/feed/vercel.rs` implements `Feed` trait
- [ ] Config parsing validates `project` and `token_env` are present
- [ ] Token resolved from env var with clear error messages
- [ ] API call to `/v6/deployments` with correct query params
- [ ] Optional `team` and `prod_only` filters applied
- [ ] All deployment states mapped to StatusKind per table above
- [ ] Branch extracted from `meta.githubCommitRef` (gracefully handle missing)
- [ ] Field overrides supported
- [ ] Shared token resolution function usable by future API feeds
- [ ] Registered in `instantiate_feed()` in `mod.rs`
- [ ] Unit tests: config validation, token resolution (set/unset/empty), state mapping (all states), API error handling (401, 500, timeout), field overrides
- [ ] `specs/main.md` updated with `vercel` config example
- [ ] `just check` passes

## Notes

- This is the first API-backed feed. It establishes patterns for future feeds (Linear, Sentry, etc.):
  - `token_env` config pattern
  - `reqwest` client with Bearer auth
  - API error handling (auth failures, rate limits, server errors)
- Uses `reqwest` (added by `http-health` task). No new dependencies needed.
- The Vercel API is clean and well-documented: https://vercel.com/docs/rest-api
- Rate limits: Vercel API allows 500 requests per 60 seconds. At 60s polling interval, this is not a concern.
- The `meta` field in the response contains Git metadata (branch, commit message) but it depends on the Git integration being set up. Handle missing fields gracefully.
- For team projects, the `team` config is required to scope API calls. Without it, the API returns the user's personal projects.

## Relevant files

- `src-tauri/src/feed/vercel.rs` -- new file
- `src-tauri/src/feed/mod.rs` -- register feed type, possibly add shared token helper
- `specs/main.md` -- update config docs
