# Change: lista de catálogos (múltiplos `--catalog` / env)

> Created: 2026-04-09T12:00:00-03:00
> Request: Cadastrar uma lista de catálogos em vez de apenas um por vez.

## Classification

| Field       | Value                                           |
|-------------|-------------------------------------------------|
| Type        | Enhancement                                     |
| Scope       | Medium                                          |
| Area        | CLI, catalog, UI, registry                      |
| Status      | Verifying                                       |
| Phase       | 4 — Verify & Close                              |
| NEXT ACTION | User confirma; então digest em DIGEST.md se desejado |
| Updated     | 2026-04-09T12:00:00-03:00                       |

## Impact

| File | What Changes | Why |
|------|-------------|-----|
| `src/constants.rs` | `ENV_CATALOGS` | Lista via variável de ambiente |
| `src/cli.rs` | `Vec` + merge + resolução em lote | Múltiplas entradas e skills unificadas |
| `src/catalog/registry.rs` | `generate_registry_merged` | `generate-registry` com N raízes |
| `src/ui/app.rs` | `run(..., &[PathBuf])` | TUI recarrega merge de N catálogos |
| `tests/cli_binary.rs` | Teste 2 catálogos | Regressão |

## Plan

1. Entrada: `--catalog` repetível; `AI_WORKFLOW_SKILLS_CATALOG` (um); `AI_WORKFLOW_SKILLS_CATALOGS` (vários, separador `|||` e/ou linhas).
2. Merge: ordem de entrada; primeiro catálogo ganha em nome de skill duplicado; aviso em stderr.
3. Comandos: list/install/update/generate-registry + TUI sem subcomando usam o merge.

## Result

| Check | Result |
|-------|--------|
| Tests pass | OK (`cargo test`) |
| Backward compat single `--catalog` / `AI_WORKFLOW_SKILLS_CATALOG` | OK (testes existentes) |

**Commits:** [pendente — implementação local]

