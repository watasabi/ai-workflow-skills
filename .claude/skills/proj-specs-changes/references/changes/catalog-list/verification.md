# Verification — catalog-list

## Checks

| Check | Result |
|-------|--------|
| `cargo fmt --check` | OK |
| `cargo clippy --all-targets --all-features -- -D warnings` | OK |
| `cargo test --all-features` | OK |

## Summary

Múltiplos catálogos via `--catalog` repetido e `AI_WORKFLOW_SKILLS_CATALOGS` (`|||` ou linhas); `AI_WORKFLOW_SKILLS_CATALOG` continua com um único caminho/URL. Skills com o mesmo nome: primeiro catálogo na lista vence; duplicados seguintes geram aviso em stderr.
