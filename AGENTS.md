# Repository Guidelines

> Guidelines for AI agents working with the **ai-workflow-skills** codebase.

**What it does:** CLI/TUI tool for installing and managing AI agent skills from a versioned local or Git catalog. Supports 20+ agents (Cursor, Claude Code, Copilot, Windsurf, etc.), local/global install via copy or symlink, lockfile tracking, audit logging, and an interactive terminal UI.

---

## Project Overview

| Property | Value |
|---|---|
| Language | Rust |
| Edition | 2021 |
| Crate type | Binary + Library |
| License | MIT OR Apache-2.0 |
| MSRV | 1.77 (not enforced in CI) |
| Current toolchain | 1.88 (see `Cargo.toml` `rust-version`) |

**Stack:** clap (CLI), ratatui + crossterm (TUI), serde + serde_json + serde_yaml (serialization), sha2 (hashing), dirs (paths), chrono (timestamps), walkdir (fs traversal), anyhow (errors), semver (version parsing), regex (sanitization).

---

## Architecture & Data Flow

```
main.rs → cli::run() → dispatch_command()
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
         catalog/         core/             ui/
       discover.rs      installer.rs       app.rs
       registry.rs      lockfile.rs        state.rs
                        audit.rs            widgets.rs
                        cache.rs
```

**Layers:**
1. **Entry** — `main.rs` → `cli::run(Cli::parse())`
2. **CLI** — clap parsing → `dispatch_command()` with subcommands
3. **Business** — catalog discovery, skill install/remove/update, lockfile management
4. **Infra** — path sanitization, project root detection, cache, audit logging

**Data flow:**
```
catalog path → discover_skills() → Vec<SkillInfo>
SkillInfo + InstallOptions → install_skills() → Vec<InstallResult>
Operations logged to .skill-lock.json (local) and ~/.ai-workflow-skills/audit.log (global)
```

**Key design decisions:**
- Fully synchronous (no async)
- anyhow::Result<T> for error propagation
- Lockfile v2 schema with unix `flock` for atomic writes
- Symlink-aware canonicalization for `is_path_safe()`
- TTY detection for interactive vs non-interactive mode

---

## Key Directories

| Directory | Purpose |
|---|---|
| `src/` | Core source: CLI, business logic, types, constants |
| `src/core/` | Installer, lockfile, audit log, cache |
| `src/catalog/` | Skill discovery, registry generation |
| `src/ui/` | Interactive TUI (ratatui-based) |
| `tests/` | Integration tests (assert_cmd, predicates, tempfile) |
| `.githooks/` | Local git hooks (`pre-commit`: fmt + tests; `commit-msg`: Conventional Commits) |

---

## Development Commands

### Build
```bash
cargo build --release        # Release binary
cargo build --locked        # Use Cargo.lock (CI default)
```

### Test
```bash
cargo test                  # Run all tests
cargo test -- --nocapture  # With stdout/stderr output
cargo test <name>          # Run specific test
```

### Lint & Format
```bash
cargo fmt --check          # Check formatting (CI gate)
cargo fmt                  # Auto-format
cargo clippy --all-targets --all-features -- -D warnings  # Lint (CI gate)
```

### Run
```bash
cargo run -- --help        # CLI help
cargo run -- list          # List skills from catalog
cargo run -- install <name> # Install skill
cargo run -- tui           # Launch interactive UI
```

### CI (GitHub Actions)
```bash
# File: .github/workflows/ci.yml
# On: push and pull_request to main/master
# Steps: cargo fmt --check, clippy -D warnings, cargo test --all-features (Rust 1.88.0)
```

---

## Code Conventions

### Error Handling
- **anyhow::Result<T>** throughout — use `?` for propagation
- Context via `.context()` or `.with_context(|| ...)` for actionable errors
- No custom error enums — anyhow handles everything

### Path Safety
```rust
// Always sanitize and validate before filesystem operations
use crate::sanitize::{is_path_safe, sanitize_name, to_slug};

let safe = is_path_safe(&target, &base)?;     // Symlink-aware canonicalization
let clean = sanitize_name(name);                // Remove dangerous chars
let slug = to_slug(name);                      // kebab-case conversion
```

### Lockfile
- Read with `read_skill_lock()` — returns `SkillLockFile` or empty default
- Write with `write_skill_lock()` — uses `flock` for atomicity
- Add entries with `add_skill_to_lock()`, remove with `remove_agent_from_lock()`

### Audit Logging
```rust
use crate::core::audit::{log_audit, read_audit_log};
log_audit(&action, &agent, &skill_name, &install_path)?;
```

### CLI Commands
All subcommands implemented in `src/cli.rs` via clap `Command` enum. Dispatch via `dispatch_command()`.

---

## Important Files

| File | Purpose |
|---|---|
| `src/main.rs` | Binary entry point |
| `src/lib.rs` | Library root — exports public API |
| `src/cli.rs` | CLI definition, subcommand dispatch |
| `src/types.rs` | Core data types: `SkillInfo`, `InstallOptions`, `SkillLockFile` |
| `src/constants.rs` | Path constants, env vars, branding |
| `src/sanitize.rs` | Path sanitization functions |
| `src/core/installer.rs` | Install/remove logic |
| `src/core/lockfile.rs` | Lockfile v2 read/write |
| `src/core/audit.rs` | Audit log (JSON lines in `~/.ai-workflow-skills/`) |
| `src/catalog/discover.rs` | Skill discovery from catalog |
| `src/catalog/registry.rs` | Registry JSON generation |
| `src/ui/app.rs` | TUI main loop (ratatui) |
| `Cargo.toml` | Dependencies, binary/lib config |
| `.github/workflows/ci.yml` | CI workflow |

---

## Testing

**Framework:** Standard Rust (`#[test]`) with:
- `assert_cmd` + `predicates` — CLI testing via `Command::cargo_bin()`
- `tempfile` — isolated temp directories for filesystem tests
- `serde_json` — fixture serialization

**Organization:**
- `tests/common/mod.rs` — Shared fixtures: `write_skill()`, `minimal_catalog_root()`, `project_with_marker()`
- `tests/integration.rs` — Sanitization, discovery, registry roundtrip
- `tests/cli_binary.rs` — Exit codes, help output, env var catalog path
- `tests/cli_lifecycle.rs` — Full install/list/update/remove lifecycle
- `tests/installer_lib.rs` — Direct library API tests

**Isolation:** Tests override `HOME` env var for filesystem isolation.

**CI gates:**
1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features`
4. (Optional) MSRV check locally if you support older compilers

---

## Runtime & Tooling

| Tool | Version | Purpose |
|---|---|---|
| Rust | 1.88+ (see `Cargo.toml`) | Compiler |
| cargo | bundled | Package manager |
| gh | optional | GitHub CLI (releases, PRs) |

**Binary targets:**
- `x86_64-unknown-linux-musl` (static, Linux)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)

---

## Git Conventions

**Commit format:** [Conventional Commits](https://www.conventionalcommits.org/)

```
<type>(<scope>)?(!)?: <description>

[optional body]
```

**Valid types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `imp`, `infra`, `chore`, `breaking`

**Examples:**
```bash
feat: add Windsurf agent support
fix(catalog): handle missing SKILL.md gracefully
chore: update dependencies
docs: update README
```

**Git hooks (optional):** point Git at this repo’s hooks directory:

```bash
git config core.hooksPath .githooks
```

- **`pre-commit`** — If staged files include `*.rs`, `*.toml`, or `Cargo.lock`, runs the same sequence as CI job `test:rust`: `cargo fetch --locked`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`. Commits that touch only docs or other paths skip this hook. To bypass: `AI_WORKFLOW_SKILLS_HOOK_SKIP=1 git commit …` or `git commit --no-verify`.
- **`commit-msg`** — Validates [Conventional Commits](https://www.conventionalcommits.org/) before the commit message is accepted.

**Release process:** Tagging and publishing binaries are defined by each fork. Typical pattern: merge to `main`, CI green, then `git tag -a vX.Y.Z -m "Release vX.Y.Z" && git push origin vX.Y.Z` and attach artifacts on GitHub Releases if applicable.
