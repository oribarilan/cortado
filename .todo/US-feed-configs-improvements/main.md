# US-feed-configs-improvements

## Goal

Streamline GitHub feed configuration for the common case and simplify the retain feature. Today, adding a GitHub feed requires manually typing `owner/repo` — even when the user just wants to watch repos they already contribute to. The retain feature's optional duration input is also more complex than it needs to be.

## Definition of Done

- [ ] GitHub feed creation offers a repo picker that lists repos the user contributes to, with a fallback "any repo" option for manual entry
- [ ] User filter offers a similar picker pattern (select from org members or known collaborators) where feasible
- [ ] Retain is simplified to a toggle + conditional duration input
- [ ] All changes pass `just check`

## Task Priority

1. `repo-picker.md` — Highest user impact; repo selection is the most common friction point
2. `user-picker.md` — Same pattern as repo picker, natural follow-on
3. `retain-simplify.md` — Independent of the above; smallest scope

## Cross-Cutting Concerns

- All GitHub API calls should go through the `gh` CLI (consistent with existing feeds)
- Repo/user lists should be fetched lazily (only when the user opens the picker) and cached for the session
- Error states: `gh` not installed, not authenticated, API rate limits — all need graceful handling
- The "any repo" / manual entry fallback must always be available; the picker is a convenience, not a gate
- The repo picker component is shared between GitHub PR and GitHub Actions feed types
- `user-picker` is optional/stretch — may be deferred if API constraints make it impractical
