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
4. Run CI parity before marking done (see **proj-specs-testing**).

## Active change folders (examples)

Browse `references/changes/` — includes e.g. `multi-catalog-audit-fixes`, `catalog-list`, `tui-catalog-skill-label`, `linux-install-script`.

## Sync

```bash
rm -rf catalog/skills/\(development\)/proj-specs-changes/references/changes
cp -r .specs/changes catalog/skills/\(development\)/proj-specs-changes/references/
```
