# Impact — Multi-catalog audit fixes

## Root-cause map

### C009 + C013 — same root cause
`SkillLockEntry` has no provenance field → `add_skill_to_lock` at `src/core/installer.rs:276-288` cannot record it → lockfile keyed by bare name → two SkillRows with same name share one lock entry in `src/ui/state.rs:173-183`.

**Physical constraint:** on-disk install path is `<agents-dir>/<sanitized-name>/`. Agents (Cursor, Claude Code, Windsurf) expect this flat layout — we cannot add a catalog prefix to the directory without breaking agent discovery. **Only one physical installation per skill name can exist.**

**Design decision:** lockfile stays keyed by bare `name`. Add `catalog_label: Option<String>` to `SkillLockEntry` for provenance. In the TUI, a row "owns" the lock entry **iff** `row.catalog_skill.catalog_label == lock_entry.catalog_label`. Rows for the same name from other catalogs show as not-installed. Installing them explicitly replaces the physical skill and updates the provenance.

Legacy entries (pre-v3) have `catalog_label = None` → match the first row we encounter for that name (migration grace).

### C010 — latent hazard from PathBuf join key
`hashes_for_skills` returns `Vec<(PathBuf, String)>`; `install_skills` joins by `p == &skill.path`. Currently safe because the same `SkillInfo` flows through. Simplest fix: **delete the parameter entirely**; compute the hash inside `install_skills` using `files_and_hash(&skill.path)`. No join, no canonicalization concern. Callers: `cli::dispatch_command` (Install, Update), `ui::app::{apply_pending, apply_install}` — all pass hashes straight through to the installer, no other consumer.

### C011 — find_skill ambiguity
Add disambiguation syntax `catalog/skill`. Parser splits on first `/`: prefix matches `catalog_label`, suffix matches name. Without prefix: if >1 match, hard error listing candidates (not a warning).

### C012 — error detail discarded
`remove_skill_scope_before_install` collapses any failure into a single literal. Pattern to copy: `apply_install:477-483` joins individual `r.error` strings.

### C013 — see C009.

### C014 — remote catalog label is hex hash
`catalog_label_for_root` at `src/cli.rs:170-176` uses `file_name()`, which for remotes is the 16-hex prefix under `catalog-git/`. Fix: change `resolve_all_catalogs` to return `(path, label, remote_url)`, computing a human label from the URL for remotes (last path segment stripped of `.git`). Plumb label into `load_catalog_skills` directly instead of re-deriving from the path.

### C015 — PT/EN mix
User-facing strings must be PT (CONVENTIONS.md §Portuguese UI strings). Audit: `ui/app.rs` flash messages (lines ~223, 246-247, 256, 275, 325-327, 420, 437), `ui/widgets.rs` help bar (114-129), description block (47).

## Tests — scoped

- **T020** — `cli_lifecycle.rs`: two catalogs with same name `demo-skill` but different file content → install from cat1 → verify lock has `catalog_label="cat1"` → install from cat2 → verify lock provenance flipped to `cat2` AND disk content matches cat2.
- **T021** — `cli_lifecycle.rs`: two catalogs with same name → `install -s demo-skill` → expect failure with candidate list; `install -s cat1/demo-skill` → succeed; lock provenance = cat1.
- **T022** — `cli_lifecycle.rs`: catalog passed with `./` prefix → still installs correctly (regression guard; also naturally covered once C010 removes the join).
- **T023** — `cli_binary.rs`: assert each of the two `demo-skill` rows shows its catalog label distinctly (not just count).
- **T024** — move `SkillInfo.catalog_label` exercise from `integration.rs` (discover_skills path) to `load_catalog_skills` call.

## Files touched

- `src/types.rs` — `SkillLockEntry.catalog_label`, bump `LOCKFILE_VERSION` to 3.
- `src/core/lockfile.rs` — migration v2→v3; `AddSkillOpts.catalog_label`; write/read roundtrip test; legacy match in `get_skill_from_lock`.
- `src/core/installer.rs` — drop `content_hashes` param; compute hash inline; plumb `catalog_label` from `SkillInfo` into `add_skill_to_lock`.
- `src/cli.rs` — `resolve_all_catalogs` returns (path, label, remote); `load_catalog_skills` takes label; `find_skill` disambiguation + hard error; `list --installed` shows catalog; drop `hashes_for_skills`; drop hashes from install/update call sites.
- `src/catalog/remote.rs` — (no change; label derived in `cli.rs`).
- `src/ui/state.rs` — `build_rows` joins by `(name, catalog_label)` with legacy-None grace.
- `src/ui/app.rs` — drop `hashes_for_skills` calls; normalize strings to PT; propagate error detail in `remove_skill_scope_before_install`.
- `src/ui/widgets.rs` — PT labels in help bar.
- `tests/cli_binary.rs` — T023 fix.
- `tests/cli_lifecycle.rs` — T020, T021, T022 new tests.
- `tests/integration.rs` — remove the wrong-path assertion (moved to cli_lifecycle or unit test).
- `tests/installer_lib.rs` — adapt existing tests to new `install_skills` signature.
- `.specs/codebase/ARCHITECTURE.md`, `.specs/codebase/CONVENTIONS.md` — note lockfile v3 + provenance invariant.
- `.specs/audits/claims.md`, `.specs/test-audits/claims.md` — mark C009–C015, T020–T024 RESOLVED.
