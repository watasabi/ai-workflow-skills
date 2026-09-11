# Report layout reference

## Artifact tree

```
reports/
├── association_results.csv       # bi-variate tests vs outcome
├── association_results.md          # lay guide: tests, p-value, findings
├── association_phik_matrix.csv     # symmetric PhiK matrix
├── profile_report.html             # classic theme
├── profile_report_minimal.html     # minimal / Apple-inspired theme
├── descriptive/
│   ├── README.md                   # inventory + how to interpret CSVs
│   ├── clin_*.csv                  # clinical aggregates
│   └── demo_*.csv                  # demographics
├── figures/
│   ├── README.md                   # inventory + how to use charts
│   └── *.png                       # static charts (HTML gallery)
├── export_clinical_summaries.py    # tables + key figures from parquet
└── generate_profile_report.py      # HTML from CSVs + figures
```

Companion `.md` files are **first-class deliverables**, not optional docs.
See [companion-markdown.md](companion-markdown.md).

## Generator responsibilities

| Script | Input | Output |
|--------|-------|--------|
| `export_clinical_summaries.py` | `data/processed/*.parquet`, raw Excel if needed | `descriptive/*.csv`, selected `figures/*.png` |
| `generate_profile_report.py` | everything under `reports/` | both HTML files, optional PhiK PNG |

`generate_profile_report.py` should call export first (or warn if stale).

## HTML section map (minimal theme)

| Section ID | TOC label | Content |
|------------|-----------|---------|
| `#associacao` | Associações | PhiK heatmap, p-value chart, association CSV preview |
| `#clinica` | Análise clínica | Curated `CLINICAL_SUMMARY_FILES` tables |
| `#figuras` | Figuras | All PNGs from `figures/` |
| `#descritivos` | CSVs descritivos | Every `descriptive/*.csv` with collapsible preview |

Classic theme uses the same sections without hero/TOC wrapper.

## Minimal CSS tokens

```css
:root {
  --bg: #f5f5f7;
  --surface: #ffffff;
  --text: #1d1d1f;
  --muted: #6e6e73;
  --line: rgba(0, 0, 0, 0.08);
  --accent: #0071e3;
  --radius: 16px;
}
```

Typography: `-apple-system, BlinkMacSystemFont, "SF Pro Text", …`

Layout: `.page` max-width ~920px; `.section` as white cards; `.figure-card` per image.

## Plotly integration

- Load CDN once per page: `plotly-latest.min.js`
- PhiK: `heatmap` trace, `zmin=0`, `zmax=1`, annotated cells
- p-values: horizontal bar, `-log10(p)`, color by significance (α=0.05)
- Descriptive CSVs: bar for low-cardinality numeric/categorical; histogram if >10 unique values

## Figure caption registry

Maintain `FIGURE_CAPTIONS: dict[str, str]` keyed by filename. Auto-discover PNGs with `sorted(FIGURES_DIR.glob("*.png"))`, skip zero-byte files.

New figures appear in HTML automatically after regeneration — no manual HTML edits.
Update `figures/README.md` inventory in the same change.
