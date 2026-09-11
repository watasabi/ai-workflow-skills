# Companion markdown (guides for non-statisticians)

Every artifact folder under `reports/` must ship a plain-language `.md` that:

1. **Lists** what files exist (inventory with one-line purpose each)
2. **Teaches** how to open, read, and interpret them
3. **Avoids** jargon without a one-line explanation

Audience: nurses, clinicians, students — not data scientists.

## Required companions

| Artifact area | File | Role |
|---------------|------|------|
| Association tests | `reports/association_results.md` | Explain χ² / Kruskal / p-value; key findings; full table |
| Descriptive CSVs | `reports/descriptive/README.md` | Inventory of CSVs + column meanings |
| Figures | `reports/figures/README.md` | Inventory of PNGs + how to use charts |
| Optional dashboard | `reports/dashboard/README.md` | How to run/build the isolated UI |

When you add a new CSV or PNG, **update the matching README in the same change**.

## Shared writing rules

- Portuguese (or the project’s stakeholder language), short sentences
- Prefer “o que significa” over formula dumps
- Always include: context (N, outcome), inventory, how to interpret, how to go deeper (HTML / other folders)
- Link related artifacts (`figures/` ↔ association md ↔ HTML generator command)

## Template: `reports/descriptive/README.md`

```markdown
# Relatórios Descritivos

Este diretório contém tabelas descritivas geradas a partir da base de pacientes.

## Arquivos incluídos

- `demo_genero.csv` — distribuição por gênero
- `demo_faixa_etaria.csv` — distribuição de faixa etária
- `clin_<variavel>.csv` — frequência / resumo de <variável clínica>
# … one bullet per file, keep in sync with disk …

## Como interpretar

- Cada CSV categórico contém `valor`, `absoluta` e `relativa`.
- A `relativa` é a proporção de cada categoria na amostra disponível
  (0–1 no arquivo; no HTML aparece como %).
- Para variáveis numéricas, os arquivos trazem `n`, `media`, `mediana`,
  `min`, `max` e `desvio_padrao`.

## Como navegar

1. Comece pelos `demo_*` (perfil da amostra).
2. Depois os `clin_*` de interesse clínico.
3. Para visão consolidada, abra `../profile_report_minimal.html`.
```

## Template: `reports/figures/README.md`

```markdown
# Gráficos exploratórios

Este diretório contém os gráficos gerados a partir do dataset processado.

## Gráficos disponíveis

- `sexo_bar.png` — distribuição por gênero
- `desfecho_pie.png` — composição de desfechos clínicos
- `obito_por_<fator>.png` — desfecho cruzado com <fator>
# … one bullet per PNG …

## Como usar / interpretar

- Abra as figuras no visualizador de imagens ou no relatório HTML.
- Barras/pizza mostram **frequências**; use junto com os CSVs em
  `../descriptive/` para os números exatos.
- Gráficos `obito_por_*` mostram padrões visuais de associação —
  confirme com `../association_results.md` (valor-p).
- A matriz PhiK (`association_phik_matrix.png`) resume associações
  0–1 entre variáveis mistas (quanto mais escuro/alto, mais forte).
```

## Template: `reports/association_results.md`

See also [statistical-tests.md](statistical-tests.md). Minimum sections:

1. Context (N, target, source CSV)
2. What we tested / why those tests
3. How to read p-value (α = 0.05) + null hypothesis
4. Key findings (bullets, plain language)
5. Full results table
6. How to use (CSV, figures/, HTML command)

## Workflow checklist (markdown)

```
- [ ] descriptive/README.md lists every clin_/demo_ CSV
- [ ] figures/README.md lists every non-empty PNG
- [ ] association_results.md updated after re-running tests
- [ ] Inventories match files on disk (no orphans, no missing bullets)
- [ ] Language reviewed for clinical stakeholders
```

## Anti-patterns

- Shipping CSVs/PNGs without a companion README
- README that only says “generated files” with no inventory
- Copy-pasting statistical jargon without explaining p-value
- Letting README drift when new exports are added
