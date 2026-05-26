---
name: proj-specs-debug
description: Debugging and risk specs for ai-workflow-skills (.specs/codebase/CONCERNS.md). Use when investigating bugs, race conditions, lockfile issues, path safety, silent frontmatter failures, or tech debt. Do NOT use for greenfield features without symptoms.
---

# Project debug & concerns

Live source: `.specs/codebase/CONCERNS.md`

## Reference

- [CONCERNS.md](references/CONCERNS.md) — tech debt, security, concurrency, installer/lockfile gaps, frontmatter regex limits

## Common investigation paths

| Symptom | Start in CONCERNS / code |
|---------|--------------------------|
| Skill on disk but not in lockfile | Installer vs lockfile transactional gap → `src/core/installer.rs` |
| Duplicate skill names across catalogs | Lockfile key + `find_skill` ambiguity |
| Wrong skill name/description | `parse_frontmatter` regex in `discover.rs` |
| Concurrent install corruption | `flock` on write; reads unlocked |
| Moved catalog path breaks install | `SkillInfo.path` absolute paths |

## Sync

```bash
cp .specs/codebase/CONCERNS.md catalog/skills/\(development\)/proj-specs-debug/references/
```
