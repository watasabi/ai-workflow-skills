# Change: TUI — rótulo de catálogo e substituição por nome

> Created: 2026-04-09
> Request: Mostrar o nome do catálogo na listagem da TUI; permitir várias linhas com o mesmo nome; ao instalar, substituir instalação existente do mesmo nome e deixar isso explícito na UI.

## Classification

| Field       | Value                                           |
|-------------|-------------------------------------------------|
| Type        | Enhancement                                     |
| Scope       | Medium                                        |
| Area        | types, cli, catalog load, installer, ui       |
| Status      | Verifying                                       |
| Phase       | 4 — Verify & Close                              |
| NEXT ACTION | `cargo clippy --all-targets --all-features -- -D warnings` |
| Updated     | 2026-04-09T00:00:00Z                           |

## Plan

1. `SkillInfo.catalog_label` + preenchimento em `load_catalog_skills`; merge sem deduplicar nomes.
2. `hashes_for_skills` / `install_skills`: associar hash ao caminho da skill (evitar colisão com nomes iguais).
3. `find_skill`: aviso se houver mais do que um match.
4. TUI: coluna catálogo; `apply_install` / update usam a skill da linha; remoção no âmbito antes de reinstalar quando substitui; rótulos de pending em PT.
5. Lista CLI: mostrar catálogo por skill.
6. Atualizar `.specs/codebase/ARCHITECTURE.md` (comportamento do merge).

## Result

| Check | Result |
|-------|--------|
| `cargo test` | OK |
| `cargo clippy ... -D warnings` | OK |
| `cargo fmt --check` | OK |

**Commits:** _(pendente — utilizador faz commit)_
