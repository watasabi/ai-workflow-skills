# Codebase Concerns

**Analyzed:** 2026-04-07

## Tech Debt

**Lockfile key is skill name only:**
- Issue: Two skills with the same name in different categories would overwrite each other in the lockfile
- Files: `src/types.rs:10-13`
- Impact: Data loss if two skills share a name but are in different categories
- Fix approach: Include category in lockfile key, or deduplicate names at catalog discovery

**SkillInfo.path is absolute:**
- Issue: `SkillInfo.path` stores the absolute path to the skill in the catalog. Relocating the catalog breaks all discovered skills.
- Files: `src/catalog/discover.rs:try_read_skill()`
- Impact: Skills fail to install after catalog directory is moved
- Fix approach: Store relative paths from catalog root, or re-discover after catalog path change

**AuditEntry has no schema version:**
- Issue: New struct fields are silently ignored on read; removed fields cause parse failures (skipped with warning)
- Files: `src/core/audit.rs:read_audit_log()`
- Impact: Evolution of AuditEntry may silently corrupt audit trail interpretation
- Fix approach: Add schema version field to AuditEntry and implement migration

**SKILL.md frontmatter parsed with regex:**
- Issue: Assumes specific YAML structure (`name:`, `description:` with exact formatting)
- Files: `src/catalog/discover.rs:parse_frontmatter()`
- Impact: Malformed frontmatter silently degrades (name/description defaults to folder name)
- Fix approach: Use serde_yaml for proper YAML parsing

## Security Considerations

**Partial concurrent write protection via `flock(2)`:**
- Multiple concurrent processes **writing** the same lockfile are serialized: `write_skill_lock()` uses `flock_exclusive()` around the read–modify–write cycle (`src/core/lockfile.rs`).
- **Reads** (`read_skill_lock()`) do not take a lock — a reader may still see a partially written file if it races with a writer. Mitigation would be shared locks on read, or a transactional store.
- Status: Write-side races mitigated; read–write overlap remains a theoretical edge case for local CLI usage.

**No transactional boundary between filesystem and lockfile:**
- Risk: If `lockfile::add_skill_to_lock()` fails after `copy_dir_all()` succeeds, the skill exists on disk but is not recorded in the lockfile
- Files: `src/core/installer.rs`, `src/core/lockfile.rs`
- Current mitigation: Backup file created before write (`lockfile.rs:write_skill_lock()`)
- Recommendation: Implement a two-phase commit, or detect and self-heal orphan skills on next run

**Path validation uses fallback when canonicalization fails:**
- Risk: If `dunce::canonicalize` fails, the fallback validation may not catch all traversal attempts
- Files: `src/sanitize.rs:is_path_safe()`
- Current mitigation: Fallback checks component prefix without resolving symlinks
- Recommendation: Fail closed (error) when canonicalization fails rather than falling back to weaker validation

## Performance Bottlenecks

**No obvious performance concerns for CLI workload:**
- This is a filesystem-based CLI tool with no database queries
- Skill discovery walks the catalog directory once per command
- SHA-256 hashing is done once per skill on install/update
- These are O(n) in number of files, acceptable for local skill catalogs

## Fragile Areas

**cli.rs handles many responsibilities:**
- Why fragile: `dispatch_command()` in `src/cli.rs` handles catalog resolution, agent detection, progress bars, and output formatting in addition to command dispatch. Large function with many responsibilities; difficult to test in isolation.
- Files: `src/cli.rs` (~450 lines)
- Safe modification guide: Extract catalog resolution and agent detection into separate functions before adding new commands
- Test coverage: Covered by integration tests in `tests/cli_binary.rs` and `tests/cli_lifecycle.rs`

**UI re-enters CLI command layer:**
- Why fragile: TTY mode calls `dispatch_command()` rather than calling `install_skills()` directly, creating a confusing call stack
- Files: `src/ui/app.rs:apply_install()`
- Safe modification guide: Have UI call `core/installer.rs` functions directly
- Test coverage: Limited — UI path has fewer tests than CLI path

## Dependencies at Risk

**sha2 (0.10):** Actively maintained (last release 2024). No known vulnerabilities. Low risk.

**semver (1):** Actively maintained. Low risk.

**ratatui (0.29) / crossterm (0.28):** Current `Cargo.toml` pins these versions. Actively maintained. Low risk.

**anyhow (1):** Very widely used, stable API. Low risk.

**clap (4):** Stable. Low risk.

**dirs (5):** Last release 2024. Low risk.

**walkdir (2):** Stable. Low risk.

**chrono (0.4):** No features enabled beyond `clock` and `std`. Low risk.

## Test Coverage Gaps

**CLI command dispatch not fully isolated:**
- What's untested: Individual command handlers in `dispatch_command()` are not tested in isolation
- Risk: Changes to command logic may not be caught by existing tests
- Priority: medium — integration tests cover the happy path

**Lockfile concurrency (residual):**
- What's untested: Stress/concurrent scenarios (multiple processes); behavior when readers overlap writers on the same file.
- Implementation note: Exclusive `flock` on **writes** reduces corruption risk from concurrent writers; tests do not assert this.
- Priority: low — unlikely in normal single-user CLI usage

**Error paths in installer:**
- What's untested: Failure scenarios (disk full, permission denied, catalog disappears mid-operation)
- Risk: Unhandled errors may cause unexpected behavior
- Priority: medium

**UI/TUI paths:**
- What's untested: Interactive TUI mode has limited test coverage
- Risk: UI bugs may not be caught
- Priority: low — UI is secondary to CLI

## Summary

This codebase has moderate tech debt concentrated in three areas:
1. **Data integrity**: No transactional boundaries between filesystem and lockfile operations
2. **Concurrency**: Lockfile **writes** are serialized with `flock(2)`; cross-process read–write overlap and stress scenarios remain untested
3. **Error handling**: Large functions with many responsibilities make error paths harder to test

The security model is reasonable for a local-only CLI tool. The primary risks are around data consistency rather than security vulnerabilities.

**No critical concerns identified.** The codebase appears well-structured for a CLI tool of its complexity.
