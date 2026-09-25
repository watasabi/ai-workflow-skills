---
name: ds-debug
description: Debug problems in Data Science projects generated from ds-template-v2. Symptom-driven checklist for wrong or suspicious metrics, data leakage, schema/column drift, non-reproducible results, notebooks that only work in a specific kernel state, UV/environment errors, Databricks/SQL query issues, and pipe/ step failures. Use when something is broken, results look too good or too bad, or a run differs from a previous one. Do NOT use for greenfield features (see ds-change) or for the ai-workflow-skills repo itself (see proj-specs-debug).
---

# ds-debug

The data-scientist version of **proj-specs-debug**: find the cause of a symptom with the fewest steps, in a project that follows **ds-template-v2** (see **ds-template-workflow**).

## Method

1. **Reproduce.** Restart the kernel and run top to bottom, or `uv run` the script. If it only fails in one state, that is the bug.
2. **Shrink.** Sample rows (`df.head(1000)` / `LIMIT 1000`), one split, one column. Keep the smallest input that still shows the symptom.
3. **Inspect, don't guess.** Print shapes, dtypes, null counts, value ranges, and the exact query/params used before changing code.
4. **Change one thing.** Re-run, compare against the previous output.
5. **Record it.** Fixes that change data, features, or metrics get a **ds-change** note and a `CHANGELOG.md` entry under `[Unreleased]` (`Fixed`).

## Symptom to first suspect

| Symptom | Check first |
|---|---|
| Metric too good (AUC ~1.0) | Target/leaky columns in features; split done after feature building; duplicates across train/test; time leakage (future data) |
| Metric worse than before | Different split/seed; changed `data/processed/` version; dropped rows in cleaning; class balance shift |
| Results differ between runs | Unset seeds (numpy, sklearn, xgboost, torch); unordered SQL without `ORDER BY`; parallel non-determinism |
| Works in notebook, fails in `pipe/` | Hidden state from earlier cells; absolute paths; imports not from the project package; missing deps in `pyproject.toml` |
| `KeyError` / missing column | Schema drift upstream; renamed column in `queries/`; stale file in `data/interim/` |
| Row count changed | Join fan-out or inner-join loss; filter added; dedup rule changed; timezone/date cutoff |
| Dtype / NaN surprises | CSV re-read losing dtypes (prefer parquet); mixed types; sentinel values (`-1`, `9999`) |
| `ModuleNotFoundError` / version conflict | Wrong env: use `uv run`, run `uv sync`; legacy model needing its own UV sub-project in `models/` |
| Ruff/pytest fails in CI only | Line length 79; Python version mismatch; tests reading local `data/` that is not committed |
| SQL/Databricks query wrong or slow | Run the `.sql` from `queries/get_data/` standalone; check filters, partitions, `LIMIT`; see **tool-databricks** |
| Pipeline step fails midway | Steps in `pipe/src/NN_*.py` must be idempotent; re-run only that step, inspect its input in `data/interim/` |
| Secrets/config errors | `config/.env` missing; copy from `config/.env.example` (`scripts/setup.sh`) |

## Data sanity checks

```python
df.shape, df.dtypes, df.isna().mean().sort_values(ascending=False).head(10)
df.duplicated(subset=key_cols).sum()          # duplicate keys
df[key_cols].nunique() / len(df)              # key uniqueness
set(train[key_col]) & set(test[key_col])      # overlap between splits
```

Leakage quick test: drop suspicious columns one at a time and watch the metric; check feature importance for an implausibly dominant feature.

## Rules while debugging

- Never edit `data/raw/`; reproduce from it.
- Do not "fix" by silently dropping rows or clipping values without recording it.
- Keep exploratory debug code in `notebooks/qa/` or `notebooks/eda/`, not in `src/`.
- Once the cause is a reusable check (e.g. schema validation), only then move it to `src/`.
- Do not commit data, outputs with sensitive values, or `config/.env`.

## Anti-patterns

- Changing several things between runs
- Trusting a metric without checking the split and leakage
- Debugging in a stale kernel state
- Fixing a symptom (clipping, filling NaN) instead of the upstream cause
- Leaving the fix undocumented: no note, no changelog entry
