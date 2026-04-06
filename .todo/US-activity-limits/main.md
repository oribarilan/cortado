# US: Configurable Activity Limits

## Theme

Make the per-feed activity cap configurable instead of a hard-coded constant, and ensure each feed type has a well-defined strategy for picking the "top N" activities.

## Current state

Every feed type hard-codes `MAX_ACTIVITIES_PER_FEED = 20`. Limiting happens at two levels:
1. **CLI query level**: feeds pass `--limit 20` / `--top 20` to their CLI tool, so the API itself only returns ~20 results.
2. **Post-fetch truncation**: after parsing, results are `.take(MAX_ACTIVITIES_PER_FEED)` or `.truncate(MAX_ACTIVITIES_PER_FEED)` as a safety net.

There's also a runtime-level cap in `feed/runtime.rs` (line 304) that truncates to 20 regardless of feed type.

## Sequencing

Task 01 adds the config plumbing, backend support, and settings UI. Task 02 audits and documents each feed's ordering strategy. They can be done in parallel -- 01 is config/UI plumbing, 02 is an audit that may surface ordering fixes.
