---
status: pending
---

# 15 — Simplify ADO PR Feed Config to Single URL

## Goal

Replace the three separate ADO PR config fields (`org`, `project`, `repo`) with a single `url` field that accepts a full Azure DevOps repository URL and parses out the components.

## Acceptance Criteria

- [ ] ADO PR feed config accepts a single `url` field instead of `org`, `project`, `repo`
- [ ] URL format: `https://dev.azure.com/{org}/{project}/_git/{repo}` (the standard Azure DevOps repo URL)
- [ ] Rust parsing extracts `org_url`, `project`, and `repo` from the URL
- [ ] Clear error message if the URL doesn't match the expected pattern
- [ ] The `az` CLI invocations still receive the correct individual arguments
- [ ] Activity IDs (URLs) are still constructed correctly
- [ ] Settings UI updated: single "Repository URL" field with placeholder and hint
- [ ] Existing TOML configs using `org` + `project` + `repo` stop working (breaking change) — document migration in error message
- [ ] Unit tests updated for the new config shape
- [ ] `specs/main.md` config examples updated

## Notes

- The standard Azure DevOps repo URL is: `https://dev.azure.com/{org}/{project}/_git/{repo}`
- On-prem / Azure DevOps Server may use a different URL pattern (e.g., `https://tfs.company.com/tfs/{collection}/{project}/_git/{repo}`). Since we're replacing entirely, this would break on-prem users with non-standard patterns. However, the existing `org` field already requires `https://` and is passed as `--organization`, so on-prem was already somewhat supported. The URL parser should handle both patterns:
  - `https://dev.azure.com/{org}/{project}/_git/{repo}` → org = `https://dev.azure.com/{org}`
  - `https://{host}/{path}/{project}/_git/{repo}` → org = everything before `/{project}/_git/{repo}` 
  - Key: find `_git` in the path and work backwards.
- The `user` field remains separate (optional, defaults to `"me"`).
