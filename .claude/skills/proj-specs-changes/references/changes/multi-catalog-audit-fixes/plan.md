# Plan — Multi-catalog audit fixes

Seven atomic commits. Each runnable independently with `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.

## Task 1 — C010 + T022: simplify install_skills hash plumbing
**Files:** `src/core/installer.rs`, `src/cli.rs`, `src/ui/app.rs`, `tests/installer_lib.rs`, `tests/cli_lifecycle.rs`
- Drop `content_hashes: &[(PathBuf, String)]` param from `install_skills`; compute hash inline via `files_and_hash(&skill.path)`.
- Delete `cli::hashes_for_skills` and `install_with_progress_bar`'s `hashes` param.
- Update `dispatch_command::Install`, `dispatch_command::Update`, `ui::app::apply_pending` (Update branch), `ui::app::apply_install`, and `tests/installer_lib.rs` to match new signature.
- T022 regression: add `cli_install_with_relative_catalog_path` in `cli_lifecycle.rs` — pass catalog as `./<name>` (relative to cwd) and assert install succeeds and lock has non-None `content_hash`.
**Commit:** `refactor(installer): compute content hashes inline (fix C010)`

## Task 2 — C014 + C013: real catalog labels + persist in lockfile (schema v3)
**Files:** `src/types.rs`, `src/core/lockfile.rs`, `src/core/installer.rs`, `src/cli.rs`, `src/catalog/remote.rs` (possibly), `tests/installer_lib.rs`
- `types.rs`: `LOCKFILE_VERSION = 3`; `SkillLockEntry.catalog_label: Option<String>` (skip_serializing_if none).
- `lockfile.rs::migrate`: v<3 → set `catalog_label = None` explicitly (no-op; just bump version field). Roundtrip test.
- `AddSkillOpts.catalog_label: Option<String>`.
- `installer::install_skills`: plumb `skill.catalog_label` (if non-empty) into `AddSkillOpts`.
- `cli.rs::resolve_all_catalogs` returns `Vec<(PathBuf, String, Option<String>)>`: (resolved_path, human_label, remote_url). For remotes: label = last path segment of URL stripped of `.git`; for local: parent `file_name()`.
- `cli.rs::load_catalog_skills` takes `(catalog_root: &Path, label: &str)`; `load_merged_catalog_skills` loops with labels. Drop `catalog_label_for_root`.
- `cli.rs::dispatch_command::List`: add `(catálogo: X)` to `--installed` output, reading from `SkillLockEntry.catalog_label`.
- Update call sites in TUI + CLI to pass labels through.
**Commit:** `feat(lockfile): record catalog provenance (v3) + human labels for remote catalogs (C013, C014)`

## Task 3 — C009 + T020 + T023: TUI row↔lock join by (name, catalog_label)
**Files:** `src/ui/state.rs`, `src/ui/app.rs`, `tests/cli_binary.rs`, `tests/cli_lifecycle.rs`, `src/ui/state.rs` unit tests
- `build_rows`: for each catalog skill, attach `local`/`global` iff lock entry's `catalog_label` matches (or is None → legacy match to first encountered row). Track "claimed" names so legacy-match only fires once.
- Orphan computation: entries in lock whose `(name, catalog_label)` did not match any row.
- `apply_install`: on replacement across catalogs (lock had entry with different `catalog_label`), set flash message `substituído catálogo {prev} → {novo}`.
- T020: two catalogs, same name, different content. Install from cat1; lock has `catalog_label="cat1"`. Install `-s cat1/demo-skill` (after task 4 lands) or use `-s demo-skill` with only one catalog first time; then repeat with cat2. Assert lock flipped and disk bytes match cat2.
- T023: in `cli_list_two_catalogs_duplicate_skill_names_lists_both`, assert presence of both catalog labels on distinct lines (e.g. each `tmpdir` last segment name appears).
- Unit test for `build_rows` covering: (a) name unique across catalogs, (b) name collision with lock pointing at cat1, (c) legacy entry (catalog_label=None) claims first row.
**Commit:** `fix(tui): join skill rows to lock by (name, catalog_label) (C009)`

## Task 4 — C011 + T021: find_skill disambiguation
**Files:** `src/cli.rs`, `tests/cli_lifecycle.rs`
- `find_skill`: if `name` contains `/`, split once: `(prefix, suffix)`; require `catalog_label == prefix` AND name match on suffix.
- Without prefix: if >1 match, return `Err(anyhow!("..."))` listing candidates as `cat/name`. Signature changes to `Result<&SkillInfo>`; call sites handle the error (current behavior prints + continues; change to return the error up).
- T021: two catalogs same name; `install -s demo-skill` → fails with stderr listing candidates; `install -s <cat1-label>/demo-skill` → succeeds.
**Commit:** `feat(cli): disambiguate skill via cat/name syntax; hard-error on ambiguity (C011)`

## Task 5 — C012: propagate error detail
**Files:** `src/ui/app.rs`
- `remove_skill_scope_before_install`: collect `r.error` strings and join like `apply_install:477-483`.
**Commit:** `fix(tui): surface concrete error when removing before install (C012)`

## Task 6 — C015: PT/EN normalization
**Files:** `src/ui/app.rs`, `src/ui/widgets.rs`
- Translate: `"Working…"`→`"A trabalhar…"`; `"Error:"`→`"Erro:"`; `"nothing pending"`→`"nada pendente"`; `"already in sync"`→`"já sincronizado"`; `"not installed"`→`"não instalado"`; `"not in catalog: only remove (r)"`→`"fora do catálogo: só remover (r)"`; `"nothing to remove"`→`"nada para remover"`; `"skill not in catalog"`→`"skill fora do catálogo"`; `"failed to remove skill"`→`"falha ao remover skill"`.
- Help bar: `navigate`→`navegar`, `install (P)`→`instalar (P)`, `install (G)`→`instalar (G)`, `update`→`atualizar`, `remove`→`remover`, `clear`→`limpar`, `apply`→`aplicar`, `quit`→`sair`.
**Commit:** `style(tui): normalizar strings de UI para português (C015)`

## Task 7 — T024 + specs + ledger
**Files:** `tests/integration.rs`, `tests/cli_lifecycle.rs` (or unit test in `cli.rs`), `.specs/codebase/ARCHITECTURE.md`, `.specs/codebase/CONVENTIONS.md`, `.specs/audits/claims.md`, `.specs/test-audits/claims.md`
- Remove the `is_empty()` assertion in `integration.rs` that exercises `discover_skills` and claims `catalog_label` is empty there. Replace with a unit test for `load_catalog_skills` in `cli.rs` asserting `catalog_label == "<label>"` for every returned skill.
- ARCHITECTURE.md: note lockfile v3 + provenance invariant + "single physical install per name".
- CONVENTIONS.md: confirm PT-only UI strings.
- Claim ledgers: mark C009–C015 and T020–T024 RESOLVED with today's date.
**Commit:** `docs(specs): mark multi-catalog audit claims resolved + update architecture`

## Verification after each task

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Risks

- **Lockfile v3 migration:** existing users have v2 lockfiles in `.agents/.skill-lock.json`. Legacy entries lose provenance but keep working. No downgrade path (new field is optional, so v3 can still be deserialized as v2 after manual edit).
- **`find_skill` signature change:** current callers expect `Option`. Must update 4 call sites and add error paths.
- **TUI join change is the hottest diff**: requires careful unit testing. Invariant: every lock entry is reachable from exactly one row OR shows as orphan.
