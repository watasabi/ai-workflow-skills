# tcr_community reference map

Reference implementation: `tcr_community` (TCR/UTI cohort, 205 patients).

Use as a template when bootstrapping similar nursing/health projects.

## Key modules

| Path | Role |
|------|------|
| `src/tcr_community/schemas/columns.py` | `COL_*`, `ASSOCIATION_PREDICTORS`, demographic/clinical column lists |
| `src/tcr_community/cleaning/standardize.py` | Gender, dates, SIM/NAO, `desfecho_padronizado` |
| `src/tcr_community/pipeline/prepare_data.py` | Excel → parquet |
| `src/tcr_community/stats/descriptive.py` | `frequency_table`, `export_report` |
| `src/tcr_community/stats/association.py` | `run_association_battery`, `phik_association_matrix` |
| `src/tcr_community/stats/clinical_summaries.py` | Diagnosis categories, interventions, APACHE bands |
| `reports/export_clinical_summaries.py` | Orchestrates CSV + figure export |
| `reports/generate_profile_report.py` | Dual HTML themes, PhiK, figure gallery |

## Clinical summary exports (example)

| CSV | Function |
|-----|----------|
| `clin_diagnostico_por_categoria.csv` | `diagnosis_by_category` |
| `clin_comorbidades_top10.csv` | `comorbidity_frequency` |
| `clin_intervencoes.csv` | `interventions_table` |
| `clin_apache_mortalidade_faixas.csv` | `apache_mortality_table` |
| `clin_ventilacao_mecanica_resumo.csv` | `ventilacao_mecanica_table` |

## Association predictors (example)

```python
ASSOCIATION_PREDICTORS = [
    "comorbidades",
    "ventilacao_mecanica",
    "drogas_vasoativas",
    "transfusao_sanguinea",
    "escala_apache_ii",      # numeric → Kruskal-Wallis
    "hemodialise",
    "procedimento_cirurgico",
    "cuidados_paliativos",
]
```

Target: `desfecho_padronizado`.

## Commands

```bash
uv run pytest tests/
uv run python reports/export_clinical_summaries.py
uv run python reports/generate_profile_report.py
```

## Deliverables produced

- `reports/profile_report.html` — classic
- `reports/profile_report_minimal.html` — minimal
- `reports/association_phik_matrix.csv` + `figures/association_phik_matrix.png`
- ~24 figures auto-embedded in HTML figures section
- Companion guides:
  - `reports/association_results.md` — tests + p-value in plain language
  - `reports/descriptive/README.md` — CSV inventory + column meanings
  - `reports/figures/README.md` — PNG inventory + how to read charts

## Adapting to a new cohort

1. Replace column mapping in `columns.py` from new data dictionary.
2. Adjust `clinical_summaries.py` functions to local variables (e.g. SOFA instead of APACHE).
3. Update `CLINICAL_SUMMARY_FILES` and `FIGURE_CAPTIONS` in generator.
4. Revise companion markdown (`association_results.md`, `descriptive/README.md`, `figures/README.md`) for the new outcome and file list.
5. Keep the **reports/** artifact contract stable so HTML generator stays reusable.
