---
name: proj-specs-conventions
description: Coding conventions for ai-workflow-skills (.specs/codebase/CONVENTIONS.md). Use when writing Rust, CLI messages, errors (anyhow), tests layout, or matching project style. Do NOT use for architecture overview or debugging production incidents.
---

# Project conventions

Live source: `.specs/codebase/CONVENTIONS.md`

## Reference

- [CONVENTIONS.md](references/CONVENTIONS.md) — naming, `anyhow`, path safety, clap, testing layout, git hooks, module organization

## Quick reminders

- Errors: `anyhow::Result`, `.context()` / `.with_context()`
- Paths: `sanitize_name`, `is_path_safe` before filesystem ops
- Tests: unit tests inline `#[cfg(test)]`; integration in `tests/` with `assert_cmd`
- Commits: Conventional Commits; hooks via `git config core.hooksPath .githooks`

## Sync

```bash
cp .specs/codebase/CONVENTIONS.md catalog/skills/\(development\)/proj-specs-conventions/references/
```
