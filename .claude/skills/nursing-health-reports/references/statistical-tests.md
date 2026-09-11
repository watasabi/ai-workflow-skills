# Statistical tests reference

## Outcome variable

Standardize early (e.g. `desfecho_padronizado`: OBITO, ALTA, TRANSFERENCIA, OUTRO).

Document inclusion/exclusion and unknown labels in `association_results.md`.

## Bi-variate battery

```python
@dataclass
class AssociationResult:
    variable: str
    target: str
    test: str          # chi2 | fisher_exact | mannwhitneyu | kruskal | none
    statistic: float
    p_value: float
    n: int
    note: str | None
```

### Test selection

```
predictor numeric?
  yes → groups from outcome → 2 groups? Mann-Whitney : Kruskal-Wallis
  no  → crosstab → 2×2 with min cell < 5? Fisher : χ²
```

Constants: `FISHER_MIN_CELL = 5`, `MIN_GROUPS = 2`, α = 0.05 for reporting.

### Preprocessing

- Map clinical yes/no to `SIM`/`NAO` via `simplify_for_association()`
- Drop NA pairs before crosstab or group comparison
- Record `n` actually used per test

## PhiK matrix

Use when stakeholders expect a **correlation-style heatmap** with mixed types.

```python
from phik import phik_matrix

work = prepare_association_frame(df)  # predictors + outcome
matrix = phik_matrix(work, interval_cols=["escala_apache_ii"])
matrix.round(4).to_csv("reports/association_phik_matrix.csv")
```

Interpretation: values near 0 = weak association; near 1 = strong. Compare row/column vs outcome for ranking predictors. PhiK complements (does not replace) hypothesis tests.

Export static heatmap with seaborn (`cmap="Blues"`, `vmin=0`, `vmax=1`, annotated).

## Markdown interpretation template

File: `reports/association_results.md`

Required sections:

1. **Context** — N patients, target variable, source CSV path
2. **What we tested** — χ² for categories, Kruskal/Mann-Whitney for numeric
3. **Understanding p-value** — plain language, α = 0.05
4. **Null hypothesis** — no association
5. **Key findings** — bullet list of significant variables with approximate p
6. **Full results table** — variable | test | comparison | p-value | significant?
7. **How to use** — point to CSV, `figures/`, HTML generator command

Write for nurses/clinicians, not statisticians. Avoid jargon without one-line explanation.

Also maintain folder inventories:

- `reports/descriptive/README.md` — list every CSV + column meanings
- `reports/figures/README.md` — list every PNG + how to read charts

Full templates: [companion-markdown.md](companion-markdown.md).

## Descriptive tables

### Frequency

| valor | absoluta | relativa |
|-------|----------|----------|

`relativa` stored as fraction in CSV; format as `%` only in HTML (`f"{100*v:.1f}%"`).

### Numeric summary

| n | media | mediana | min | max | desvio_padrao |

Use `ddof=1` for sample std; handle n < 2 gracefully.

## Testing

pytest examples:

- Known 2×2 table → Fisher vs χ² branch
- Insufficient groups → `test="none"`, `p_value=1.0`
- PhiK matrix includes outcome column and is symmetric
