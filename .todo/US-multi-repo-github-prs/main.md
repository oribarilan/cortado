# US-multi-repo-github-prs

## Goal

Streamline GitHub feed configuration for the common case and simplify the retain feature. Today, adding a GitHub feed requires manually typing `owner/repo` — even when the user just wants to watch repos they already contribute to. The retain feature's optional duration input is also more complex than it needs to be.

## Definition of Done

- [ ] GitHub feeds support multiple repos per feed (`repos = [...]` config format, backward compat with `repo`)
- [ ] GitHub feed creation offers a repo picker that lists repos the user contributes to, with "Any repo" manual entry
- [ ] Retain is simplified to a toggle + conditional duration input
- [ ] All changes pass `just check`

## Task Priority

1. `retain-simplify.md` — Independent, smallest scope, quick win
2. `multi-repo.md` — Backend model change for multi-repo feeds (required before picker UI)
3. `repo-picker.md` — Frontend repo picker UI (depends on multi-repo backend)

## Cross-Cutting Concerns

- All GitHub API calls should go through the `gh` CLI (consistent with existing feeds)
- Repo/user lists should be fetched lazily (only when the user opens the picker) and cached for the session
- Error states: `gh` not installed, not authenticated, API rate limits — all need graceful handling
- The "any repo" / manual entry fallback must always be available; the picker is a convenience, not a gate
- The repo picker component is shared between GitHub PR and GitHub Actions feed types
- Multi-repo is GitHub-only for now; ADO PR stays single-repo
