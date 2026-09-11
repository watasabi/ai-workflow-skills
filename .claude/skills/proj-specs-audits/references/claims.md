# Claims Ledger

> Persistent record of all audit claims across runs.
> Settled claims (RESOLVED, WONTFIX, DUPLICATE, INVALID) are never re-reported.

## Headers

| ID | Category | Status | Title | File:Line |
|----|----------|--------|-------|-----------|
| C001 | Logic | RESOLVED | Lockfile read-modify-write race condition | `src/core/lockfile.rs:66-88` |
| C002 | Logic | RESOLVED | Transactional boundary gap between filesystem and lockfile | `src/core/installer.rs:266` |
| C003 | Logic | RESOLVED | Audit log write failures silently ignored | `src/core/installer.rs:434` |
| C004 | Logic | RESOLVED | Lockfile backup failures silently ignored | `src/core/lockfile.rs:80-82` |
| C005 | Security | RESOLVED | Path safety fallback weakens security validation | `src/sanitize.rs:23-31` |
| C006 | Logic | RESOLVED | Frontmatter parsing with regex is fragile | `src/catalog/discover.rs:175-183` |
| C007 | Logic | RESOLVED | Audit log schema evolution silently loses data | `src/core/audit.rs:28-35` |
| C008 | Logic | RESOLVED | suppress_stdout leaks file descriptor on error paths | `src/ui/app.rs:25-62` |
| C009 | Logic | RESOLVED | Name collision between catalogs corrupts TUI state and overwrites installation | `src/core/lockfile.rs:169`, `src/sanitize.rs:66-68`, `src/ui/state.rs:173-183` |
| C010 | Logic | RESOLVED | `hashes_for_skills` lookup by `PathBuf` equality is a latent hazard | `src/cli.rs:210-218`, `src/core/installer.rs:268-271` |
| C011 | Logic | RESOLVED | `find_skill` ambiguity silently resolved by catalog order | `src/cli.rs:190-208` |
| C012 | Hygiene | RESOLVED | `remove_skill_scope_before_install` discards concrete error detail | `src/ui/app.rs:373-378` |
| C013 | Architecture | RESOLVED | `catalog_label` is runtime-only; installation loses provenance | `src/types.rs:33-35`, `src/core/lockfile.rs:169-189`, `src/core/installer.rs:276-288` |
| C014 | Hygiene | RESOLVED | `catalog_label` for remote catalogs is a SHA-256 hash prefix | `src/cli.rs:170-176`, `src/catalog/remote.rs:23-29` |
| C015 | Hygiene | RESOLVED | Mixed PT/EN in user-facing strings | `src/ui/app.rs`, `src/ui/widgets.rs` |

## Claims

### C001 — Lockfile read-modify-write race condition
- **File:** `src/core/lockfile.rs:66-88`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)
- **Confidence:** CONFIRMED
- **Impact:** Concurrent writes corrupt lockfile data

### C002 — Transactional boundary gap between filesystem and lockfile
- **File:** `src/core/installer.rs:266`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)
- **Impact:** Orphaned skills on partial failures

### C003 — Audit log write failures silently ignored
- **File:** `src/core/installer.rs:434`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)

### C004 — Lockfile backup failures silently ignored
- **File:** `src/core/lockfile.rs:80-82`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)

### C005 — Path safety fallback weakens security validation
- **File:** `src/sanitize.rs:23-31`
- **Category:** Security
- **Status:** RESOLVED (2026-04-07)

### C006 — Frontmatter parsing with regex is fragile
- **File:** `src/catalog/discover.rs:175-183`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)

### C007 — Audit log schema evolution silently loses data
- **File:** `src/core/audit.rs:28-35`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)

### C008 — suppress_stdout leaks file descriptor on error paths
- **File:** `src/ui/app.rs:25-62`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-07)

### C009 — Name collision between catalogs corrupts TUI state and overwrites installation
- **Files:** `src/core/lockfile.rs:169` (lock keyed by bare name), `src/sanitize.rs:66-68` (install path by name only), `src/ui/state.rs:173-183` (row↔lock joined by name; two rows for one slot)
- **Category:** Logic
- **Status:** RESOLVED (2026-04-10) — `SkillLockEntry.catalog_label` persists provenance (lockfile v3); `build_rows` now joins TUI rows by `(name, catalog_label)` with legacy-None grace; cross-catalog replacement surfaces as explicit flash `substituído: X → Y`. Single physical install per name preserved (matches agent disk-layout constraint). Covered by `tests/cli_lifecycle.rs::cli_install_same_name_from_two_catalogs_flips_provenance` and `src/ui/state.rs::tests::build_rows_*`.
- **Confidence:** CONFIRMED
- **Impact:** When two catalogs expose the same skill name (the scenario this diff adds support for), the second install silently overwrites the first on disk and in the lockfile. Both TUI rows then share a single lock entry; each reports its own `catalog_hash` against the shared `lock.content_hash`, producing oscillating "outdated" state (updating row A flips row B to outdated and vice versa). The merged-catalog UX is structurally inconsistent for collisions. See `runs/audit-002/findings/claims.md` C009.

### C010 — `hashes_for_skills` lookup by `PathBuf` equality is a latent hazard
- **Files:** `src/cli.rs:210-218`, `src/core/installer.rs:268-271`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-10) — `install_skills` now computes `files_and_hash(&skill.path)` inline; `hashes_for_skills` and the brittle `PathBuf` join are deleted. Regression guarded by `tests/cli_lifecycle.rs::cli_install_with_relative_catalog_path_records_hash`.
- **Confidence:** LIKELY

### C011 — `find_skill` ambiguity silently resolved by catalog order
- **File:** `src/cli.rs:190-208`
- **Category:** Logic
- **Status:** RESOLVED (2026-04-10) — `find_skill` returns `Result`; without prefix, >1 match produces a hard error listing candidates as `catalogo/skill`; `catalogo/skill` syntax resolves exactly. Covered by `tests/cli_lifecycle.rs::cli_install_ambiguous_name_errors_with_candidates` + unit tests in `src/cli.rs::tests::find_skill_*`.
- **Confidence:** CONFIRMED

### C012 — `remove_skill_scope_before_install` discards concrete error detail
- **File:** `src/ui/app.rs:373-378`
- **Category:** Hygiene
- **Status:** RESOLVED (2026-04-10) — concrete `r.error` strings are now joined into the returned error (mirroring the `apply_install:477-483` pattern).
- **Confidence:** CONFIRMED

### C013 — `catalog_label` is runtime-only; installation loses provenance
- **Files:** `src/types.rs:33-35`, `src/core/lockfile.rs:169-189`, `src/core/installer.rs:276-288`
- **Category:** Architecture
- **Status:** RESOLVED (2026-04-10) — `SkillLockEntry.catalog_label: Option<String>` added; `LOCKFILE_VERSION` bumped to 3 with a no-op migration for legacy entries; `install_skills` plumbs `SkillInfo.catalog_label` into `AddSkillOpts`; `list --installed` shows the catalog column.
- **Confidence:** CONFIRMED

### C014 — `catalog_label` for remote catalogs is a SHA-256 hash prefix
- **Files:** `src/cli.rs:170-176`, `src/catalog/remote.rs:23-29`
- **Category:** Hygiene
- **Status:** RESOLVED (2026-04-10) — `resolve_all_catalogs` now returns `ResolvedCatalog { path, label, remote }`. For remotes the label is derived from the URL's last path segment stripped of `.git`; for local it's the resolved `file_name()`. Unit-tested via `src/cli.rs::tests::remote_label_from_url_strips_dot_git`.
- **Confidence:** CONFIRMED

### C015 — Mixed PT/EN in user-facing strings
- **Files:** `src/ui/app.rs:275,325,327,376,420,437`; `src/ui/widgets.rs:114-120,221,226`
- **Category:** Hygiene
- **Status:** RESOLVED (2026-04-10) — flash messages and help bar normalized to Portuguese (matches CONVENTIONS.md §Portuguese UI strings).
- **Confidence:** CONFIRMED

---

_Last updated: 2026-04-10 (C009–C015 resolved — multi-catalog audit fixes)_
