---
name: proj-specs-audits
description: Audit ledger for ai-workflow-skills (.specs/audits/). Use when resolving audit claims, consolidation reviews, or closing PENDING items from codebase audits. Do NOT use for writing new tests (see proj-specs-testing).
---

# Project audit specs

Live source: `.specs/audits/`

## References

| Topic | File |
|-------|------|
| Open/resolved claims (C001…) | [claims.md](references/claims.md) |
| Consolidation map | [consolidation-map.md](references/consolidation-map.md) |
| Audit digest history | [DIGEST.md](references/DIGEST.md) |

## Workflow

1. Find your claim ID in `claims.md` (status PENDING / RESOLVED).
2. Implement fix with evidence paths cited in the claim.
3. Mark RESOLVED in `claims.md` when verified.
4. Cross-check **proj-specs-debug** for related systemic risks.

Test-specific claims: use **proj-specs-testing** (`test-audits/claims.md`).

## Sync

```bash
DEST=catalog/skills/\(development\)/proj-specs-audits/references
cp .specs/audits/{claims,consolidation-map,DIGEST}.md $DEST/
```
