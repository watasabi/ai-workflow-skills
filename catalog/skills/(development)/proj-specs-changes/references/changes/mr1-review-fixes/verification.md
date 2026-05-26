# Verification — MR !1 review fixes

## Checks

| Check | Result |
|-------|--------|
| `cargo fmt --check` | OK |
| `cargo clippy --all-targets --all-features -- -D warnings` | OK |
| `cargo test` | OK — 72 testes passam (incl. 1 novo unit + 1 regression) |

## New/updated tests

- `src/cli.rs::tests::find_skill_with_catalog_prefix_errors_on_duplicates` — unit test do ramo prefixed estrito (R2).
- `tests/cli_lifecycle.rs::cli_install_same_name_from_two_catalogs_flips_provenance` — estendido com `cat1_only.txt` regression (R3).

## What changed

| File | Change |
|------|--------|
| `src/ui/app.rs` | `apply_install` cleanup guard agora também dispara em cross-catalog replacement; `format!` flash normalizado para args nomeados (R1, R4). |
| `src/cli.rs` | `find_skill` ramo `catalogo/skill` passa a `collect`+match em `len()` como o ramo sem prefixo; Remove arm com `catalog: _` idiomático (R2, R5). Unit test novo para o ramo prefixado estrito. |
| `tests/cli_lifecycle.rs` | `cli_install_same_name_from_two_catalogs_flips_provenance` escreve `cat1_only.txt` antes do primeiro install e afirma que não sobrevive ao install de cat2 (R3). |

## Opportunities deferred (from review, out of scope)

- Fallback `["cursor"]` em `remove_skill_scope_before_install` — pre-existing, não tocado.
- Symlink-in-parent test para `is_path_safe` — sugestão de hardening, não é regressão.
- Docs drift em ARCHITECTURE.md — nada a atualizar; invariantes já documentados.

## Commits

_(pendente — próximo passo é criar o commit atómico com a mensagem do plan.md)_
