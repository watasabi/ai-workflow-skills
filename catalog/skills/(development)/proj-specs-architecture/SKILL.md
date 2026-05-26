---
name: proj-specs-architecture
description: Architecture and structure specs for ai-workflow-skills (.specs/codebase). Use when navigating modules, data flow, lockfile/install pipeline, stack, integrations, or project state. Read references before large refactors. Do NOT use for test-only or change-planning tasks.
---

# Project architecture specs

Live source: `.specs/codebase/` (prefer when in this repo).

## References

| Topic | File |
|-------|------|
| Module map, directory tree | [STRUCTURE.md](references/STRUCTURE.md) |
| Types, install flow, lockfile v3 | [ARCHITECTURE.md](references/ARCHITECTURE.md) |
| Rust edition, dependencies | [STACK.md](references/STACK.md) |
| Env vars, paths, agents | [INTEGRATIONS.md](references/INTEGRATIONS.md) |
| Recent drift / sync notes | [STATE.md](references/STATE.md) |
| Domain boundaries | [domain-map.md](references/domain-map.md) |

## Related skills

- **proj-specs-conventions** — Rust/CLI style
- **proj-specs-debug** — known bugs and risks
- **proj-specs-changes** — active change specs

## Sync bundled copies

```bash
SPECS=.specs CODE=catalog/skills/\(development\)/proj-specs-architecture/references
cp $SPECS/codebase/{STRUCTURE,ARCHITECTURE,STACK,INTEGRATIONS,STATE}.md $CODE/
cp $SPECS/audits/domain-map.md $CODE/
```
