---
name: tool-github-cli
description: Use GitHub CLI (gh) from agents for PRs, issues, checks, and API access. Covers --json/--jq, pagination, repo targeting, and gh api. Use when the user mentions gh, GitHub PRs, Actions checks, or GitHub API tasks. Requires gh auth (gh auth login). Do NOT use for non-GitHub VCS.
---

# GitHub CLI (gh) for agents

Patterns adapted from [cli/cli `skills/gh`](https://github.com/cli/cli/blob/trunk/skills/gh/SKILL.md) (MIT). PR check workflows align with [tech-leads-club/agent-skills `gh-fix-ci`](https://github.com/tech-leads-club/agent-skills).

## Prerequisites

1. `gh` installed (`gh --version`).
2. Authenticated: `gh auth status`. If not, ask the user to run `gh auth login` (repo + workflow scopes for PR/Actions work).

## Interactivity

`gh` skips the pager and avoids prompts in non-TTY contexts. You usually do not need `GH_PAGER` or `--no-pager`.

## Structured output

- Add `--json field1,field2,...` for structured output.
- Run with `--json` and no field list to see all available fields.
- Use `--jq 'filter'` to filter without piping to `jq`.
- Use `--template` with `--json` for shaped text (check `--help` on subcommands; `-T` collides on a few commands like `gh pr create`).

## Pagination

List commands cap results (often 30 by default).

- Pass `-L N` / `--limit N` on `gh issue list`, `gh pr list`, `gh search ...`.
- `gh issue list` / `gh pr list` do not expose `totalCount` via `--json`; use `gh api graphql` if you need a true total.
- For raw API: `gh api --paginate <path>`; combine with `--jq` and optionally `--slurp`.

## Repo targeting

`gh` infers the repo from cwd git remotes. Pass `--repo OWNER/REPO` (`-R`) to override.

## Search vs list

- `gh search issues|prs|code|repos|commits|users` — full GitHub search syntax; pass the whole query as one quoted string.
- `gh issue list --search "..."` / `gh pr list --search "..."` — scoped to one repo.

## Fall back to `gh api`

When typed commands lack a field:

- PR review-thread comments: `gh api repos/{owner}/{repo}/pulls/{n}/comments` (`gh pr view --comments` is issue-level only).
- GraphQL: `gh api graphql -f query='...' -F var=value`.
- REST: `gh api repos/{owner}/{repo}/...` — `{owner}/{repo}` may be filled from detected remotes; pass literally for determinism.

## Common workflows

### Current branch PR

```bash
gh pr view --json number,url,title,state
gh pr checks
gh pr diff
```

### Create or update PR

```bash
gh pr create --title "..." --body "..."
gh pr edit --add-label "..."
```

### Issues

```bash
gh issue list -L 20 --json number,title,state
gh issue view 123 --json title,body,comments
```

### Debug failing GitHub Actions on a PR

1. `gh auth status`
2. Resolve PR: `gh pr view --json number,url` (or user-provided number/URL).
3. `gh pr checks --json name,state,bucket,link,workflow`
4. For each failing check with a GitHub Actions `detailsUrl`, extract run id and run:
   - `gh run view <id> --json name,workflowName,conclusion,status,url`
   - `gh run view <id> --log`
5. If `detailsUrl` is external (Buildkite, etc.), report the URL only — do not scrape non-GitHub providers.
6. Summarize failure snippet; draft a fix plan; implement only after explicit approval.
7. Re-check: `gh pr checks`

For addressing PR **review comments** (not CI), use a dedicated review-comments skill if available — not this workflow.

## Authentication

- `gh auth status` — active host, user, env vars.
- `gh auth status --json` — machine-readable.

## Environment

- `NO_COLOR`, `CLICOLOR_FORCE`, `GH_FORCE_TTY` are honored.
- Set `GH_FORCE_TTY=1` only if you need TTY-style output inside an agent harness.

## When not to use

- Non-GitHub VCS (GitLab, Bitbucket, etc.).
- Full repository implementation without GitHub operations — use project-specific skills instead.
