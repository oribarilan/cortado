# Handoff: general-purpose Copilot usage feed

## Current state

Repository: `/Users/orbarila/repos/personal/cortado`.

Research and implementation are complete. Read-only authenticated probes succeeded for both locally configured GitHub accounts, and the finished feed passed its focused tests, live read-only test, and full project checks.

The user wants a generic Copilot consumption feed for ordinary employees, with explicit GitHub account selection, fresh data, and an attention threshold expressed as a percentage of a dollar amount. Do not assume billing-admin access or special-case the user's employers.

Confirmed product decisions:

- The feed may depend on the `gh` CLI and require the selected account to be authenticated there.
- Use of the undocumented account quota endpoint is acceptable if the feed is clearly labelled experimental.
- Initial host support is `github.com` only. GHE.com is deferred.
- The Activity action should default to GitHub's Copilot settings page and support an optional HTTPS override for company-specific consumption pages.

The user approved the nominal USD threshold semantics and `details_url` override described below.

## Implemented direction

Explore an experimental "Copilot Usage" feed backed by the account-level quota endpoint used by VS Code:

```http
GET https://api.github.com/copilot_internal/user
```

This is an undocumented API, although first-party clients use it. It is a better candidate for ordinary employees than organization billing APIs. Read-only probes succeeded with standard `gh` OAuth credentials for both tested enterprise-plan accounts.

Distinguish individual quota consumption, nominal USD equivalent, billed charges, and Microsoft internal consumption. They are not interchangeable.

## Account quota findings

Current VS Code source declares these fields. Authenticated probes confirmed the relevant fields on both tested accounts:

```text
copilot_plan
access_type_sku
organization_login_list
token_based_billing
quota_reset_date
quota_reset_date_utc

quota_snapshots.{chat,completions,premium_interactions}:
  entitlement
  quota_remaining
  credits_used
  percent_remaining
  unlimited
  has_quota
  quota_reset_at
  overage_count
  overage_entitlement
  overage_permitted
```

- Both tested responses identified the selected login and returned token-based billing data with a finite `premium_interactions` entitlement, `credits_used`, `quota_remaining`, and `percent_remaining`.
- `chat` and `completions` appeared as unlimited in both tested responses while `premium_interactions` was finite. Do not treat an unlimited category as zero usage or apply one category's denominator to another.
- Use `credits_used` directly for individual consumption. In one live response it did not equal `entitlement - quota_remaining`, so subtraction is not a reliable total-consumption calculation.
- Without a useful denominator, `credits_used` can still supply individual consumption. A configured nominal USD reference amount would make percentage thresholds work for unlimited pools.
- `has_quota` semantics are not stable enough to drive health or threshold state. Live token-based responses returned `true`, while a current VS Code source comment says it is always false under token-based billing.
- `token_based_billing` distinguishes AI credits from legacy premium-request units. Never apply a credit-to-dollar conversion to legacy request counts.
- GitHub currently publishes one AI credit as $0.01 USD. Recheck this before implementation. This is a nominal equivalent, not evidence of actual invoice charges or internal cost.
- The live response included no source-data timestamp. Its HTTP response advertised `Cache-Control: private, max-age=60, s-maxage=60`, but no upstream freshness SLA was established. Do not assume the Microsoft portal's `refresh=1` parameter applies.

Sources to recheck against current code:

- Individual usage documentation: https://docs.github.com/en/copilot/how-tos/manage-and-track-spending/monitor-ai-usage
- Response types: https://github.com/microsoft/vscode/blob/main/src/vs/base/common/defaultAccount.ts
- Parsing: https://github.com/microsoft/vscode/blob/main/src/vs/workbench/services/chat/common/chatEntitlementService.ts
- Quota requests: https://github.com/microsoft/vscode/blob/main/extensions/copilot/src/platform/chat/common/chatQuotaServiceImpl.ts
- Direct endpoint/auth handling: https://github.com/microsoft/vscode/blob/main/src/vs/platform/agentHost/node/shared/copilotApiService.ts
- Host derivation: https://github.com/microsoft/vscode/blob/main/src/vs/platform/agentHost/common/githubEndpoints.ts

Research found `api.github.com` for github.com and `api.<tenant>.ghe.com` for tenant.ghe.com. GHES routing was explicitly unverified in first-party source. Do not promise all enterprise hosts work.

## Authentication findings and generic contract

Account-specific read-only probes succeeded with this flow:

1. Resolve the selected account's token with `gh auth token --hostname github.com --user <login>`.
2. Pass that token only through the child request's environment.
3. Query `/copilot_internal/user` without changing the globally active gh account.
4. Verify the response's `login` matches the configured account. If `login` is ever absent, call `/user` with the same token and environment.

The configured account must already be authenticated in `gh`. Clear inherited `GH_TOKEN` and `GITHUB_TOKEN` before token resolution so they cannot silently select a different account. Never print tokens or auth headers, include tokens in command displays, or copy browser cookies into the repository. Do not use `gh auth switch`; feeds for different accounts must poll safely in parallel.

The existing process abstraction has no per-command environment overrides. Any implementation must add a secret-safe mechanism that prevents environment values from appearing in `Debug`, display strings, and errors.

At probe time, GitHub's public `/user` route rejected the formerly supplied `2025-04-01` API version and listed newer supported versions. The internal quota route accepted both the current version and no explicit version. Recheck version handling before implementation rather than freezing the old probe header.

GitHub documents Copilot CLI support for gh OAuth credentials, and the tested gh OAuth credentials worked with this endpoint. This does not guarantee compatibility with every credential type. Classic PATs are not interchangeable with OAuth credentials. Treat 401/403 responses as evidence to investigate, not a reason to request broader permissions immediately.

Source: https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli

## Supported billing APIs: a different access model

The official personal endpoint is:

```text
GET /users/{username}/settings/billing/ai_credit/usage
```

The documentation explicitly excludes organization/enterprise-paid usage from user-level billing responses. It documents fine-grained PAT or GitHub App user authentication with Plan read permission. Normal gh OAuth compatibility was not tested.

Organization and enterprise routes can filter usage by user but require elevated billing access:

```text
GET /organizations/{org}/settings/billing/ai_credit/usage?user={login}
GET /enterprises/{enterprise}/settings/billing/ai_credit/usage?user={login}
```

These are candidates for a supported administrator-facing billing integration, not a universal employee feed. Recheck exact roles and token requirements before implementation.

Source: https://docs.github.com/en/rest/billing/usage

## Microsoft portal findings

Original page: https://copilot.github.microsoft.com/?section=limits&tab=consumption

The public backend/framework is https://github.com/microsoft/opensource-management-portal. Its company-specific extensions and separate React frontend are not included in that repository. Deployed frontend bundles were publicly readable.

Verified from deployed JavaScript:

```text
GET /api/client/context/copilot/quota
GET /api/client/context/copilot/quota?refresh=1
```

The endpoint returns associated accounts; the UI reads `results[].login` and `results[].enterprise`. An optional `/quota/{alias}` path selects a corporate alias, not a GitHub login.

The frontend accesses:

```text
dataFetchedAt
aggregate.consumedAmountUsd
user.aadAlias
results[].ok
results[].login
results[].enterprise
results[].utilization.consumedAmountUsd
results[].utilization.budgetAmountUsd
results[].utilization.remainingAmountUsd
results[].utilization.utilizationPercent
```

These property accesses do not establish a complete schema, requiredness, or nullability. Neither account's authenticated response was inspected.

The page calls the values "Month-to-date internal USD consumption across Microsoft-provided GitHub enterprises" and the denominator "Monthly limit (not a budget)". Do not label these as personal billed spend.

The normal client query has a two-minute stale window and no automatic refetch on mount or focus. Manual refresh calls `?refresh=1`. During the first 36 hours of a UTC month, the consumption page refreshes every five minutes using that parameter. It displays `dataFetchedAt` as "Data retrieved", or reports unavailable freshness. Backend cache duration and upstream accounting delay remain unknown.

Authentication uses Microsoft Entra sign-in and a portal session. A gh token is not a substitute. Anonymous API requests returned HTTP 401. This route is Microsoft-specific and should not be the basis of a general-purpose employee feed.

Parent directly verified HTTP 200 and relevant source snippets from these deployment-specific assets:

- https://copilot.github.microsoft.com/assets/microsoft-D5lXrY9Y.js
- https://copilot.github.microsoft.com/assets/quotaSection-Bh-P21bi.js

Asset hashes changed during research. Old asset URLs redirected to sign-in while a fetch cache replayed earlier JavaScript. Rediscover current assets through `/index.html` rather than treating old hashes or cached responses as current evidence.

No browser MCP, installed Playwright package/CLI, or CDP connection on localhost:9222 was available. No authenticated browser inspection occurred.

## Cortado integration context

Read `AGENTS.md`, `specs/main.md`, `specs/glossary.md`, and, for UI work, `specs/ux_design.md`. They hold the existing contracts; this handoff is not a replacement spec.

Relevant paths:

- `src-tauri/src/feed/mod.rs`: Feed trait, fields, factory, network feed registration.
- `src-tauri/src/feed/config.rs`: flat type-specific TOML settings.
- `src-tauri/src/feed/github_common.rs`: existing gh checks, currently not account-specific.
- `src-tauri/src/feed/process.rs`: process abstraction lacked per-command environment overrides during inspection.
- `src-tauri/src/feed/runtime.rs`: polling, cached activities, errors, successful-poll timestamp.
- `src-tauri/src/notification/change_detection.rs`: notifications follow status-kind changes, not numeric increments.
- `src/shared/feedTypes.ts`: catalog and feed-form metadata.
- `src/App.tsx`: Tray.
- `src/main-screen/MainScreenApp.tsx`: Panel and activity details.
- `src/settings/SettingsApp.tsx`: feed forms and connection testing.

Existing GitHub `user` settings filter authors or actors; they do not select credentials. New account selection must not reuse that behavior accidentally.

The implementation uses one feed per account, with one stable Activity and existing formatted text/status fields. A chart is unnecessary initially. It uses `AttentionNegative` when the configured threshold is reached and `Idle` below it. Numeric changes alone do not notify, and crossing 100% after an earlier attention threshold does not produce another status-kind change.

Approved threshold config:

```toml
reference_amount_usd = 2000
attention_at_percent = 80
```

Calculate nominal usage as `credits_used * $0.01` and compare it with the configured reference amount. Require `reference_amount_usd`; default `attention_at_percent` to 80. Label the result "nominal usage," not spend, cost, billed charges, budget, or internal consumption. If `token_based_billing` is false or required usage data is absent, show an actionable error instead of calculating a dollar value or reporting a healthy status.

The Activity action should default to `https://github.com/settings/copilot`. Add an optional config field for an override:

```toml
details_url = "https://copilot.github.microsoft.com/?section=limits&tab=consumption"
```

Accept validated HTTPS URLs only. `src/shared/feedTypes.ts` already supports feed-type `notes` rendered in the Settings feed form. Add a note such as: "Links open with your browser's active GitHub session, which may differ from the GitHub account selected above." The `details_url` field hint should explain that it defaults to GitHub Copilot settings and can point to a company-specific consumption page.

Distinguish last successful check from source data freshness. Missing data must not become zero or healthy. Existing cached-error rendering differs between Tray and Panel and needs inspection before relying on stale-value presentation.

Research noted existing spec/code differences: the spec lists a URL field type absent from the backend enum, and its attention-section wording/filter differs from the current UI. Before changing affected behavior, ask whether the spec or implementation should be corrected. Do not fix unrelated drift silently.

## Maintenance notes

- Keep the feed labelled experimental while it depends on `/copilot_internal/user`.
- Recheck the endpoint schema, supported API versions, and GitHub's published AI credit conversion when maintaining the feed.
- Keep the response parser allowlisted. The raw endpoint response contains unrelated account and service metadata that must not be retained or logged.
- Preserve validation for zero reference amounts, missing or non-finite values, over-limit usage, legacy units, login mismatches, and action URLs with embedded credentials.
- Compare against the Microsoft portal only if an authenticated portal response becomes available. Do not promise equivalent dollar values.

No dependency was added. No commit or push was requested.

## Suggested skills

- `brainstorming` and `working-with-users-and-team` before resolving remaining product choices.
- `security-and-trust-boundaries` for credential handling and authenticated probes.
- `using-97` before coding, then its matching skills.
- `tasks` and `writing-plans` after design approval.
- `verification-planning`, `test-driven-development`, and `testing-discipline` before implementation.
- `verification-before-completion` and `pre-commit-self-review` before declaring implementation complete.

Use a librarian for future API/source research, a designer for UI changes, and a fixer for bounded backend maintenance. Start from the completed authenticated probes and implementation; do not repeat broad feasibility research or present tentative proposals as decisions already made.
