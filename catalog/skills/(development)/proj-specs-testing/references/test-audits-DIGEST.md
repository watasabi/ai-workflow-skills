# Test audit digest — ai-workflow-skills

> Condensed history from completed runs (raw `runs/*` cleaned after digest).

### Run 20260407-a

> 2026-04-07 | Tests: 5

Claims: 5 new (Coverage: 4, Quality: 1, Necessity: 0, Flakiness: 0).

Observations: Suíte mínima e estável (`tempfile`); maior lacuna é ausência de testes para `cli::run`, módulos `core` (installer, lockfile, audit, cache) e `agents`; `normalize_tag_filter` / fluxo `list --tag` sem cobertura; teste `path_safe_under_base` só valida caso seguro de `is_path_safe`. Detalhes e evidências: `claims.md`.

TH:
### Run 20260407-b

> 2026-04-07 | Tests: 13

Claims: 8 new (Coverage: 6, Quality: 2, Necessity: 0, Flakiness: 0).

Observations: Continuação da auditoria. Lacunas confirmadas em subcomandos `update`/`remove` sem teste dedicado (T006-T007); `agents.rs` sem qualquer cobertura (T008); `cache --clear`/`--clear-registry` (T009) e `audit` principal (T010) sem teste; `list --tag` com zero results não verificado (T011); asserções fracas em roundtrip tests — string search sem validação de efeito (T012) e `.failure()` sem validação de mensagem (T013). TTY mode, `run()`, `run_update_batch()` sem cobertura. 5 PENDING de 20260407-a continuam sem resolução. Detalhes e evidências: `claims.md`.

### Run 20260409-a

> 2026-04-09 | Tests: 4 ficheiros de integração (32 testes em `tests/*.rs`)

Claims: 6 new (Coverage: 4, Quality: 2, Necessity: 1, Flakiness: 0).

Observations: Suíte sólida para CLI; novos gaps após multi-catálogo em `list`: `generate_registry_merged` e `install`/`update` multi-catálogo sem teste; TUI continua sem cobertura; teste `cli_audit_limit_shows_n_entries` sobrescreve o log (comentário incorreto); `cli_update_catalog_unchanged_is_noop` com asserção fraca; `TESTING.md` desatualizado em contagens e nome do binário em exemplo. Evidências: `claims.md` T014–T019.