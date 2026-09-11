---
name: nursing-health-reports
description: Scaffold nursing and health-research analytics with CSV exports, clinical figures, association tests (χ²/Fisher, Kruskal), PhiK heatmaps, dual HTML reports (classic + minimal), and plain-language companion READMEs that list and explain every artifact for non-statisticians. Use when starting or extending clinical cohort studies, UTI/nursing datasets, descriptive stats, or profile_report-style deliverables in Python (UV, pandas, pytest).
---

# Nursing & health research reports

Patterns distilled from **tcr_community** — reproducible clinical analytics with legible HTML deliverables **and** markdown guides for clinical stakeholders.

## When to use

- New cohort / chart-review project (enfermagem, UTI, epidemiologia clínica)
- Descriptive tables + figures + association tests against a standardized outcome
- HTML profile report for clinicians (not only notebooks)
- PhiK matrix for mixed categorical + numeric variables
- Need README/guides that teach lay readers how to navigate and interpret results

Do **not** use for ML modeling pipelines, FHIR/EHR integration, or non-Python stacks unless adapting the layout only.

## Target layout

```
project/
├── src/<package>/
│   ├── schemas/columns.py      # COL_* constants, predictors, demographic/clinical lists
│   ├── cleaning/standardize.py # normalize SIM/NAO, desfecho, dates
│   ├── pipeline/prepare_data.py # raw → parquet limpo
│   ├── stats/
│   │   ├── descriptive.py    # frequency_table, numeric_summary, export_report
│   │   ├── association.py    # chi2/fisher, kruskal, phik_association_matrix
│   │   └── clinical_summaries.py # domain tables (diagnóstico, intervenções, …)
│   └── io/loaders.py
├── reports/
│   ├── descriptive/
│   │   ├── README.md           # inventário + como interpretar CSVs
│   │   ├── clin_*.csv
│   │   └── demo_*.csv
│   ├── figures/
│   │   ├── README.md           # inventário + como usar gráficos
│   │   └── *.png
│   ├── association_results.csv
│   ├── association_results.md  # guia leigo: testes, p-valor, achados
│   ├── association_phik_matrix.csv
│   ├── export_clinical_summaries.py
│   └── generate_profile_report.py
├── notebooks/eda/
└── tests/
```

## Workflow (checklist)

```
- [ ] 1. Schema: define COL_* and ASSOCIATION_PREDICTORS in columns.py
- [ ] 2. Pipeline: ingest → clean → data/processed/*.parquet
- [ ] 3. Stats: pure functions → pd.DataFrame (no I/O inside stats/)
- [ ] 4. Export: CSVs to reports/descriptive/, PNGs to reports/figures/
- [ ] 5. Association: run battery → association_results.csv + PhiK matrix
- [ ] 6. Companion markdown (required — see below)
- [ ] 7. HTML: generate_profile_report.py → classic + minimal themes
- [ ] 8. Tests: pytest on summary/association helpers
```

Run end-to-end:

```bash
uv run python reports/export_clinical_summaries.py
uv run python reports/generate_profile_report.py
```

## Code conventions

| Rule | Detail |
|------|--------|
| Runtime | UV + Python 3.12+ |
| Types | All public functions typed; Google docstrings |
| Lint | Ruff, line length 79 |
| Commits | Conventional Commits (`feat(reports): …`) |
| Stats layer | Return DataFrames; export scripts write files |
| Column names | snake_case Portuguese clinical terms, stable across pipeline |

## Statistical patterns

### Descriptive

- Categorical → `valor`, `absoluta`, `relativa` (proportions 0–1 in CSV; format `%` only in HTML)
- Numeric → `n`, `media`, `mediana`, `min`, `max`, `desvio_padrao`
- Prefix exports: `demo_*` demographics, `clin_*` clinical aggregates

### Association vs outcome

| Predictor type | Test | Notes |
|----------------|------|-------|
| Categorical × categorical | χ² | Fisher exact if 2×2 and cell count < 5 |
| Numeric × categorical groups | Mann-Whitney (2 groups) / Kruskal-Wallis (3+) | Non-parametric default |
| Mixed matrix overview | **PhiK** (`phik` package) | 0–1 heatmap; mark `interval_cols` for numeric |

Battery output columns: `variable`, `target`, `test`, `statistic`, `p_value`, `n`, `note`.

Simplify categoricals before tests (`SIM`/`NAO`, standardized outcome labels).

### Clinical summaries

Implement **one function per table** in `clinical_summaries.py`:

```python
def interventions_table(df: pd.DataFrame) -> pd.DataFrame:
    """Suportes e intervenções com frequência absoluta/relativa."""
```

Group related exports in `export_clinical_summaries.py` via a `tables` dict + `export_report()`.

Figures: `@dataclass BarChartSpec` for horizontal bar charts; save to `reports/figures/` at dpi ≥120.

## Companion markdown (guides for leigos)

**Every report section must have a `.md` that lists contents and teaches interpretation.**
Artifacts without a guide are incomplete.

| File | Must include |
|------|----------------|
| `reports/descriptive/README.md` | Bullet list of every `demo_*` / `clin_*` CSV + how to read `valor`/`absoluta`/`relativa` (or numeric summary columns) |
| `reports/figures/README.md` | Bullet list of every PNG + how to use/read the charts; point to descriptive CSVs and association md |
| `reports/association_results.md` | Plain-language tests, p-value, null hypothesis, key findings, full results table, links to CSV/figures/HTML |

### Writing rules

- Audience: enfermagem / clínica — short sentences, Portuguese (or stakeholder language)
- Always: **inventory** (what exists) + **interpretation** (what it means) + **navigation** (what to open next)
- When adding a CSV or PNG, update the matching README in the **same change**
- Do not leave clinical meaning only in code comments or notebooks

Templates and checklist: [companion-markdown.md](references/companion-markdown.md).

## HTML report structure

Generate **two files** from one renderer:

| Output | Theme |
|--------|-------|
| `profile_report.html` | Classic (Arial, simple borders) |
| `profile_report_minimal.html` | Apple-inspired (system font, cards, TOC) |

### Sections (order)

1. **Associações** — PhiK heatmap (Plotly) + collapsible matrix table; `-log10(p)` bar chart from `association_results.csv`
2. **Análise descritiva clínica** — curated summary CSVs as HTML tables
3. **Figuras** — auto-include every non-empty `reports/figures/*.png` with captions map
4. **CSVs descritivos** — per-file `<details>` with preview, `describe()`, optional Plotly bar/histogram

### HTML rules

- Self-contained: inline CSS; Plotly via CDN
- `theme="classic"|"minimal"` parameter; do not overwrite old theme when adding new
- `FIGURE_CAPTIONS: dict[str, str]` for human-readable titles; fallback from filename
- Meta lines: source path + row counts
- Companion `.md` files remain the primary teaching layer; HTML is the interactive view

## Dependencies (typical)

```toml
pandas >= 2.2
numpy, scipy, matplotlib, seaborn, plotly, pyarrow, openpyxl, phik
dev: pytest, ruff
```

## Extending a new project

1. Copy layout skeleton; rename package under `src/`.
2. Map source columns → `COL_*` in `columns.py`.
3. List `ASSOCIATION_PREDICTORS` and `COL_DESFECHO_PADRONIZADO` (or equivalent outcome).
4. Port `export_report()`, `run_association_battery()`, `phik_association_matrix()`.
5. Customize `CLINICAL_SUMMARY_FILES` and figure specs for the domain.
6. Write/update companion READMEs (`descriptive/`, `figures/`, `association_results.md`).
7. Keep notebooks for exploration; **reports/** for published artifacts + guides.

## Anti-patterns

- Putting file writes inside `stats/` modules
- Only Jupyter outputs (no CSV/HTML for stakeholders)
- Hard-coding absolute paths (use `Path(__file__).resolve().parents[1]`)
- Replacing classic HTML when adding minimal theme
- Clinical interpretation only in code comments — always mirror in `.md`
- CSVs/PNGs without inventory README in the same folder
- README that lists nothing or drifts out of sync with files on disk

## Reference

- [report-layout.md](references/report-layout.md) — directory tree, HTML section IDs, CSS tokens
- [companion-markdown.md](references/companion-markdown.md) — layperson README templates + checklist
- [statistical-tests.md](references/statistical-tests.md) — test selection, PhiK, association md template
- [tcr-community-example.md](references/tcr-community-example.md) — concrete file map from reference repo

## Sync from reference repo

When updating patterns from tcr_community:

```bash
REF=/path/to/tcr_community
DEST=catalog/skills/\(research\)/nursing-health-reports/references
cp "$REF/reports/generate_profile_report.py" "$DEST/"  # review manually
```

Prefer summarizing patterns in references/ over copying full scripts into the skill.
