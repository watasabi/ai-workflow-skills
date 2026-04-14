# Change: MR !1 review fixes — cross-catalog cleanup + find_skill strict + nits

> Created: 2026-04-13
> Request: Aplicar as correções levantadas no review da MR !1 (`feat/multi-catalog`).

## Classification

| Field       | Value                                                                 |
|-------------|-----------------------------------------------------------------------|
| Type        | Bug Fix + Enhancement                                                 |
| Scope       | Small                                                                 |
| Area        | `src/ui/app.rs`, `src/cli.rs`, `tests/cli_lifecycle.rs`                |
| Status      | Verifying                                                             |
| Phase       | 4 — Verify & Close                                                    |
| NEXT ACTION | Apresentar plano de commit ao utilizador; só commitar após confirmação |
| Updated     | 2026-04-13                                                            |

## Scope — Items in this change

| ID  | Severity | Title                                                                                    |
|-----|----------|------------------------------------------------------------------------------------------|
| R1  | 🔴       | Cross-catalog replacement in TUI doesn't clean stale files (skipped remove path)        |
| R2  | 🟡       | `find_skill` prefixed branch uses `next()` — doesn't detect duplicate `(label, name)`   |
| R3  | test     | Regression test: cat1-exclusive file must not persist after cat2 install                |
| R4  | 🟢 nit   | `apply_install` replacement flash: mixed positional/named args in `format!`             |
| R5  | 🟢 nit   | `src/cli.rs` Remove dispatch: `catalog: _catalog` → `catalog: _`                        |

## Out of scope

- Fallback `["cursor"]` in `remove_skill_scope_before_install` (pre-existing pattern; review note 🟡 3)
- Test com symlink na parent para `is_path_safe` (review suggestion, não é regressão)
- Architectural docs update (small scope — defer unless Phase 4 shows drift)
