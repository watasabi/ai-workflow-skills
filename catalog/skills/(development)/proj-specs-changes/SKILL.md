---
name: proj-specs-changes
description: Change specs for ai-workflow-skills (.specs/changes/). Use when implementing a planned change, reading CHANGE.md/plan.md/impact.md, or continuing multi-step work from the changes ledger. Do NOT use for ad-hoc edits without a change folder.
---

# Project change specs

Live source: `.specs/changes/` (authoritative; may be newer than bundled copies).

## Layout

```
.specs/changes/
  DIGEST.md              # Summarized history
  <change-id>/
    CHANGE.md            # Scope, status, claims
    plan.md              # Tasks (optional)
    impact.md            # Files / risk (optional)
    verification.md      # Verify checklist (optional)
```

Bundled mirror: [references/changes/](references/changes/)

## Workflow

1. Read `references/changes/DIGEST.md` for context on recent work.
2. Open the specific `<change-id>/CHANGE.md` for scope and status.
3. Follow `plan.md` if present; update `verification.md` when closing.
4. **Add an entry to `CHANGELOG.md`** under `## [Unreleased]` describing the change (see below) — do this in the same change, not as a follow-up.
5. Run CI parity before marking done (see **proj-specs-testing**).

## CHANGELOG.md

Root `CHANGELOG.md` ([Keep a Changelog](https://keepachangelog.com/) format, Semantic Versioning) is the **user-facing** record of what changed, separate from `.specs/changes/` (which is the internal planning ledger). Every change that a consumer of this CLI/catalog would care about — a new CLI flag, a new/changed skill, a bug fix, a breaking change — needs an entry under `## [Unreleased]`:

```markdown
## [Unreleased]

### Added
- `install --symlink` flag for linking instead of copying skills.

### Fixed
- Lockfile race when two installs run concurrently.
```

Categories: `Added`, `Changed`, `Fixed`, `Removed`, `Deprecated`, `Security`. On a version bump (`Cargo.toml` + release tag), move `[Unreleased]` entries under a new `## [x.y.z] - YYYY-MM-DD` heading. A `CHANGE.md` without a matching `CHANGELOG.md` entry is incomplete — the ledger explains *how*, the changelog tells users *what*.

## Active change folders (examples)

Browse `references/changes/` — includes e.g. `multi-catalog-audit-fixes`, `catalog-list`, `tui-catalog-skill-label`, `linux-install-script`.

## Sync

```bash
rm -rf catalog/skills/\(development\)/proj-specs-changes/references/changes
cp -r .specs/changes catalog/skills/\(development\)/proj-specs-changes/references/
```
