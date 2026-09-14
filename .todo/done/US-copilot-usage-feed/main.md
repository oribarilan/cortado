# US: Experimental Copilot usage feed

## Theme

Add a generic, account-specific GitHub Copilot usage feed for ordinary GitHub users. The feed reads AI credit consumption through the undocumented endpoint used by first-party clients, compares nominal usage with a user-configured reference amount, and opens either GitHub Copilot settings or an optional company consumption page.

## Decisions

- Feed type: `copilot-usage`, clearly labelled experimental.
- Initial host support: `github.com` only.
- Authentication: selected `account` must already be authenticated in the `gh` CLI; Cortado must not switch the active gh account.
- One stable Activity per configured account.
- Nominal usage is `credits_used * $0.01`; it is not billed spend or internal cost.
- `reference_amount_usd` is required and positive.
- `attention_at_percent` defaults to 80 and accepts 1 through 100.
- The Activity is `Idle` below the threshold and `AttentionNegative` at or above it.
- The Activity opens `https://github.com/settings/copilot` by default. `details_url` may override it with an HTTPS URL that has no embedded credentials.
- Settings must note that the browser's active GitHub account can differ from the selected gh account.
- Default interval: 120 seconds.
- No new dependency is required.

## Sequencing

Task 01 records the approved contract in the source-of-truth specs. Task 02 implements the secret-safe process support and backend feed. Task 03 adds the Settings catalog and user documentation. Task 04 performs final verification and closes the story.

## Research

See `research.md` for authenticated probe results, API caveats, and rejected administrator-only alternatives.
