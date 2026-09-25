---
name: ds-change
description: Make a planned change in a Data Science project generated from ds-template-v2, keeping it simple. Creates one short change note in docs/changes/, implements the work in the right template folder (notebooks/, queries/, src/, pipe/, models/), verifies it, and records it in CHANGELOG.md. Use when adding or altering an EDA/processing/training notebook, a query, a feature, a model, or a pipeline step and you want a lightweight paper trail. Do NOT use for trivial typo fixes, or for the ai-workflow-skills repo itself (see proj-specs-changes).
---

# ds-change

The data-scientist version of **proj-specs-changes**: same idea (plan, do, verify, record), but **one small file per change**, no ceremony, and always aligned with the **ds-template-v2** layout (see **ds-template-workflow**).

## Principles

- **One file per change.** No plan/impact/verification split.
- **Simple over modular.** Linear notebook or script first; extract to `src/` only when reused (per the project's `AGENT.md`).
- **Follow the template.** Never invent new top-level folders. Put things where **ds-template-workflow** says.
- **Reproducible.** Fixed seeds, paths relative to project root, no hard-coded absolute paths.
- **Data is not code.** Never commit `data/` contents or `config/.env`; `data/raw/` is immutable.

## Change note

Create `docs/changes/YYYY-MM-DD-<slug>.md` (create `docs/changes/` if missing):

```markdown
# <Short title>

- Date: YYYY-MM-DD
- Status: planned | in progress | done
- Type: eda | data | features | model | pipeline | fix | docs

## Goal
One or two sentences: what question or problem this answers.

## Approach
Bullets. Say where each piece lives, e.g.
- Query: `queries/get_data/churn_base.sql`
- Notebook: `notebooks/get_data/01_churn_base.ipynb`
- Reusable code: `src/<package>/features.py` (only if reused)

## Data / model impact
- Datasets read / written (`data/raw|interim|processed/...`)
- Metric before -> after, and how it was measured (split, seed)
- Anything that breaks downstream (columns renamed, schema changed)

## Done when
- [ ] Runs top to bottom from a clean kernel / `uv run`
- [ ] `uv run ruff check` and `uv run ruff format` clean
- [ ] Tests pass (`uv run pytest`) if `src/` or `pipe/` changed
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
```

Keep it under one screen. Update `Status` and fill the metrics when closing.

## Workflow

1. **Scope.** Restate the goal; check `docs/changes/` and `CHANGELOG.md` for related recent work.
2. **Write the change note** (status `in progress`).
3. **Place the work** using the template map:

   | Work | Location |
   |---|---|
   | Exploration | `notebooks/eda/` |
   | Extraction | `notebooks/get_data/` + `queries/get_data/*.sql` |
   | Cleaning / features | `notebooks/processing/` |
   | Training | `notebooks/training/` |
   | Evaluation / experiments | `notebooks/modeling/` |
   | Validation | `notebooks/qa/` |
   | Reused code | `src/<package>/` |
   | Production step | `pipe/src/NN_name.py` |
   | Conflicting deps / legacy model | UV sub-project in `models/` |
   | Figures / reports | `reports/figures/`, `reports/` |

4. **Implement simply.** Type hints on functions, Google docstrings for public code, Ruff line length 79, imports from the project package.
5. **Verify.** Run the notebook/script end to end, then:
   ```bash
   uv run ruff check src/ tests/
   uv run ruff format src/ tests/
   uv run pytest tests/
   ```
   Record real numbers in the note, not expectations.
6. **Changelog (required).** Add an entry under `## [Unreleased]` in `CHANGELOG.md` (`Added`, `Changed`, `Fixed`, `Removed`). Create the file with a Keep a Changelog header if it is missing.
7. **Close.** Set note `Status: done`. Commit with Conventional Commits, e.g. `feat(notebooks): adiciona base de churn` (types: feat, fix, docs, style, refactor, perf, test, chore, infra, imp, breaking).

## When to skip the note

Skip `docs/changes/` (but still update `CHANGELOG.md` if user-visible) for one-line fixes, typo/docs edits, and dependency bumps. Use a note for anything that changes data, features, metrics, or pipeline behavior.

## Anti-patterns

- Multi-file ceremony (separate plan/impact/verification) for a small change
- Extracting classes/registries before a second real use
- Notebooks that only run in a specific kernel state or with absolute paths
- Reporting a metric without saying split, seed, and dataset version
- Editing `data/raw/` or committing data/`.env`
- Closing a change without a `CHANGELOG.md` entry
