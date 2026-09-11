---
name: ds-template-workflow
description: Navigate and extend Data Science projects scaffolded from ds-template-v2 (cookiecutter). Covers the fixed folder layout (data/, notebooks/, queries/, pipe/, src/, reports/, models/), UV workspace + sub-projects, Ruff conventions (line-length 79), Conventional Commits, and where to put new notebooks/queries/pipeline steps. Use when working inside a project generated from ds-template-v2, or when scaffolding a new DS project by hand that should follow the same structure. Do NOT use for generic Python packages without this layout, or for ML-ops/orchestration platforms outside the pipe/ convention.
---

# ds-template-v2 workflow

Patterns for Data Science projects generated from **ds-template-v2** (cookiecutter, UV + Ruff).

## When to use

- Working inside a repo whose root has `AGENT.md` + `CLAUDE.md` pointing to ds-template-v2 conventions, or a `pyproject.toml` with a UV workspace and the `data/ notebooks/ queries/ pipe/` layout below
- Adding a notebook, a Databricks/SQL query, a pipeline step, or a reusable module and unsure where it belongs
- Setting up a new model artifact that needs isolated dependencies (legacy sklearn/xgboost versions)
- Reviewing code style / commit messages for a DS project on this template

Do **not** use for ML pipelines built on a different orchestration tool without adapting `pipe/`, for pure software-engineering repos, or for the health/nursing reporting deliverables covered by **nursing-health-reports** (that skill layers on top of a similar layout but is about clinical stats output, not general project navigation).

## Target layout

```
project/
├── config/                 # .env / .env.example (never commit .env)
├── data/
│   ├── raw/                # immutable originals
│   ├── external/           # third-party sources
│   ├── interim/            # intermediate transforms
│   └── processed/          # model-ready datasets
├── docs/                   # markdown docs, e.g. uv_subprojects.md
├── notebooks/
│   ├── get_data/           # extraction — pairs with queries/get_data/
│   ├── eda/                # exploratory analysis
│   ├── processing/         # cleaning / feature engineering
│   ├── training/           # model training
│   ├── modeling/           # experiments and evaluation
│   └── qa/                 # validation / quality assurance
├── queries/
│   └── get_data/           # .sql used by notebooks/get_data/
├── pipe/                   # production pipeline (orchestrator-agnostic)
│   ├── orchestrator.py     # e.g. Azure ML, Airflow entrypoint
│   ├── src/                # numbered steps: 01_load.py, 02_preprocess.py, ...
│   └── utils/              # pipeline-only helpers
├── models/                 # model artifacts; also home for UV sub-projects
├── reports/
│   └── figures/            # generated charts/images
├── scripts/                # bash utilities: setup.sh, lint.sh
├── src/<package>/          # reusable library code, imported as `<package>.x`
├── AGENT.md                # agent guidelines (source of truth, keep updated)
├── CLAUDE.md                # thin pointer to AGENT.md for Claude Code
├── .cursorrules            # thin pointer to AGENT.md for Cursor
└── pyproject.toml          # UV workspace root
```

## Deciding where new code goes

| You're adding... | Put it in |
|---|---|
| A one-off exploration | `notebooks/eda/` |
| A data extraction notebook | `notebooks/get_data/` + matching `.sql` in `queries/get_data/` |
| Feature engineering / cleaning notebook | `notebooks/processing/` |
| Model training notebook | `notebooks/training/` |
| Model comparison / evaluation | `notebooks/modeling/` |
| Pre-deploy checks | `notebooks/qa/` |
| Code reused across 2+ notebooks | `src/<package>/` — import it, don't copy-paste |
| A production pipeline step | `pipe/src/NN_name.py` (numbered, run in order by `pipe/orchestrator.py`) |
| A legacy model needing pinned/conflicting deps | new UV sub-project under `models/` (see below) |
| Generated chart/report | `reports/figures/` (or `reports/` for tables) |

## UV workspace & sub-projects

The root `pyproject.toml` is a UV workspace. Use sub-projects only when a model genuinely needs isolated/conflicting dependencies (e.g. pinned old `scikit-learn`/`xgboost`):

```bash
uv init models/modelo_legado_v1
cd models/modelo_legado_v1 && uv add scikit-learn==0.24.2 xgboost==1.5.0
uv run --package modelo_legado_v1 python predict.py
```

Root `pyproject.toml` picks these up via:

```toml
[tool.uv.workspace]
members = ["models/*"]
```

Details: [uv-subprojects.md](references/uv-subprojects.md).

## Code conventions

| Rule | Detail |
|------|--------|
| Runtime | UV + Python 3.12+ (project-defined version) |
| Types | Always add type hints to function signatures |
| Docstrings | Google style, for public APIs |
| Lint/format | Ruff, line length **79** |
| Style | **Prefer simple, readable code over modularity** — this is exploratory/analytical code, read far more than reused; don't extract a function/class/registry unless already reused or the file is hard to follow |
| Imports | Always `from <project_slug> import ...`, never relative hacks |

```toml
[tool.ruff]
line-length = 79

[tool.ruff.lint]
preview = true
select = ['I', 'F', 'E', 'W', 'PL', 'PT']
ignore = ['E402', 'F811']
```

```bash
uv run ruff check src/ tests/
uv run ruff format src/ tests/
uv run pytest tests/
```

## Commit convention

Conventional Commits are **required**. Allowed types: `feat, fix, docs, style, refactor, perf, test, chore, infra, imp, breaking`.

```
feat: adiciona modelo de classificação
fix(pipeline): corrige leitura de dados raw
refactor(src): simplifica feature engineering
```

## Agent guideline files

`AGENT.md` at project root is the single source of truth for agent behavior in a generated project; `CLAUDE.md` and `.cursorrules` are thin pointers to it. When conventions change, **update `AGENT.md`**, not the pointer files. If `AGENT.md` is missing or stale relative to this skill, treat `AGENT.md` as authoritative for that specific project — templates evolve per-repo after generation.

## Anti-patterns

- Notebooks with logic that's reused elsewhere and never promoted to `src/`
- Pipeline steps in `pipe/src/` that aren't numbered / aren't idempotent
- Committing `config/.env` (only `.env.example` is tracked)
- Adding a UV sub-project when the workspace's shared deps would do
- Extracting abstractions in exploratory code before a second real use case exists
- Line length or lint rules diverging from the `pyproject.toml` shown above without updating it

## Reference

- [uv-subprojects.md](references/uv-subprojects.md) — full sub-project workflow, mirrors `docs/uv_subprojects.md` in generated projects

## Sync

This skill mirrors conventions from `ds-template-v2`. When the template's `AGENT.md`, `README.md`, or `docs/uv_subprojects.md` change, update this SKILL.md and `references/uv-subprojects.md` to match.
