# Change: Multi-catalog audit fixes (C009–C015 + T020–T024)

> Created: 2026-04-10
> Request: Address all PENDING claims in `.specs/audits/claims.md` and `.specs/test-audits/claims.md` stemming from the multi-catalog diff. Stacks on top of uncommitted `tui-catalog-skill-label` work.

## Classification

| Field       | Value                                                                 |
|-------------|-----------------------------------------------------------------------|
| Type        | Enhancement + Bug Fix (follow-up to multi-catalog feature)            |
| Scope       | Large                                                                 |
| Area        | types, lockfile, installer, cli (find_skill / hashes_for_skills), ui (app, state, widgets), remote catalog label, tests |
| Status      | Verifying                                                             |
| Phase       | 4 — Verify & Close                                                    |
| NEXT ACTION | Run final `cargo fmt --check && cargo clippy -- -D warnings && cargo test`; present commit plan to user |
| Updated     | 2026-04-10T00:00:00Z                                                  |

## Scope — Items in this change

### Code (audits/claims.md)

| ID   | Title                                                                        | Priority |
|------|------------------------------------------------------------------------------|----------|
| C009 | Name collision between catalogs corrupts TUI state and overwrites install    | P0 (architectural) |
| C010 | `hashes_for_skills` lookup by `PathBuf` equality is a latent hazard          | P1 |
| C011 | `find_skill` ambiguity silently resolved by catalog order                    | P1 |
| C012 | `remove_skill_scope_before_install` discards concrete error detail           | P2 |
| C013 | `catalog_label` is runtime-only; installation loses provenance               | P0 (couples with C009) |
| C014 | `catalog_label` for remote catalogs is a SHA-256 hash prefix                 | P2 |
| C015 | Mixed PT/EN in user-facing strings                                           | P2 |

### Tests (test-audits/claims.md)

| ID   | Title                                                                                      |
|------|--------------------------------------------------------------------------------------------|
| T020 | Test for install/update with duplicated name across catalogs                               |
| T021 | Test for ambiguity warning in `find_skill`                                                 |
| T022 | Test for `hashes_for_skills` key robustness (non-canonical paths / symlinks)               |
| T023 | Assert per-row catalog label in `cli_list_two_catalogs_duplicate_skill_names_lists_both`   |
| T024 | Test `SkillInfo.catalog_label` via real path (`load_catalog_skills`), not `discover_skills` |

## Phase log

### Phase 0 — Intake & Classification (complete 2026-04-10)
- Scope confirmed with user: FULL (include C009/C013 architectural work).
- Stacking strategy: extend current uncommitted tree; commits will be atomic on top.

### Phase 1 — Research & Impact (in progress)
_Output: `impact.md`_
