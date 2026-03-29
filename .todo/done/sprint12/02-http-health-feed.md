---
status: pending
---

# HTTP Health Check feed (`http-health`)

## Goal

Add a curated `http-health` feed type that monitors HTTP endpoint availability. Each configured URL becomes an activity. Pure Rust implementation -- no external CLI dependency.

## Config

```toml
[[feed]]
name = "api health"
type = "http-health"
url = "https://api.example.com/health"    # Single URL

# Optional
method = "GET"                             # GET (default) or HEAD
timeout = "5s"                             # Per-request timeout (default: 10s)
expected_status = 200                      # Expected HTTP status code (default: 200)
```

For multiple endpoints, users create separate feeds (one per URL). This is simpler and consistent with the existing model where each feed is independently configured.

## Auth & preflight

No external binary needed. No preflight checks required.

Config-time validation:
- `url` is present and well-formed (parseable as a URL)
- `method` is `GET` or `HEAD` if present
- `timeout` is a valid positive duration if present
- `expected_status` is a valid HTTP status code (100-599) if present

## Data source

Pure Rust HTTP request using `reqwest`.

```rust
let response = client
    .request(method, &url)
    .timeout(timeout)
    .send()
    .await;
```

Measure response time with `std::time::Instant`.

## New dependency

**`reqwest`** with features: `rustls-tls` (not `native-tls` -- avoids OpenSSL dependency on macOS).

```toml
[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
```

This is a significant but well-justified dependency. It will be reused by any future API-backed feeds (Linear, Sentry, Vercel, etc.).

## Provided fields

| Field           | Type   | Label         | Description                       |
|----------------|--------|---------------|-----------------------------------|
| `status`       | status | Status        | Health status (healthy/down/etc)  |
| `response_time`| number | Response Time | Response time in milliseconds     |
| `status_code`  | number | Status Code   | HTTP status code                  |

## Status kind mapping

| Condition                                  | Value       | StatusKind        |
|--------------------------------------------|-------------|-------------------|
| Request failed (timeout, DNS, connect)     | `down`      | AttentionNegative |
| HTTP status != expected_status             | `unhealthy` | AttentionNegative |
| All good                                   | `healthy`   | Idle              |

Keep the mapping simple. No "slow" threshold in the initial implementation -- response_time is surfaced as a number field for the user to see, but doesn't drive status. Can add `slow_threshold` config later if needed.

## Activity identity

The URL itself.

## Activity title

URL hostname + path (e.g., `api.example.com/health`). Strip scheme and trailing slash.

## Default interval

`60s` (health checks are lightweight and benefit from more frequent polling).

## Acceptance criteria

- [ ] `reqwest` added to `Cargo.toml` with `rustls-tls` feature
- [ ] `src-tauri/src/feed/http_health.rs` implements `Feed` trait
- [ ] Config parsing validates `url` is present and well-formed
- [ ] HTTP request uses configured method, timeout, and expected status
- [ ] Response time measured and reported as milliseconds
- [ ] Config defaults applied when optional fields are omitted: method=GET, timeout=10s, expected_status=200
- [ ] `expected_status` validated as 100-599 at config time
- [ ] `method` validated as GET or HEAD at config time
- [ ] Connection/timeout/DNS errors mapped to `down` / AttentionNegative
- [ ] Unexpected status codes mapped to `unhealthy` / AttentionNegative
- [ ] Field overrides supported
- [ ] Registered in `instantiate_feed()` in `mod.rs`
- [ ] Unit tests: config validation, status mapping (healthy, unhealthy, down), response time measurement, field overrides
- [ ] `specs/main.md` updated: replace the existing `http-health` future-feed entry with the final field contract and config example
- [ ] `just check` passes

## Notes

- This is the first feed that doesn't use an external CLI. It establishes the pure-Rust HTTP pattern.
- The `reqwest` client should be created once per feed instance (not per poll). Store it in the feed struct.
- User-Agent header: `cortado/{version}` or just `cortado`.
- For tests: use a mock HTTP server or mock the `reqwest` client. The existing `ProcessRunner` trait pattern (trait-based injection) could be adapted -- e.g., a `HealthChecker` trait with a `TokioHealthChecker` impl and a mock.
- Do NOT add JSON body field checking in this task. Keep it to status code + reachability. JSON health check parsing can be a follow-up.
- Consider: should the feed expose the URL as a `Url`-type field so the panel can link to it? The activity title already shows the URL, and the identity is the URL, so this may be redundant.

## Relevant files

- `src-tauri/Cargo.toml` -- add `reqwest`
- `src-tauri/src/feed/http_health.rs` -- new file
- `src-tauri/src/feed/mod.rs` -- register feed type
- `specs/main.md` -- update config docs
