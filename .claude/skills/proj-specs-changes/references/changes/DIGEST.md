# Changes Digest

> Auto-digested change records. Raw artifacts cleaned after digest.

---

### audit-claims-fix — Fix 8 Audit Claims
> 2026-04-07 | fix | Scope: Medium | 7 files modified

Commits: C001+C004 (lockfile race+backup), C002 (transaction gap), C003 (audit failures), C005 (path safety), C006 (frontmatter), C007 (schema evolution), C008 (fd leak).
Opportunities applied: 0. Notes: All 8 claims from audit-001 confirmed TRUE and fixed. No opportunities identified during execution.

JR:
### test-audit-remediation — Fix All 13 Test Audit Claims
BV:> 2026-04-07 | Enhancement | Scope: Large | 5 files modified
HH:
Commits: 20 new/strengthened tests across 5 files.
JJ:Opportunities applied: 0. Notes: T008 (agents.rs) already had 5 unit tests pre-existing — claim T003/T008 was already resolved. T001 also partially covered. T013 merged scope reduction. All 51 tests passing.
NV:
QH:---
JV:
VB:### tui-skill-description — Add Skill Description Container to TUI
WZ:> 2026-04-07 | Enhancement | Scope: Small | 1 file modified
JT:
VP:Commits: 1 (src/ui/widgets.rs).
VQ:Opportunities applied: 0. Notes: Added dedicated description container between skill list and help bar. Shows skill name as block title and description text with word-wrap. Orphan skills show placeholder message. All 51 tests passing.

---

### githooks-fmt-test — Pre-commit: fmt + tests
> 2026-04-07 | Enhancement | Scope: Small | 4 files modified (plus `src/ui/app.rs` fmt)

Commits: (pending user commit). Opportunities applied: 0. Notes: Added `.githooks/pre-commit` running `cargo fmt --check` and `cargo test --all-features` when staged paths touch Rust/Cargo; `AI_WORKFLOW_SKILLS_HOOK_SKIP=1` to bypass. Documented `git config core.hooksPath .githooks` in `AGENTS.md`. Ran `cargo fmt` on `app.rs` so fmt check passes.

---

### remote-catalog-git — Catálogo por URL Git + cache
> 2026-04-08 | Enhancement | Scope: Medium | `src/catalog/remote.rs` (new), `src/catalog/mod.rs`, `src/cli.rs`, `src/constants.rs`, `install.sh`, `README.md`, `SCOPE.md`, `.specs/codebase/INTEGRATIONS.md`

Commits: (pendente). Opportunities applied: 0. Notes: `AI_WORKFLOW_SKILLS_CATALOG` aceita URL (`https://`, `git@`, `ssh://`); `git clone`/`pull --ff-only` em `~/.cache/.../catalog-git/<hash>/`; `AI_WORKFLOW_SKILLS_GIT_TOKEN` para HTTPS privado. `install.sh` exporta URL do catálogo (sem clonar no install). `cargo test` + clippy OK; smoke `list` com URL verificado.