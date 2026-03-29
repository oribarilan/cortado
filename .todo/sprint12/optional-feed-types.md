---
status: pending
---

# Optional Feed Types -- Feasibility & Design

This document catalogs candidate feed types for cortado. Each entry includes:
config design, auth strategy, data source mechanics, field/status mappings,
and implementation considerations. All are independent -- pick and choose.

Existing curated feeds for reference: `github-pr`, `ado-pr`, `shell`.

---

## Table of Contents

1. [GitHub Actions](#1-github-actions)
2. [GitHub Issues](#2-github-issues)
3. [GitLab Merge Requests](#3-gitlab-merge-requests)
4. [HTTP Health Check](#4-http-health-check)
5. [Docker Containers](#5-docker-containers)
6. [Kubernetes Pods](#6-kubernetes-pods)
7. [Linear Issues](#7-linear-issues)
8. [Jira Issues](#8-jira-issues)
9. [Sentry Issues](#9-sentry-issues)
10. [PagerDuty Incidents](#10-pagerduty-incidents)
11. [Vercel Deployments](#11-vercel-deployments)
12. [RSS/Atom Feed](#12-rssatom-feed)
13. [npm Outdated](#13-npm-outdated)
14. [SSL Certificate Expiry](#14-ssl-certificate-expiry)

---

## 1. GitHub Actions

**Feed type:** `github-actions`

**What it tracks:** Workflow runs for a repository. Each activity is a workflow
run (or the latest run per workflow). Shows CI/CD health at a glance.

### Data source

CLI: `gh run list` / `gh run view` via the `gh` CLI (already a dependency for
`github-pr`).

```sh
gh run list --repo OWNER/REPO --limit 20 --json name,status,conclusion,headBranch,event,url,updatedAt,workflowName
```

Alternatively, for a "latest run per workflow" view:

```sh
gh run list --repo OWNER/REPO --workflow WORKFLOW --limit 1 --json ...
```

### Auth

Same as `github-pr` -- uses `gh auth status`. No new auth mechanism needed.

### Preflight

1. `gh --version` -- binary exists
2. `gh auth status` -- authenticated

Identical to `github-pr` preflight.

### Config

```toml
[[feed]]
name = "my ci"
type = "github-actions"
repo = "owner/repo"

# Optional filters
branch = "main"              # Only runs on this branch
workflow = "ci.yml"          # Only this workflow file
event = "push"               # Filter by trigger event
user = "@me"                 # Only runs triggered by this user
```

### Provided fields

| Field        | Type   | Description                              |
|-------------|--------|------------------------------------------|
| `status`    | status | Run status (completed, in_progress, etc) |
| `branch`    | text   | Head branch                              |
| `workflow`  | text   | Workflow name                            |
| `event`     | text   | Trigger event (push, pull_request, etc)  |
| `duration`  | text   | Run duration                             |

### Status kind mapping

| Condition                                    | Value         | StatusKind          |
|----------------------------------------------|---------------|---------------------|
| conclusion = failure/timed_out/startup_failure | `failing`     | AttentionNegative   |
| conclusion = cancelled                       | `cancelled`   | AttentionNegative   |
| status = in_progress                         | `running`     | Running             |
| status = queued / waiting                    | `queued`      | Waiting             |
| conclusion = success                         | `passing`     | Idle                |
| conclusion = skipped/neutral                 | `skipped`     | Idle                |

### Activity identity

`{repo}/actions/runs/{run_id}` or the URL from the API.

### Activity title

`{workflow_name} #{run_number}` or `{workflow_name} ({branch})` for
latest-per-workflow mode.

### Default interval

`120s` (same as `github-pr` -- these share API rate limits).

### Implementation notes

- Very natural extension of the existing `gh` CLI pattern.
- Could support two modes: "recent runs" (multi-activity, last N runs) vs
  "latest per workflow" (one activity per workflow file). The latter is more
  dashboard-friendly. Consider making this a config option (`mode = "recent"` vs
  `mode = "latest-per-workflow"`).
- Share the `gh` preflight logic with `github-pr` (extract to a shared helper).
- The `gh` CLI handles pagination and auth token management.

### Complexity: Low

Closest to existing code. Same CLI, same auth, same JSON parsing pattern.

---

## 2. GitHub Issues

**Feed type:** `github-issue`

**What it tracks:** Issues assigned to or created by the user in a repository
(or across repos). Each activity is an issue.

### Data source

CLI: `gh issue list` via the `gh` CLI.

```sh
gh issue list --repo OWNER/REPO --assignee @me --state open --limit 20 --json number,title,state,url,labels,milestone,updatedAt,createdAt
```

For cross-repo (all assigned issues):

```sh
gh search issues --assignee @me --state open --limit 20 --json number,title,state,url,labels,repository
```

### Auth

Same as `github-pr` / `github-actions`.

### Preflight

Identical to `github-pr`.

### Config

```toml
[[feed]]
name = "my issues"
type = "github-issue"
repo = "owner/repo"       # Optional -- if omitted, searches across all repos
user = "@me"               # Default: @me
role = "assignee"          # assignee | creator | mentioned
state = "open"             # open | closed | all (default: open)
```

### Provided fields

| Field       | Type   | Description                          |
|------------|--------|--------------------------------------|
| `state`    | status | Issue state (open, closed)           |
| `labels`   | text   | Comma-separated label names          |
| `milestone`| text   | Milestone name                       |
| `repo`     | text   | Repository (for cross-repo mode)     |
| `age`      | text   | Time since creation (e.g., "3d ago") |

### Status kind mapping

| Condition          | Value    | StatusKind        |
|--------------------|----------|-------------------|
| state = open       | `open`   | Waiting           |
| state = closed     | `closed` | Idle              |

Note: Issues don't have as rich a status model as PRs. The `labels` field could
carry semantic meaning (e.g., a label named "bug" or "urgent") but mapping
arbitrary labels to StatusKind is fragile. Keep it simple.

### Activity identity

`{repo}/issues/{number}` or the URL from the API.

### Default interval

`120s`

### Implementation notes

- Very similar to `github-pr`. Could share even more infra.
- The cross-repo mode (`gh search issues`) uses a different API endpoint and
  JSON schema than `gh issue list`. If supporting both, handle the schema
  differences.
- Consider whether to support both single-repo and cross-repo, or just one.
  Cross-repo is more useful for a dashboard but the API is slightly different.

### Complexity: Low

Same CLI, same patterns. Slightly different JSON schema.

---

## 3. GitLab Merge Requests

**Feed type:** `gitlab-mr`

**What it tracks:** Merge requests authored by or assigned to the user in a
GitLab project. The GitLab equivalent of `github-pr`.

### Data source

**Option A -- `glab` CLI** (preferred, mirrors the `gh` pattern):

```sh
glab mr list --repo OWNER/PROJECT --author @me --json url,title,state,draft,labels,reviewers,pipeline
```

The `glab` CLI is the official GitLab CLI, actively maintained by GitLab.
Install: `brew install glab` or from https://gitlab.com/gitlab-org/cli.

**Option B -- GitLab REST API** (if `glab` is insufficient):

```
GET /api/v4/projects/:id/merge_requests?author_username=USER&state=opened
```

Requires a personal access token with `read_api` scope.

### Auth

**Option A (glab):** `glab auth status` -- same pattern as `gh`.
**Option B (API):** `token` field in TOML config, or `GITLAB_TOKEN` env var.

### Preflight

For `glab`:
1. `glab --version`
2. `glab auth status`

### Config

```toml
[[feed]]
name = "my gitlab mrs"
type = "gitlab-mr"
project = "group/project"    # GitLab project path
user = "@me"                 # Default: @me
host = "gitlab.com"          # Optional, for self-hosted instances
```

### Provided fields

| Field       | Type   | Description                              |
|------------|--------|------------------------------------------|
| `review`   | status | Approval status                          |
| `pipeline` | status | CI pipeline status                       |
| `draft`    | status | Draft state                              |
| `labels`   | text   | Comma-separated labels                   |
| `conflicts`| status | Merge conflict status                    |

### Status kind mapping

**review:**

| Condition                  | Value              | StatusKind        |
|----------------------------|--------------------|-------------------|
| approved (enough approvals)| `approved`         | AttentionPositive |
| changes requested          | `changes requested`| AttentionNegative |
| awaiting review            | `awaiting`         | Waiting           |
| no reviewers               | `none`             | Idle              |

**pipeline:**

| Condition                  | Value     | StatusKind        |
|----------------------------|-----------|-------------------|
| failed/cancelled           | `failing` | AttentionNegative |
| running/pending            | `running` | Running           |
| success                    | `passing` | Idle              |
| skipped/manual             | `skipped` | Idle              |

**draft:**

| Condition | Value | StatusKind        |
|-----------|-------|-------------------|
| true      | `yes` | AttentionPositive |
| false     | `no`  | Idle              |

### Activity identity

MR web URL.

### Default interval

`120s`

### Implementation notes

- The `glab` CLI has good JSON output support but it's less mature than `gh`.
  Test thoroughly.
- Self-hosted GitLab instances are common in enterprise. The `host` config field
  is important.
- `glab` supports `GITLAB_HOST` env var for self-hosted, which may be sufficient
  instead of a config field.
- The field mappings are very similar to `github-pr` -- could potentially share
  some mapping logic, but don't over-abstract.

### Complexity: Low-Medium

Same pattern as `github-pr` but with a different CLI. Self-hosted support adds
a small amount of complexity.

---

## 4. HTTP Health Check

**Feed type:** `http-health`

**What it tracks:** The health/availability of one or more HTTP endpoints.
Each activity is an endpoint. Shows whether services are up, slow, or down.

### Data source

Pure Rust HTTP client. No external CLI needed. Use `reqwest` (or the existing
Tauri HTTP plugin if available).

Performs an HTTP request (GET or HEAD) and evaluates:
- Response status code
- Response time
- Optionally, response body content (e.g., JSON health check field)

### Auth

No third-party auth. The feed itself makes HTTP requests. If the target
endpoint requires auth, support:
- `header` config (e.g., `Authorization: Bearer TOKEN`)
- Or document that users should use the `shell` feed with `curl` for
  authenticated health checks.

### Preflight

None needed (no external binary). Could optionally validate that the URL is
well-formed at config parse time.

### Config

```toml
[[feed]]
name = "api health"
type = "http-health"
url = "https://api.example.com/health"

# Optional
method = "GET"                     # GET (default) or HEAD
timeout = "5s"                     # Request timeout (default: 10s)
expected_status = 200              # Expected HTTP status (default: 200)
json_field = "status"              # Extract a field from JSON response body
json_healthy = "ok"                # Value that means healthy

# Multiple endpoints (alternative config shape)
# urls = ["https://api1.example.com/health", "https://api2.example.com/health"]
```

### Provided fields

| Field          | Type   | Description                        |
|---------------|--------|------------------------------------|
| `status`      | status | Health status (healthy/unhealthy)  |
| `response_time`| number | Response time in milliseconds      |
| `status_code` | number | HTTP status code                   |

### Status kind mapping

| Condition                         | Value       | StatusKind        |
|-----------------------------------|-------------|-------------------|
| Request failed (timeout, DNS, etc)| `down`      | AttentionNegative |
| Unexpected status code            | `unhealthy` | AttentionNegative |
| Response time > threshold         | `slow`      | Waiting           |
| JSON field != healthy value       | `degraded`  | Waiting           |
| All good                          | `healthy`   | Idle              |

### Activity identity

The URL itself (or a shortened version).

### Activity title

URL hostname + path (e.g., `api.example.com/health`).

### Default interval

`60s` (health checks can be more frequent since they're lightweight).

### Implementation notes

- **New dependency required:** `reqwest` (with `rustls-tls` feature, not
  `native-tls`, to avoid OpenSSL). This is a significant but well-justified
  dependency. Alternatively, check if Tauri's HTTP plugin can be used from the
  Rust side.
- Single-URL is simpler. Multi-URL (one activity per URL) is more useful.
  Recommend supporting both via `url` (single) vs `urls` (multi) config fields.
- For the JSON body check: parse the response as JSON, extract a field by path,
  compare to expected value. Keep it simple -- single top-level field, no
  JSONPath.
- Consider adding `slow_threshold` config (e.g., `"2s"`) for the "slow" status.
- This feed is unique in that it doesn't depend on any external CLI -- it's a
  pure Rust implementation. This is a different pattern from all existing feeds.

### Open questions

- Should we add `reqwest` as a dependency, or use `tauri-plugin-http`'s Rust
  API? Need to check if the plugin exposes a usable Rust-side client.
- Should multi-URL be supported, or should users create separate feeds per URL?
  Separate feeds is simpler and more consistent with the existing model.

### Complexity: Medium

New dependency, new pattern (pure Rust HTTP instead of CLI). But the logic
itself is straightforward.

---

## 5. Docker Containers

**Feed type:** `docker`

**What it tracks:** Running/stopped Docker containers on the local machine.
Each activity is a container. Useful for developers running local services.

### Data source

CLI: `docker ps` / `docker container ls`

```sh
docker container ls --all --format '{{json .}}' --no-trunc
```

Returns JSON per line with fields: `ID`, `Names`, `Image`, `Status`, `State`,
`Ports`, `CreatedAt`.

Alternatively, for structured output:

```sh
docker container ls --all --format json
```

(Docker 23+ supports `--format json` natively.)

### Auth

None. Docker CLI uses the local Docker socket. If Docker Desktop is not running
or the socket is inaccessible, the preflight check will catch it.

### Preflight

1. `docker --version` -- binary exists
2. `docker info --format '{{.ServerVersion}}'` -- daemon is running

If daemon is not running:
- "Docker feed requires Docker Desktop or Docker Engine to be running."

### Config

```toml
[[feed]]
name = "my containers"
type = "docker"

# Optional filters
label = "project=myapp"       # Filter by container label
name_pattern = "myapp-*"      # Filter by container name glob
image = "postgres"             # Filter by image name
show_stopped = false           # Include stopped containers (default: false)
```

### Provided fields

| Field    | Type   | Description                              |
|---------|--------|------------------------------------------|
| `state` | status | Container state (running, exited, etc)   |
| `image` | text   | Image name                               |
| `ports` | text   | Published ports                          |
| `uptime`| text   | How long the container has been running  |

### Status kind mapping

| Condition          | Value      | StatusKind        |
|--------------------|------------|-------------------|
| state = running    | `running`  | Idle              |
| state = restarting | `restarting`| Running          |
| state = paused     | `paused`   | Waiting           |
| state = exited (0) | `stopped`  | Idle              |
| state = exited (!0)| `crashed`  | AttentionNegative |
| state = dead       | `dead`     | AttentionNegative |
| state = created    | `created`  | Waiting           |

### Activity identity

Container ID (short form, 12 chars).

### Activity title

Container name (without leading `/`).

### Default interval

`30s` (local operation, fast).

### Implementation notes

- `docker` CLI is very common on developer machines. Good coverage.
- The `--format json` flag (Docker 23+) gives clean JSON. For older versions,
  fall back to `--format '{{json .}}'` which outputs JSON per line.
- Filtering by label/name/image can all be done with `docker container ls`
  flags (`--filter`), which is more efficient than filtering in Rust.
- Container state is very well-defined in Docker's API.
- Consider health check status: Docker containers with `HEALTHCHECK` have a
  `Health.Status` field (healthy/unhealthy/starting). This could be surfaced
  as an additional field.

### Complexity: Low

Simple CLI, well-structured JSON output, no auth.

---

## 6. Kubernetes Pods

**Feed type:** `kubernetes-pod`

**What it tracks:** Pod status in a Kubernetes namespace. Each activity is a
pod. For developers working with K8s clusters.

### Data source

CLI: `kubectl get pods`

```sh
kubectl get pods --namespace NAMESPACE --output json
```

Returns a `PodList` JSON object with `.items[]` containing full pod specs and
status.

For a more targeted query:

```sh
kubectl get pods -n NAMESPACE -l app=myapp -o json
```

### Auth

`kubectl` uses the kubeconfig file (`~/.kube/config`) for authentication.
No additional auth config needed in cortado -- if `kubectl` works, the feed
works.

Optionally support:
- `context` config field to select a specific kubeconfig context
- `kubeconfig` config field for a custom kubeconfig path

### Preflight

1. `kubectl version --client --output json` -- binary exists
2. `kubectl cluster-info --request-timeout 5s` -- cluster is reachable

If cluster unreachable:
- "Kubernetes feed cannot reach the cluster. Check your kubeconfig and cluster
  connectivity."

### Config

```toml
[[feed]]
name = "staging pods"
type = "kubernetes-pod"
namespace = "staging"          # Required

# Optional
context = "my-cluster"         # Kubeconfig context
selector = "app=myapp"         # Label selector
```

### Provided fields

| Field      | Type   | Description                              |
|-----------|--------|------------------------------------------|
| `phase`   | status | Pod phase (Running, Pending, etc)        |
| `ready`   | status | Ready condition                          |
| `restarts`| number | Total restart count across containers    |
| `age`     | text   | Time since pod creation                  |
| `node`    | text   | Node the pod is scheduled on             |

### Status kind mapping

**phase:**

| Condition         | Value       | StatusKind        |
|-------------------|-------------|-------------------|
| phase = Running, all containers ready | `running` | Idle |
| phase = Running, not all ready | `degraded` | Waiting |
| phase = Pending   | `pending`   | Running           |
| phase = Succeeded | `completed` | Idle              |
| phase = Failed    | `failed`    | AttentionNegative |
| phase = Unknown   | `unknown`   | Waiting           |

**ready:**

| Condition          | Value      | StatusKind        |
|--------------------|------------|-------------------|
| All ready          | `yes`      | Idle              |
| Partially ready    | `partial`  | Waiting           |
| None ready         | `no`       | AttentionNegative |

### Activity identity

`{namespace}/{pod_name}`

### Activity title

Pod name.

### Default interval

`30s`

### Implementation notes

- `kubectl` output is very verbose. Parse the full JSON but extract only what's
  needed.
- Pod status is nuanced: a pod can be "Running" but with containers in
  CrashLoopBackOff. The `ready` field and `restarts` count help surface this.
- For CrashLoopBackOff: detect via container status
  `waiting.reason = "CrashLoopBackOff"` and map to AttentionNegative.
- The `kubectl` JSON schema is well-documented (Kubernetes API spec).
- Consider capping activities to avoid flooding when a namespace has hundreds
  of pods. The label selector helps, but also apply the existing 20-activity cap.

### Open questions

- Should we support Deployment-level view (one activity per deployment, showing
  rollout status) instead of or in addition to pod-level? Deployment-level is
  more "dashboard-like" but pod-level gives more detail.

### Complexity: Medium

Well-structured JSON, but the Kubernetes status model is complex (phases,
conditions, container statuses, restart counts). Needs careful mapping.

---

## 7. Linear Issues

**Feed type:** `linear`

**What it tracks:** Issues assigned to the user in Linear. Each activity is
an issue. Linear is widely used in startups and modern dev teams.

### Data source

**Option A -- Linear CLI** (`linear`):

Linear doesn't have an official CLI with good JSON output. There is a
community `linear` CLI but it's not well-maintained.

**Option B -- Linear GraphQL API** (recommended):

```graphql
query {
  viewer {
    assignedIssues(
      filter: { state: { type: { nin: ["completed", "cancelled"] } } }
      first: 20
      orderBy: updatedAt
    ) {
      nodes {
        id identifier title url priority priorityLabel
        state { name type color }
        labels { nodes { name } }
        project { name }
        cycle { name number }
      }
    }
  }
}
```

Endpoint: `https://api.linear.app/graphql`

### Auth

**Linear API key:** Personal API key created at
https://linear.app/settings/api.

Config:
```toml
token_env = "LINEAR_API_KEY"     # Env var containing the API key
```

Or:
```toml
token_command = "security find-generic-password -s linear-api-key -w"
```

Using a `token_command` that calls macOS Keychain is more secure than env vars.

### Preflight

Make a lightweight API call:
```graphql
query { viewer { id } }
```

If 401: "Linear feed requires a valid API key. Create one at
https://linear.app/settings/api and set it via `token_env` or `token_command`."

### Config

```toml
[[feed]]
name = "my linear"
type = "linear"
token_env = "LINEAR_API_KEY"

# Optional
team = "ENG"                   # Filter by team key
project = "Project Name"       # Filter by project
include_backlog = false        # Include backlog items (default: false)
```

### Provided fields

| Field      | Type   | Description                    |
|-----------|--------|--------------------------------|
| `state`   | status | Workflow state                 |
| `priority`| status | Priority level                 |
| `project` | text   | Project name                   |
| `labels`  | text   | Comma-separated label names    |
| `cycle`   | text   | Current cycle name             |

### Status kind mapping

**state:**

| Condition                         | Value        | StatusKind        |
|-----------------------------------|--------------|-------------------|
| state.type = started (In Progress)| `in progress`| Running           |
| state.type = unstarted (Todo)     | `todo`       | Waiting           |
| state.type = backlog              | `backlog`    | Idle              |
| state.type = completed            | `done`       | Idle              |
| state.type = cancelled            | `cancelled`  | Idle              |

**priority:**

| Condition        | Value    | StatusKind        |
|------------------|----------|-------------------|
| Urgent (1)       | `urgent` | AttentionNegative |
| High (2)         | `high`   | Waiting           |
| Medium (3)       | `medium` | Idle              |
| Low (4) / None   | `low`    | Idle              |

### Activity identity

Issue identifier (e.g., `ENG-123`).

### Activity title

Issue title.

### Default interval

`120s`

### Implementation notes

- **New dependency required:** An HTTP client (`reqwest`) for the GraphQL API.
  If `http-health` is also implemented, they can share this dependency.
- Linear's GraphQL API is clean and well-documented.
- The `token_command` pattern (shelling out to retrieve a secret) is a general
  pattern that could be reused across API-backed feeds. Consider building a
  shared `TokenSource` abstraction: `{ env: String }` or
  `{ command: String }`.
- Linear's state model (workflow states) maps cleanly to StatusKind.
- Filtering by team/project should happen at the API level (GraphQL filters),
  not client-side.

### Open questions

- The `token_command` / `token_env` pattern is new. It would be the first feed
  to introduce an auth pattern beyond "CLI handles it." This pattern should be
  designed carefully as it will be reused by other API-backed feeds (Sentry,
  PagerDuty, etc.). Consider designing this as a shared config pattern first.

### Complexity: Medium

New pattern (direct API calls instead of CLI), new auth mechanism. But the API
itself is clean and well-documented.

---

## 8. Jira Issues

**Feed type:** `jira`

**What it tracks:** Jira issues assigned to the user. Each activity is a Jira
issue. Jira is the most widely used issue tracker in enterprise.

### Data source

**Jira REST API v3:**

```
GET https://{domain}.atlassian.net/rest/api/3/search?jql=assignee=currentUser()+AND+statusCategory!=Done&maxResults=20&fields=summary,status,priority,labels,project,issuetype
```

There is also an `atlas` CLI from Atlassian, but it's not widely adopted. The
REST API is more reliable.

### Auth

Jira Cloud uses **API tokens** (not OAuth for personal use):
- Email + API token, sent as Basic auth:
  `Authorization: Basic base64(email:token)`

Create at: https://id.atlassian.com/manage-profile/security/api-tokens

Config:
```toml
token_env = "JIRA_API_TOKEN"
email_env = "JIRA_EMAIL"          # or hardcode email in config
```

Or:
```toml
token_command = "security find-generic-password -s jira-api-token -w"
email = "user@example.com"
```

For Jira Server/Data Center (self-hosted):
- Personal access tokens (PAT) with `Bearer` auth.

### Preflight

Make a lightweight API call:
```
GET /rest/api/3/myself
```

If 401: "Jira feed requires valid credentials. Create an API token at
https://id.atlassian.com/manage-profile/security/api-tokens."

### Config

```toml
[[feed]]
name = "my jira"
type = "jira"
domain = "mycompany"             # {domain}.atlassian.net
email = "user@example.com"
token_env = "JIRA_API_TOKEN"

# Optional
jql = "project = ENG AND sprint in openSprints()"   # Custom JQL filter
project = "ENG"                  # Simple project filter (alternative to jql)
```

### Provided fields

| Field      | Type   | Description                    |
|-----------|--------|--------------------------------|
| `status`  | status | Issue status                   |
| `priority`| status | Priority level                 |
| `type`    | text   | Issue type (Bug, Story, etc)   |
| `project` | text   | Project key                    |
| `labels`  | text   | Comma-separated labels         |

### Status kind mapping

**status (by status category):**

| Condition               | Value          | StatusKind        |
|-------------------------|----------------|-------------------|
| category = In Progress  | status name    | Running           |
| category = To Do        | status name    | Waiting           |
| category = Done         | status name    | Idle              |

Jira's status categories are a reliable abstraction over custom workflow
statuses. Use the category, not the individual status name, for StatusKind
mapping.

**priority:**

| Condition    | Value      | StatusKind        |
|-------------|------------|-------------------|
| Blocker     | `blocker`  | AttentionNegative |
| Critical    | `critical` | AttentionNegative |
| Major/High  | `high`     | Waiting           |
| Medium      | `medium`   | Idle              |
| Low/Trivial | `low`      | Idle              |

### Activity identity

Issue key (e.g., `ENG-1234`).

### Activity title

Issue summary.

### Default interval

`120s`

### Implementation notes

- Jira's API is well-documented but verbose. Parse only the needed fields.
- JQL (Jira Query Language) is very powerful. Supporting a raw `jql` config
  field gives advanced users full control.
- Jira Cloud vs Jira Server/Data Center have slightly different APIs and auth.
  Consider supporting Cloud only initially and documenting the limitation.
- The Basic auth pattern (email:token) is different from Bearer token. Need
  to handle both if supporting self-hosted.
- This is a very high-value feed for enterprise users, but also more complex
  due to Jira's configuration flexibility (custom fields, workflows, etc.).

### Open questions

- Support Jira Cloud only, or also Server/Data Center? Cloud-only is simpler.
- Should custom fields be supported? They're common in Jira but significantly
  increase complexity. Recommendation: don't support custom fields initially.

### Complexity: Medium-High

Well-documented API, but auth is more complex (Basic vs Bearer), and the
Jira ecosystem is highly configurable. Keep scope tight.

---

## 9. Sentry Issues

**Feed type:** `sentry`

**What it tracks:** Unresolved issues (errors) in a Sentry project. Each
activity is a Sentry issue (grouped error). For developers who monitor
production errors.

### Data source

**Sentry API:**

```
GET https://sentry.io/api/0/projects/{org}/{project}/issues/?query=is:unresolved&sort=date&limit=20
```

Returns JSON array of issues with fields: `id`, `title`, `status`, `level`,
`count`, `userCount`, `firstSeen`, `lastSeen`, `permalink`.

### Auth

**Sentry Auth Token:** Create at https://sentry.io/settings/account/api/auth-tokens/
Requires `project:read` scope.

```
Authorization: Bearer {token}
```

Config:
```toml
token_env = "SENTRY_AUTH_TOKEN"
```

Or for self-hosted:
```toml
token_command = "..."
host = "https://sentry.mycompany.com"
```

### Preflight

```
GET /api/0/
```

Returns API metadata. If 401: auth error message with link to token creation.

### Config

```toml
[[feed]]
name = "api errors"
type = "sentry"
org = "my-org"
project = "my-project"
token_env = "SENTRY_AUTH_TOKEN"

# Optional
host = "https://sentry.io"      # Default; override for self-hosted
query = "is:unresolved level:error"  # Sentry search query
```

### Provided fields

| Field        | Type   | Description                          |
|-------------|--------|--------------------------------------|
| `level`     | status | Error level (fatal, error, warning)  |
| `events`    | number | Total event count                    |
| `users`     | number | Affected user count                  |
| `last_seen` | text   | Time since last occurrence           |
| `status`    | status | Issue status (unresolved, etc)       |

### Status kind mapping

**level:**

| Condition      | Value     | StatusKind        |
|----------------|-----------|-------------------|
| fatal          | `fatal`   | AttentionNegative |
| error          | `error`   | AttentionNegative |
| warning        | `warning` | Waiting           |
| info           | `info`    | Idle              |
| debug          | `debug`   | Idle              |

### Activity identity

Sentry issue permalink.

### Activity title

Issue title (error message or type).

### Default interval

`120s`

### Implementation notes

- Sentry's API is simple and well-documented. Clean JSON responses.
- The `query` config field maps directly to Sentry's search syntax, giving
  advanced users full control.
- Self-hosted Sentry is common. Support via `host` config.
- The event/user counts are useful for prioritization but change frequently.
  The `last_seen` field is more stable.
- **Shares the `reqwest` + `token_env` pattern** with Linear, Jira, etc.

### Complexity: Low-Medium

Simple API, clean data model. Shares patterns with other API-backed feeds.

---

## 10. PagerDuty Incidents

**Feed type:** `pagerduty`

**What it tracks:** Active incidents assigned to or acknowledged by the user.
Each activity is an incident. For developers on-call rotations.

### Data source

**PagerDuty REST API v2:**

```
GET https://api.pagerduty.com/incidents?statuses[]=triggered&statuses[]=acknowledged&user_ids[]={user_id}&limit=20
```

To get the current user ID:
```
GET https://api.pagerduty.com/users/me
```

### Auth

**PagerDuty API key:** User token or API key.
Create at: Account Settings > API Access > Create New API Key.

```
Authorization: Token token={api_key}
```

Config:
```toml
token_env = "PAGERDUTY_API_KEY"
```

### Preflight

```
GET /users/me
```

If 401: "PagerDuty feed requires a valid API key. Create one in PagerDuty
Account Settings > API Access."

### Config

```toml
[[feed]]
name = "on-call"
type = "pagerduty"
token_env = "PAGERDUTY_API_KEY"

# Optional
include_acknowledged = true    # Include acknowledged incidents (default: true)
team_ids = ["PTEAMID"]         # Filter by team
service_ids = ["PSVCID"]       # Filter by service
```

### Provided fields

| Field       | Type   | Description                         |
|------------|--------|-------------------------------------|
| `urgency`  | status | Incident urgency (high/low)         |
| `status`   | status | Triggered / Acknowledged            |
| `service`  | text   | Service name                        |
| `escalation`| text  | Escalation policy name              |
| `age`      | text   | Time since incident creation        |

### Status kind mapping

**status:**

| Condition     | Value          | StatusKind        |
|---------------|----------------|-------------------|
| triggered     | `triggered`    | AttentionNegative |
| acknowledged  | `acknowledged` | Waiting           |
| resolved      | `resolved`     | Idle              |

**urgency:**

| Condition | Value  | StatusKind        |
|-----------|--------|-------------------|
| high      | `high` | AttentionNegative |
| low       | `low`  | Waiting           |

### Activity identity

Incident URL (e.g., `https://mycompany.pagerduty.com/incidents/PINCID`).

### Activity title

Incident title.

### Default interval

`30s` (incidents are time-sensitive, poll frequently).

### Implementation notes

- PagerDuty's API is clean and well-documented.
- The `/users/me` endpoint is useful for resolving the user ID automatically.
- Incidents are time-sensitive -- this feed benefits from a shorter default
  interval.
- The `triggered` status is the most urgent -- it means nobody has responded
  yet.
- This feed has high value for on-call developers but a smaller audience.
- **Shares the `reqwest` + `token_env` pattern** with other API feeds.

### Complexity: Low-Medium

Simple API, small data model. The value proposition is clear and focused.

---

## 11. Vercel Deployments

**Feed type:** `vercel`

**What it tracks:** Recent deployments for a Vercel project. Each activity
is a deployment. Shows deploy status and production state.

### Data source

**Option A -- Vercel CLI:**

```sh
vercel list --json --limit 20
```

The Vercel CLI requires login via `vercel login`.

**Option B -- Vercel REST API** (more reliable for structured data):

```
GET https://api.vercel.com/v6/deployments?projectId={projectId}&limit=20
```

### Auth

**Option A (CLI):** `vercel whoami` to check auth. Login via `vercel login`.
**Option B (API):** Bearer token. Create at https://vercel.com/account/tokens.

For CLI approach:
```toml
# No auth config needed -- uses vercel CLI auth
```

For API approach:
```toml
token_env = "VERCEL_TOKEN"
```

### Preflight

**CLI:**
1. `vercel --version`
2. `vercel whoami`

**API:**
```
GET /v2/user
```

### Config

```toml
[[feed]]
name = "my deploys"
type = "vercel"
project = "my-project"

# Optional
team = "my-team"               # Vercel team slug
prod_only = false              # Only show production deployments
```

### Provided fields

| Field       | Type   | Description                         |
|------------|--------|-------------------------------------|
| `state`    | status | Deployment state                    |
| `target`   | text   | Target environment (production/preview) |
| `branch`   | text   | Git branch                          |
| `age`      | text   | Time since deployment               |
| `url`      | url    | Deployment URL                      |

### Status kind mapping

| Condition          | Value        | StatusKind        |
|--------------------|--------------|-------------------|
| state = ERROR      | `error`      | AttentionNegative |
| state = CANCELED   | `cancelled`  | AttentionNegative |
| state = BUILDING   | `building`   | Running           |
| state = INITIALIZING| `queued`    | Running           |
| state = QUEUED     | `queued`     | Waiting           |
| state = READY      | `ready`      | Idle              |

### Activity identity

Deployment URL or deployment UID.

### Activity title

`{branch} -> {target}` or the deployment URL.

### Default interval

`60s`

### Implementation notes

- The CLI approach is simpler and consistent with the `gh`/`glab` pattern.
- The API approach is more reliable for structured data and doesn't require
  the CLI to be installed.
- Choose one approach. CLI is recommended for consistency with existing feeds.
- The Vercel CLI (`vercel`) is installed via npm: `npm i -g vercel`. This is
  a less common install path than `brew install gh`.

### Open questions

- CLI or API? CLI is simpler, API is more reliable. The CLI's `--json` output
  may not be stable across versions.
- Is Vercel common enough to justify a curated feed? It's very popular for
  frontend/Next.js projects.

### Complexity: Low-Medium

Either CLI or API, both are straightforward. Data model is simple.

---

## 12. RSS/Atom Feed

**Feed type:** `rss`

**What it tracks:** Items from any RSS 2.0 or Atom feed. Each activity is a
feed item. Useful for tracking release blogs, changelogs, security advisories,
or any content feed.

### Data source

Pure Rust HTTP + XML/RSS parsing. Fetch the feed URL and parse the XML.

```
GET https://example.com/feed.xml
```

### Auth

Typically none (RSS feeds are public). For authenticated feeds, support a
`header` config or `token_env`.

### Preflight

None needed (no external binary). Validate URL format at config time.

### Config

```toml
[[feed]]
name = "rust blog"
type = "rss"
url = "https://blog.rust-lang.org/feed.xml"

# Optional
limit = 10                     # Max items to show (default: 10)
title_contains = "release"     # Filter items by title substring
```

### Provided fields

| Field     | Type   | Description                    |
|----------|--------|--------------------------------|
| `date`   | text   | Publication date               |
| `source` | text   | Feed/channel title             |
| `link`   | url    | Item URL                       |

### Status kind mapping

RSS items don't have inherent status. Options:
- All items are `Idle` (simple, no status field).
- Age-based: items < 24h old are `AttentionPositive` ("new"), older are `Idle`.
- No status field at all (just text fields).

Recommendation: **No status field.** RSS is informational, not actionable.
If users want status, they can use the `shell` feed with a custom script.

### Activity identity

Item GUID or link URL.

### Activity title

Item title.

### Default interval

`300s` (5 minutes -- RSS feeds update infrequently).

### Implementation notes

- **New dependency required:** An RSS/XML parser. Options:
  - `feed-rs` (parses RSS 1.0, RSS 2.0, Atom, JSON Feed) -- well-maintained.
  - `rss` + `atom_syndication` (separate crates for each format).
  - `quick-xml` (low-level XML parser, build RSS parsing manually).
  - Recommendation: `feed-rs` -- handles all formats, single dependency.
- Also needs `reqwest` for HTTP fetching (shared with `http-health`, `linear`,
  etc.).
- This feed is unique in that its activities have no actionable status.
  It's purely informational. This is fine -- not all feeds need status fields.
- The `title_contains` filter is simple substring matching. Could expand to
  regex later, but keep it simple initially.
- Some RSS feeds are very large (hundreds of items). The `limit` config and
  the existing 20-activity cap handle this.

### Open questions

- Is a purely informational (no status) feed valuable in cortado? The app's
  design is oriented around status-driven attention. An RSS feed without status
  might feel out of place. On the other hand, it's a commonly requested feature
  in dashboard tools.
- Should we add an "age-based" status (new vs old) to give RSS items some
  visual differentiation? This would require an `age_threshold` config.

### Complexity: Medium

New dependency (RSS parser), new pattern (no status fields), HTTP fetching.

---

## 13. npm Outdated

**Feed type:** `npm-outdated`

**What it tracks:** Outdated npm/pnpm dependencies in a project. Each activity
is an outdated package. Helps developers stay on top of dependency updates.

### Data source

CLI: `npm outdated` or `pnpm outdated`

```sh
npm outdated --json --long
```

Returns JSON object keyed by package name:
```json
{
  "lodash": {
    "current": "4.17.20",
    "wanted": "4.17.21",
    "latest": "4.17.21",
    "dependent": "my-app",
    "type": "dependencies",
    "homepage": "https://lodash.com/"
  }
}
```

For pnpm:
```sh
pnpm outdated --format json
```

Similar output structure.

### Auth

None. Reads from the local project and public registry.

### Preflight

1. `npm --version` (or `pnpm --version`) -- binary exists
2. Check that `package.json` exists at the configured path.

### Config

```toml
[[feed]]
name = "my deps"
type = "npm-outdated"
path = "/path/to/project"       # Project directory with package.json

# Optional
manager = "pnpm"                # npm (default) or pnpm
include_dev = true              # Include devDependencies (default: true)
major_only = false              # Only show major version bumps (default: false)
```

### Provided fields

| Field     | Type   | Description                         |
|----------|--------|-------------------------------------|
| `gap`    | status | Version gap severity                |
| `current`| text   | Currently installed version         |
| `latest` | text   | Latest available version            |
| `type`   | text   | dependencies / devDependencies      |

### Status kind mapping

**gap (based on semver distance):**

| Condition          | Value   | StatusKind        |
|--------------------|---------|-------------------|
| Major version bump | `major` | AttentionNegative |
| Minor version bump | `minor` | Waiting           |
| Patch version bump | `patch` | Idle              |

### Activity identity

Package name.

### Activity title

`{package}@{current} -> {latest}`

### Default interval

`3600s` (1 hour -- dependency updates are infrequent and the check is slow).

### Implementation notes

- The `npm outdated --json` command is slow (several seconds) because it hits
  the npm registry. The long default interval accounts for this.
- Semver gap detection: compare major/minor/patch of `current` vs `latest` to
  determine the severity. Use string parsing or a semver library.
- The output format differs slightly between npm and pnpm. Handle both.
- This feed can produce many activities (large projects have many deps). The
  `major_only` filter and the 20-activity cap help.
- Consider sorting by gap severity (major first, then minor, then patch).

### Open questions

- Should this support yarn as well? Yarn's `outdated` output is different.
  Could add later.
- Is this feed better served by the `shell` feed with a custom script? The
  curated feed adds semver gap detection and proper status mapping, which
  a shell command can't do easily.

### Complexity: Low-Medium

Simple CLI, well-structured JSON. Semver parsing adds a small amount of logic.

---

## 14. SSL Certificate Expiry

**Feed type:** `ssl-cert`

**What it tracks:** SSL/TLS certificate expiry for one or more domains.
Each activity is a domain. Alerts when certificates are approaching expiry.

### Data source

**Option A -- OpenSSL CLI** (available on macOS by default):

```sh
echo | openssl s_client -servername DOMAIN -connect DOMAIN:443 2>/dev/null | openssl x509 -noout -enddate -subject
```

Returns:
```
subject=CN = example.com
notAfter=Dec 31 23:59:59 2025 GMT
```

**Option B -- Pure Rust TLS** (using `rustls` or `native-tls`):

Connect to the domain, extract the certificate, read the expiry date. More
complex but no external CLI dependency.

### Auth

None. SSL certificates are public.

### Preflight

For OpenSSL: `openssl version` -- binary exists.
For pure Rust: none needed.

### Config

```toml
[[feed]]
name = "my certs"
type = "ssl-cert"
domains = ["example.com", "api.example.com", "*.example.com"]

# Optional
warn_days = 30                 # Days before expiry to warn (default: 30)
critical_days = 7              # Days before expiry to alert (default: 7)
port = 443                     # Default: 443
```

### Provided fields

| Field      | Type   | Description                         |
|-----------|--------|-------------------------------------|
| `expiry`  | status | Expiry status                       |
| `days`    | number | Days until expiry                   |
| `issuer`  | text   | Certificate issuer (e.g., Let's Encrypt) |
| `subject` | text   | Certificate subject/CN              |

### Status kind mapping

| Condition                   | Value      | StatusKind        |
|-----------------------------|------------|-------------------|
| days <= critical_days       | `critical` | AttentionNegative |
| days <= warn_days           | `expiring` | Waiting           |
| days > warn_days            | `valid`    | Idle              |
| Connection/TLS error        | `error`    | AttentionNegative |

### Activity identity

Domain name.

### Activity title

Domain name.

### Default interval

`3600s` (1 hour -- certificates don't change frequently).

### Implementation notes

- The OpenSSL CLI approach is simpler and OpenSSL is pre-installed on macOS.
- Date parsing: `notAfter` format is `Mon DD HH:MM:SS YYYY GMT`. Parse with
  `jiff` or `chrono`.
- Multiple domains = multiple activities. Each domain requires a separate TLS
  connection, so polling is inherently sequential (or use `map_concurrent`).
- This is a "set it and forget it" feed -- configure once, get alerted when
  certs are about to expire.
- Consider supporting `SNI` (Server Name Indication) for domains behind
  shared hosting.

### Open questions

- OpenSSL CLI vs pure Rust? OpenSSL is simpler and already on macOS. Pure
  Rust avoids the CLI dependency but adds complexity.

### Complexity: Low

Simple CLI output parsing, date arithmetic. Very focused use case.

---

## Cross-Cutting Design Considerations

### Shared auth pattern for API-backed feeds

Feeds 7-11 (Linear, Jira, Sentry, PagerDuty, Vercel) all need API
authentication. They share a common pattern:

```toml
token_env = "ENV_VAR_NAME"           # Read token from env var
# or
token_command = "some-command"       # Run a command to get the token
```

This should be designed as a shared `TokenSource` type:

```rust
enum TokenSource {
    Env(String),          // Read from env var
    Command(String),      // Run a command, use stdout as token
}
```

Build this once before implementing any API-backed feed. It will be reused
across all of them.

### Shared HTTP client

Feeds 4, 7, 8, 9, 10, 11, 12, 14 all need HTTP. If `reqwest` is added, it
should be added once and shared. Consider wrapping it in a thin
`HttpFeedClient` that handles:
- Timeout
- User-Agent header (e.g., `cortado/{version}`)
- Token injection from `TokenSource`
- Error normalization

### Implementation priority suggestion

If implementing a subset, consider this ordering by value/effort ratio:

1. **GitHub Actions** -- lowest effort (same CLI, same auth as `github-pr`)
2. **HTTP Health Check** -- high value, moderate effort (needs `reqwest`)
3. **Docker Containers** -- low effort, good for local dev
4. **Linear Issues** -- establishes the API-backed feed pattern
5. **SSL Certificate Expiry** -- low effort, unique use case
6. **Sentry Issues** -- reuses the API pattern from Linear
7. **npm Outdated** -- low effort, unique use case
8. **GitLab Merge Requests** -- valuable for GitLab users
9. **Kubernetes Pods** -- valuable for K8s users, moderate complexity
10. **PagerDuty Incidents** -- niche but high value for on-call
11. **Vercel Deployments** -- niche, good for frontend teams
12. **Jira Issues** -- high value but complex (enterprise auth, JQL)
13. **RSS/Atom Feed** -- informational only, may not fit cortado's model
14. **Datadog Monitors** -- added below as a bonus entry

---

## Bonus: Additional Candidates (Brief Notes)

These didn't make the detailed list but are worth mentioning for completeness:

### Datadog Monitors

- Track monitor alert status. API: `GET /api/v1/monitor`.
- Auth: API key + Application key.
- Status mapping: `OK` -> Idle, `Warn` -> Waiting, `Alert` -> AttentionNegative.
- Similar pattern to Sentry/PagerDuty.
- Niche audience (Datadog is expensive / enterprise).

### Fly.io Machines

- Track Fly app/machine status. CLI: `fly status --json`.
- Auth: `fly auth login`.
- Similar pattern to Docker.
- Smaller user base than Docker/K8s.

### Homebrew Outdated

- Track outdated Homebrew packages. CLI: `brew outdated --json`.
- No auth. macOS-specific (which is fine for cortado).
- Similar to `npm-outdated` but for system packages.
- Simpler (no semver gap analysis, just outdated/current).

### Buildkite Pipelines

- Track CI/CD pipeline status. API: `GET /v2/organizations/{org}/builds`.
- Auth: API token.
- Similar to GitHub Actions but for Buildkite users.

### Uptime Robot

- Track website uptime monitors. API: `POST /v2/getMonitors`.
- Auth: API key.
- Overlaps with `http-health` but uses a third-party monitoring service.
- Could be useful for users already using Uptime Robot.

---

## General Open Questions

1. **HTTP client dependency:** Several feeds need `reqwest`. Should we commit
   to adding it? It's a well-maintained, widely-used crate but it's a
   significant dependency (pulls in hyper, tokio, etc. -- though we already
   have tokio). Alternative: use Tauri's built-in HTTP capabilities from Rust.

2. **Token management pattern:** The `token_env` / `token_command` pattern
   is critical for API-backed feeds. Should this be a prerequisite task
   (build the shared `TokenSource` infra) before implementing any API feed?
   I'd recommend yes.

3. **RSS without status:** Is a purely informational feed (no StatusKind)
   valuable in cortado's status-oriented design? Or does every feed need at
   least one status field to feel "at home" in the UI?

4. **How many feeds to implement?** Each feed is ~200-400 lines of Rust plus
   tests. Implementing all 14 is feasible but time-consuming. Recommend
   picking 4-6 that cover different patterns (CLI-backed, API-backed,
   pure-Rust, status-rich, informational).

5. **Feed-type specific config validation:** Currently, unknown config keys
   in `type_specific` are silently preserved. Should we add per-feed config
   validation that warns about unknown keys? This would catch typos.

6. **Should CLI-backed feeds share a preflight check helper?** `github-pr`,
   `github-actions`, and `github-issue` all use the same `gh` preflight.
   Extract to a shared function?
