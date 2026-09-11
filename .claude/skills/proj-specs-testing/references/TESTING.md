# TESTING.md — Testing Infrastructure

> **Analysis date:** 2026-04-07 (drift sync: 2026-04-09 — contagens e exemplo `cargo_bin`)

---

## 1. Test Framework

The project uses the **standard Rust testing ecosystem** with three external dev-dependencies:

| Tool | Version | Role |
|------|---------|------|
| `#[test]` | Built-in (`std`) | Test attribute for test functions |
| `assert_cmd::Command` | `dev-dependencies.assert_cmd = "2"` | Spawn and assert on CLI binary |
| `predicates::prelude::*` | `dev-dependencies.predicates = "3"` | Composable assertions on stdout/stderr |
| `tempfile` | `dev-dependencies.tempfile = "3"` | Ephemeral temp directories with RAII cleanup |

**Source:** `Cargo.toml` dev-dependencies block

No property-based testing (proptest), no fuzzing, no snapshot testing.

---

## 2. Test Organization

### Integration Tests (`tests/`)

All tests that require the compiled binary live under `tests/` per Rust convention:

| File | Tests | Scope |
|------|-------|-------|
| `tests/integration.rs` | 7 | Catalog discover, sanitize, path safety, registry single + merged |
| `tests/cli_binary.rs` | 17 | CLI smoke: help, list (multi-catálogo), generate-registry, cache/audit |
| `tests/cli_lifecycle.rs` | 9 | Lifecycle: install/update (incl. multi `--catalog`) → list → remove |
| `tests/installer_lib.rs` | 2 | Direct library calls: install + remove roundtrip, unknown agent |
| **Total** | **35** | |

**Source:** `tests/*.rs`

### Unit Tests (`src/`)

Inline `#[cfg(test)]` modules in source files. These are **white-box** tests that access internal module members:

| File | Test Count | Module Under Test |
|------|------------|-------------------|
| `src/agents.rs` | 5 | Agent detection logic, validation |
| `src/sanitize.rs` | 1 | Sanitize name function |
| `src/catalog/discover.rs` | 5 | Skill discovery, tag normalization |
| `src/core/audit.rs` | 2 | Audit log write/read |
| `src/core/cache.rs` | 3 | Cache path and clear operations |
| `src/core/lockfile.rs` | 4 | Lockfile read/write, migration, flock write path |
| `src/core/installer.rs` | 1 | `install_one` with unknown agent (error path) |
| **Total** | **21** | |

**Source:** `grep -n '#[cfg(test)]' src/**/*.rs`

### Test Architecture Pattern

| Pattern | Location | Approach |
|---------|----------|----------|
| **Black-box CLI** | `tests/cli_*.rs` | `Command::cargo_bin("ai-workflow-skills")` — binário definido em `Cargo.toml` (`[[bin]] name = "ai-workflow-skills"`) |
| **Black-box lib** | `tests/integration.rs`, `tests/installer_lib.rs` | Calls public API of `ai_workflow_skills` crate |
| **White-box unit** | `src/**/mod.rs` | `#[cfg(test)]` modules import internal paths like `use crate::catalog::…` |

---

## 3. Test Fixtures and Helpers

### `tests/common/mod.rs`

Provides three shared helpers:

**`write_skill(dir, name, category, skill_folder)`** — lines 9-28
- Creates a skill directory at `dir/(category)/skill_folder`
- Writes `SKILL.md` (with YAML frontmatter: name + description)
- Writes `skill.manifest.json` (with version + tags)
- Creates `scripts/noop.sh` for layout parity with real skills

**`minimal_catalog_root(catalog)`** — lines 31-33
- Builds a valid minimal catalog at `catalog/skills/(demo)/demo-skill/`
- Uses `write_skill()` to create a single `demo-skill` skill
- Used by every test that needs a catalog fixture

**`project_with_marker(project)`** — lines 37-39 [INFERRED]
- Writes a `package.json` marker file to `project/`
- Triggers project-root detection (`find_project_root`) in tests

**Source:** `tests/common/mod.rs`

### Temp Directory Strategy

All tests use `tempfile::tempdir()` for ephemeral directories. The `TempDir` RAII guard ensures:
- Automatic cleanup on drop (success or panic)
- No cross-test contamination
- No manual teardown code needed

Typical pattern (from `tests/integration.rs` lines 32-36):
```rust
let tmp = tempdir().unwrap();
let catalog = tmp.path().join("catalog");
common::minimal_catalog_root(&catalog);
let skills = tmp.path().join("skills");
let root = resolve_skills_root(tmp.path()).unwrap();
```

---

## 4. Test Patterns

### Black-Box CLI Pattern (assert_cmd)

From `tests/cli_binary.rs`:
```rust
Command::cargo_bin("ai-workflow-skills")
    .unwrap()
    .env("HOME", home.path())
    .args(["list", "--catalog", cat.path().to_str().unwrap()])
    .assert()
    .success()
    .stdout(predicate::str::contains("demo-skill"));
```

Key characteristics:
- `Command::cargo_bin()` compiles and spawns the binary from `src/main.rs`
- `HOME` env var overridden to isolate from user's real config
- `predicates::str::contains()` for output assertions
- `.failure()` used to assert non-zero exit codes (e.g., invalid tags)

### Black-Box Library Pattern

From `tests/integration.rs` lines 38-45:
```rust
let list = discover_skills(&root).unwrap();
assert_eq!(list.len(), 1);
assert_eq!(list[0].name, "demo-skill");
assert_eq!(list[0].catalog_version, "1.0.0");
assert_eq!(list[0].tags, vec!["demo".to_string(), "test".to_string()]);
```

### Roundtrip Pattern

`tests/installer_lib.rs` and `tests/cli_lifecycle.rs` use a **roundtrip pattern**: install → verify files → remove → verify gone.

From `tests/installer_lib.rs` lines 55-74:
```rust
let results = install_skills(&project, &home, &skills, &opts, &hashes, || ());
assert!(results[0].success, "{:?}", results[0].error);
assert!(installed.is_file());

remove_skills(...);
assert!(!installed.exists());
```

### TTY-Safe Testing

Tests that would prompt for input (interactive mode) are tested in **non-TTY mode** and assert on failure with messages about TTY/terminal:

From `tests/cli_lifecycle.rs` lines 26-36:
```rust
.assert()
.failure()
.stderr(
    predicate::str::contains("skill")
        .or(predicate::str::contains("TTY"))
        .or(predicate::str::contains("terminal")),
);
```

---

## 5. Running Tests

### All Tests
```bash
cargo test
```

### Integration Tests Only
```bash
cargo test --test '*'
```

### Unit Tests Only (from src/)
```bash
cargo test --lib
```

### Single Test File
```bash
cargo test --test integration
cargo test --test cli_binary
cargo test --test cli_lifecycle
cargo test --test installer_lib
```

### With Output
```bash
cargo test -- --nocapture
```

---

## 6. Coverage Situation

**No coverage instrumentation configured.** No `cargo-tarpaulin`, `llvm-cov`, `grcov`, or equivalent in `Cargo.toml`. CI (`.github/workflows/ci.yml`) runs `cargo test` but does not publish coverage artifacts.

**What is tested:**
- CLI binary smoke (`--help`, subcommands)
- Catalog discovery and registry generation
- Sanitization / path safety (traversal rejection)
- Full install → list → update → remove lifecycle
- Lockfile read/write
- Cache path and clear operations
- Audit log write/read
- Agent detection and validation

**What is NOT tested:**
- Interactive TUI (`ui::run`, ratatui) — sem PTY/snapshot nos testes; fluxos cobertos via CLI não interativa
- Network via git subprocess (catalog sync is tested via unit tests in `src/catalog/remote.rs`)
- Concurrent access / race conditions
- Large catalog performance
- Network (catalog is always local filesystem)

---

## 7. CI Configuration

**GitHub Actions** — `.github/workflows/ci.yml`:

- **Runner:** `ubuntu-latest`
- **Steps:** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`

Release artifacts and tags are defined by each fork (not duplicated here).

---

## 8. Summary

| Dimension | Status |
|-----------|--------|
| Framework | Rust built-in + assert_cmd + predicates + tempfile |
| Organization | Integration tests in `tests/`, unit tests inline in `src/` |
| Fixtures | `tests/common/mod.rs` with catalog and project helpers |
| CLI testing | Black-box via `Command::cargo_bin()` |
| Coverage | Not instrumented |
| CI/CD | GitHub Actions (fmt, clippy, test) |

**Test counts:** 35 integration tests (`tests/*.rs`) + 22 unit tests (`cargo test --lib`) = **57** tests total (`cargo test`).
