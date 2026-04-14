# Test Audit Claims Ledger

> Append-only across runs. Never delete rows.

| ID    | Run        | Dimension   | Title                                                                 | File(s)                                      | Confidence | Status    |
|-------|------------|-------------|-----------------------------------------------------------------------|----------------------------------------------|------------|-----------|
| T001  | 20260407-a | Coverage    | Nenhum teste automatizado exercita `cli::run` nem os subcomandos CLI | `src/main.rs`, `src/cli.rs`                  | CONFIRMED  | RESOLVED  |
| T002  | 20260407-a | Coverage    | Módulos `core` (installer, lockfile, audit, cache) sem testes        | `src/core/*.rs`                              | CONFIRMED  | RESOLVED  |
| T003  | 20260407-a | Coverage    | API pública de `agents` (detecção, validação, defaults) sem testes   | `src/agents.rs`                              | CONFIRMED  | RESOLVED  |
| T004  | 20260407-a | Coverage    | `normalize_tag_filter` e fluxo `list --tag` sem cobertura de teste     | `src/catalog/discover.rs`, `src/cli.rs`      | CONFIRMED  | RESOLVED  |
| T005  | 20260407-a | Quality     | `path_safe_under_base` só cobre caso seguro; não nega path inseguro  | `tests/integration.rs`                       | CONFIRMED  | RESOLVED  |
| T006  | 20260407-b | Coverage    | Subcomando `update` sem teste dedicado — só exercitado como parte de install→update roundtrip | `src/cli.rs`, `tests/cli_lifecycle.rs`       | CONFIRMED  | RESOLVED  |
| T007  | 20260407-b | Coverage    | Subcomando `remove` sem teste dedicado — só como passo final de roundtrip         | `src/cli.rs`, `tests/cli_lifecycle.rs`       | CONFIRMED  | RESOLVED  |
| T008  | 20260407-b | Coverage    | Funções de `agents.rs` (detect, validate, defaults, get_def) sem cobertura de teste           | `src/agents.rs`                              | CONFIRMED  | RESOLVED  |
| T009  | 20260407-b | Coverage    | `cache --clear` e `cache --clear-registry` sem teste — só `--path` tem cobertura              | `src/cli.rs`, `tests/cli_binary.rs`           | CONFIRMED  | RESOLVED  |
| T010  | 20260407-b | Coverage    | Subcomando `audit` principal (sem `--path`) sem teste                                       | `src/cli.rs`, `tests/cli_binary.rs`           | CONFIRMED  | RESOLVED  |
| T011  | 20260407-b | Coverage    | `list --tag` com tag válida mas sem skills correspondentes — output não verificado            | `src/cli.rs`, `tests/cli_binary.rs`           | CONFIRMED  | RESOLVED  |
| T012  | 20260407-b | Quality     | `cli_update_after_catalog_hash_change` usa string search sem verificar re-instalação          | `tests/cli_lifecycle.rs`                       | CONFIRMED  | RESOLVED  |
| T013  | 20260407-b | Quality     | `cli_list_invalid_tag_fails` só verifica `.failure()` sem validar mensagem de erro            | `tests/cli_binary.rs`                         | CONFIRMED  | RESOLVED  |
| T014  | 20260409-a | Coverage    | `generate_registry_merged` não tem teste (só `generate_registry` em integração; CLI `generate-registry` só 1 catálogo) | `tests/integration.rs:68-76`, `tests/cli_binary.rs:147-167`, `src/catalog/registry.rs:228-246` | CONFIRMED | RESOLVED |
| T015  | 20260409-a | Coverage    | `install` e `update` via CLI não exercitam múltiplos `--catalog` (só `list` tem) | `tests/cli_lifecycle.rs` (todos os `args` com `--catalog`) | CONFIRMED | RESOLVED |
| T016  | 20260409-a | Coverage    | Modo TUI / `ui::run` sem teste automatizado | `src/ui/app.rs:36-73`, `.specs/codebase/TESTING.md` §6 | CONFIRMED | RESOLVED |
| T017  | 20260409-a | Quality     | `cli_audit_limit_shows_n_entries`: loop usa `fs::write` por iteração — ficheiro tem 1 linha; comentário diz 5 entradas; `-n 2` com várias linhas não é demonstrado | `tests/cli_binary.rs:299-328` | CONFIRMED | RESOLVED |
| T018  | 20260409-a | Quality     | `cli_update_catalog_unchanged_is_noop` asserção permissiva (`!stderr.contains("erro") \|\| success`) | `tests/cli_lifecycle.rs:242-252` | CONFIRMED | RESOLVED |
| T019  | 20260409-a | Necessity   | `TESTING.md` desatualizado: contagens (ex.: 14 vs 16 em cli_binary), total 30 vs 32, exemplo `cargo_bin("ai-workflow-skills")` vs binário real | `.specs/codebase/TESTING.md:32-36`, `TESTING.md:112-114` | CONFIRMED | RESOLVED |
| T020  | 20260410-a | Coverage    | Sem teste para `install`/`update` com nome duplicado entre catálogos (cenário central do diff multi-catálogo) | `tests/cli_binary.rs:54-80`, `tests/cli_lifecycle.rs` | CONFIRMED | RESOLVED |
| T021  | 20260410-a | Coverage    | Sem teste para aviso de ambiguidade em `find_skill` quando vários catálogos expõem o mesmo nome | `src/cli.rs:190-208`, `tests/cli_lifecycle.rs` | CONFIRMED | RESOLVED |
| T022  | 20260410-a | Coverage    | Sem teste para robustez da chave `PathBuf` em `hashes_for_skills` (paths não canonicalizados, symlinks, `./` relativos) | `tests/installer_lib.rs:27-33,88-94`, `src/core/installer.rs:268-271` | CONFIRMED | RESOLVED |
| T023  | 20260410-a | Quality     | `cli_list_two_catalogs_duplicate_skill_names_lists_both` só conta ocorrências; não valida que cada linha mostra o rótulo do catálogo de origem | `tests/cli_binary.rs:54-80` | CONFIRMED | RESOLVED |
| T024  | 20260410-a | Coverage    | Campo novo `SkillInfo.catalog_label` só é exercitado no caminho errado: `integration.rs` assere `is_empty()` via `discover_skills` (onde nunca é preenchido); caminho real `load_catalog_skills` não tem teste | `tests/integration.rs:67`, `src/cli.rs:170-176`, `src/catalog/discover.rs:181-184` | CONFIRMED | RESOLVED |

**T020–T024 resolved (2026-04-10) — multi-catalog audit fixes:**
- T020 → `tests/cli_lifecycle.rs::cli_install_same_name_from_two_catalogs_flips_provenance` asserts content_hash + catalog_label flip.
- T021 → `tests/cli_lifecycle.rs::cli_install_ambiguous_name_errors_with_candidates` asserts hard error with candidate list, then `cat/skill` resolves.
- T022 → `tests/cli_lifecycle.rs::cli_install_with_relative_catalog_path_records_hash` covers non-canonical catalog root; bug class eliminated since `install_skills` now computes hash inline (no `PathBuf` join).
- T023 → `cli_list_two_catalogs_duplicate_skill_names_lists_both` now asserts each catalog label appears on a distinct row.
- T024 → `src/cli.rs::tests::load_catalog_skills_sets_catalog_label` covers the real population path; `integration.rs` now documents that `discover_skills` is the low-level path that deliberately leaves the field empty.
