# Code Conventions — `ai-workflow-skills`

> Documented from code inspection of `src/cli.rs`, `src/core/installer.rs`, `src/types.rs`, `src/sanitize.rs`, `src/agents.rs`, `src/catalog/discover.rs`, `src/core/lockfile.rs`, `src/ui/app.rs`, `src/ui/widgets.rs`, `src/project_root.rs`, `src/catalog/remote.rs`, and `Cargo.toml`.
> All observations marked `[INFERRED]` are conclusions drawn from patterns, not from explicit config files.

## 1. Naming Conventions

**Rust idioms — compliant.**

| Category | Convention | Example |
|---|---|---|
| Functions / variables | `snake_case` | `install_skills`, `skill_target`, `content_hash`, `catalog_version` |
| Types / enums / structs | `PascalCase` | `SkillInfo`, `InstallMethod`, `InstallResult`, `AuditEntry` |
| Enum variants | `PascalCase` | `InstallMethod::Copy`, `InstallMethod::Symlink`, `PendingAction::InstallProject` |
| Constants | `SCREAMING_SNAKE_CASE` | `LOCKFILE_VERSION`, `AUDIT_LOG_FILE`, `AGENTS_DIR`, `CANONICAL_SKILLS_DIR` |
| Module names | `snake_case` | `core/`, `sanitize.rs`, `audit.rs`, `lockfile.rs` |
| Boolean variables | `is_` / `has_` / `used_` prefix or plain `success`/`removed` | `is_file()`, `success`, `symlink_failed`, `used_global_symlink`, `removed_local`, `removed_global` |
| Internal helpers | prefixed by operation or return type | `ok_result()`, `fail_result()`, `copy_dir_all()`, `symlink_skill()` |

File paths referenced:
- `src/sanitize.rs:1–43` — functions `sanitize_name`, `is_path_safe`, `skill_install_path`, `to_slug`
- `src/types.rs:1–93` — structs `SkillInfo`, `InstallOptions`, `InstallResult`, enum `InstallMethod`
- `src/core/installer.rs:109–215` — helper functions and field naming
- `src/agents.rs:1–200` — agent definitions with `id`, `display_name`, `skills_dir`, `global_skills_dir`, `detect`

---

## 2. Import Ordering

No `rustfmt.toml` or `clippy.toml` found in the project root. [INFERRED]

Import ordering observed in `src/cli.rs:1–16` and `src/core/installer.rs:1–16`:

```rust
// 1. std library imports first
use std::fs;
use std::path::{Path, PathBuf};
use std::io::IsTerminal;

// 2. Third-party crates
use anyhow::{anyhow, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

// 3. crate:: self imports (grouped by module prefix)
use crate::agents::{default_agents_for_install, detect_installed_agents, validate_agents};
use crate::catalog::{discover_skills, files_and_hash, generate_registry, normalize_tag_filter, resolve_skills_root};
use crate::core::lockfile;
```

Groups (in order):
1. `std` / core library imports
2. Third-party crates (`anyhow`, `clap`, `serde`, `regex`, etc.)
3. `crate::` self imports, grouped by module prefix

No `use super::*` imports observed. Explicit paths preferred. Within each group, alphabetical ordering appears to apply. [INFERRED]

File paths: `src/cli.rs:1–16`, `src/core/installer.rs:1–16`, `src/catalog/discover.rs:1–12`.

---

## 3. Error Handling

**`anyhow::Result<T>` for application logic; `std::io::Result<T>` for low-level I/O primitives.**

| Pattern | Evidence |
|---|---|
| `anyhow::Result<T>` as function return type | `src/cli.rs` — `gather_catalog_inputs`, `resolve_all_catalogs`, `load_catalog_skills`, `load_merged_catalog_skills` |
| `anyhow!("message")` for immediate errors | `src/cli.rs` — `"informe --catalog (uma ou mais vezes) ou…"` |
| `anyhow!("fmt {var}")` with formatting | `src/cli.rs` |
| `.context("description")` for wrapped errors | `src/cli.rs` — `.with_context(|| format!("catálogo inválido: {}", catalog.display()))` |
| `.with_context(|| format!(…))` with closure | `src/catalog/discover.rs:95` — file read errors |
| `anyhow::bail!("message")` for early exit with error | `src/catalog/discover.rs:128`, `src/catalog/discover.rs:148` |
| `std::io::Result<()>` for I/O helpers | `src/core/installer.rs:28` — `copy_dir_all`, `symlink_skill` |
| `?` operator propagation | ubiquitous |
| `let Some(x) = expr else { return … }` (let-else) | `src/core/installer.rs:119` — agent lookup with early return |
| `eprintln!("aviso: …")` for non-fatal warnings | `src/core/lockfile.rs:44` — corrupted lockfile handling |

Notable: **all error messages in Portuguese** — `"informe --catalog (uma ou mais vezes) ou…"`, `"catálogo inválido: {}"`, `"aviso: lockfile JSON inválido"`, `"tag inválida"`.

File paths: `src/cli.rs` (resolução de catálogo), `src/core/installer.rs:119–132`, `src/core/lockfile.rs:40–50`, `src/catalog/discover.rs:95`, `src/catalog/discover.rs:128–148`.

---

## 4. Type Usage

**`Clone` preferred; `Copy` only when semantically correct (small, trivial types).**

| Decision | Evidence |
|---|---|
| Structs derive `Clone` | `src/types.rs:10`, `src/types.rs:18`, `src/types.rs:30`, `src/types.rs:38`, `src/types.rs:46`, `src/types.rs:53` — all domain structs |
| Structs derive `Copy` only for enum/variant | `src/types.rs:46` — `InstallMethod` derives `Copy` |
| `Option<T>` for nullable fields | `src/types.rs:20`, `src/types.rs:24`, `src/types.rs:26`, `src/types.rs:28` — `category`, `content_hash`, `agents`, `method` |
| `#[serde(skip_serializing_if = "Option::is_none")]` to omit absent optionals | `src/types.rs:22`, `src/types.rs:72`, `src/types.rs:74`, `src/types.rs:76` — clean JSON output |
| `Result<Vec<T>>` for fallible collection builders | `src/cli.rs:116` — `hashes_for_skills` returns `Result<Vec<(String, String)>>` |
| `HashMap<String, T>` for keyed lookups | `src/types.rs:11` — `SkillLockFile.skills` is `HashMap<String, SkillLockEntry>` |
| `Vec<T>` for ordered collections | `src/catalog/discover.rs` — `discover_skills` returns `Vec<SkillInfo>` |
| Interior mutability not used | All mutation via explicit `&mut self` or `&mut` borrows |
| `fn(&Path) -> PathBuf` for computed paths | `src/agents.rs:7` — `global_skills_dir: fn(&Path) -> PathBuf` |
| `fn(&Path, &Path) -> bool` for detectors | `src/agents.rs:7` — `detect: fn(home: &Path, project_root: &Path) -> bool` |

---

## 5. Comment Style

**Doc comments (`///`) for public API surface; inline comments (`//`) for non-obvious logic; module-level `//!` for module purpose.**

| Style | Usage | Example |
|---|---|---|
| `//!` module-level doc | `src/sanitize.rs:1`, `src/core/installer.rs:1`, `src/ui/widgets.rs:1`, `src/ui/app.rs:13` | `//! Sanitiza nome de skill para uso em paths…` |
| `///` struct/field doc | `src/types.rs:3`, `src/types.rs:35–38` | `/// Versão SemVer lida apenas de skill.manifest.json` |
| `#[doc = "…"]` attribute | `src/ui/widgets.rs:13` | `#[doc = "Truncates text to fit…"]` |
| `//` inline — implementation detail | `src/core/installer.rs:215`, `src/core/installer.rs:330` | `// Remove canonical copy for local symlink installs` |
| `//` inline — platform-specific block | `src/core/installer.rs:65–78` | `#[cfg(unix)]`, `#[cfg(windows)]` with `//` comment per branch |
| `//` inline — section divider | `src/ui/app.rs:100–105` | `// Load catalog and lockfiles`, `// Initialize app state` |

**Language: Portuguese** for all doc comments and user-facing error messages.

Inline comments appear on the line above the relevant code (not trailing). Comments explain *why*, not *what*.

File paths: `src/types.rs:3`, `src/sanitize.rs:1`, `src/core/installer.rs:1`, `src/ui/app.rs:25`, `src/ui/widgets.rs:13`, `src/core/installer.rs:65–78`.

---

## 6. Testing Conventions

**Inline `#[cfg(test)] mod tests` for unit tests; `tests/` directory for integration tests.**

| Pattern | Evidence |
|---|---|
| `tests/cli_binary.rs` — integration binary tests | `Command::cargo_bin("ai-workflow-skills")` for subprocess CLI testing |
| `tests/cli_lifecycle.rs` — multi-step lifecycle tests | install → list → remove roundtrip validation |
| `mod common;` shared helpers | `tests/cli_binary.rs:9`, `tests/cli_lifecycle.rs:9` |
| `assert_cmd`, `predicates`, `tempfile` dev-dependencies | `Cargo.toml:48–51` |
| `#[test]` on all test functions | `tests/cli_binary.rs:15`, `tests/cli_lifecycle.rs:36` |
| `pub(crate) fn` for testable internals | `src/cli.rs` — `gather_catalog_inputs`, `resolve_all_catalogs`, `load_catalog_skills`, `load_merged_catalog_skills`, `find_skill` |
| `#[cfg(test)] mod tests` inline units | `src/sanitize.rs:54`, `src/core/audit.rs:35`, `src/agents.rs:157`, `src/catalog/remote.rs:117` |

Inline unit test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn my_test() { /* … */ }
}
```

Inline test imports use `use super::*` to pull in the parent module's public items.

File paths: `tests/cli_binary.rs` (full file), `tests/cli_lifecycle.rs` (full file), `src/sanitize.rs:54–62`, `src/core/audit.rs:35–75`, `src/agents.rs:157–205`, `src/catalog/remote.rs:117–128`.

---

## 7. Formatting / Linting Config

**`.githooks/` directory present.** Local git hooks: `commit-msg` (Conventional Commits validation) and `pre-commit` (fmt + clippy + tests). Enable with: `git config core.hooksPath .githooks`.

**GitHub Actions** (`.github/workflows/ci.yml`) — `rust-toolchain` 1.88.x, fmt, clippy, test. No `rustfmt.toml` or `clippy.toml` in project root — default `rustfmt` behavior applies.

- `Cargo.toml` has no `[profile]` overrides.
- Default `rustfmt` behavior: 4-space indent, 100-char soft wrap, alphabetical imports within groups.

File paths searched: `**/rustfmt.toml`, `**/clippy.toml`, `**/.editorconfig`, `**/.pre-commit-config.yaml`, `**/.github/`, `**/Makefile`.

---

## 8. Module Organization

```
src/
├── main.rs              # entry point — parse CLI, call run()
├── lib.rs               # pub mod re-exports
├── cli.rs               # CLI parsing + dispatch (large, complex)
├── types.rs             # shared domain structs and enums
├── sanitize.rs          # path sanitization utilities (+ unit tests)
├── constants.rs         # hardcoded constants (branding, paths, filenames)
├── agents.rs            # agent definitions, detection, and validation (+ unit tests)
├── project_root.rs      # project root discovery
├── catalog/             # catalog discovery, registry, and remote sync
│   ├── discover.rs      # skill discovery, manifest parsing, hashing (+ unit tests)
│   ├── registry.rs     # skills-registry.json generation
│   └── remote.rs       # Git clone/pull for remote catalog URLs (+ unit tests)
├── core/                # business logic
│   ├── audit.rs        # audit logging (+ unit tests)
│   ├── cache.rs        # cache management
│   ├── installer.rs    # install/remove logic
│   └── lockfile.rs     # lockfile read/write/migration (+ unit tests)
└── ui/                 # TUI (ratatui)
    ├── app.rs           # main application struct and event loop
    ├── state.rs         # app state, skill row model
    └── widgets.rs       # UI rendering primitives
```

Observations:

- `pub(crate)` visibility for internal entry points called from `cli.rs` (e.g., `resolve_all_catalogs`, `load_merged_catalog_skills`, `load_catalog_skills`, `find_skill`).
- `pub fn` for library boundary (e.g., `install_skills`, `remove_skills` in `core/installer.rs`).
- `pub` fields on all domain structs — no encapsulation beyond module-level.
- Module-level doc comments (`//!`) in `sanitize.rs`, `core/installer.rs`, `catalog/remote.rs`, and both `ui/` modules.
- `catalog/remote.rs` uses doc comments (`///`, `//!`) in Portuguese.

File paths: `src/lib.rs` for re-export list, all `src/` subdirectories.

---

## 9. Serialization Conventions

| Pattern | Evidence |
|---|---|
| `serde_json` for lockfile and audit log | `src/core/lockfile.rs`, `src/core/audit.rs` |
| Line-based JSON (one `AuditEntry` per line) | `src/core/audit.rs` — `serde_json::to_string` per entry |
| `#[serde(rename_all = "camelCase")]` on `AuditEntry` | `src/types.rs:67` |
| `#[serde(skip_serializing_if = "Option::is_none")]` to keep JSON clean | `src/types.rs:22, 72, 74, 76` |
| `serde` and `serde_json` as dependencies | `Cargo.toml` |
| `serde_yaml` available but not used in sampled files | `Cargo.toml` — `[INFERRED]` |

---

## 10. Visibility and Encapsulation

| Visibility | Usage | Example |
|---|---|---|
| `pub fn` | Library boundary / public API | `src/core/installer.rs:242` — `install_skills` |
| `pub(crate) fn` | Internal entry points called from `cli.rs` | `src/cli.rs` — `resolve_all_catalogs`, `load_merged_catalog_skills`, `load_catalog_skills` |
| `pub struct` | Data-only types | `src/types.rs` — all domain structs |
| `pub enum` | Command enum | `src/cli.rs:94` — `Commands` |
| `pub` fields on structs | Data transfer objects | `src/types.rs` — all structs have `pub` fields |

Minimal encapsulation — structs act as plain data containers with `pub` fields throughout. No getter/setter patterns observed.

---

## 11. Cargo / Dependency Conventions

| Pattern | Evidence |
|---|---|
| Crate name matches display name | `Cargo.toml:1` — `ai-workflow-skills` |
| Bin + lib layout | `main.rs` + `lib.rs` re-exports |
| `anyhow` for application error handling | `Cargo.toml` |
| `clap` derive for CLI parsing | `Cargo.toml` — `features = ["derive"]` |
| `serde` + `serde_json` | serialization |
| `chrono` for timestamps | `Cargo.toml` — `chrono::Utc::now().to_rfc3339()` in lockfile |
| `sha2` for content hashing | `Cargo.toml` — `Sha256::new()` in catalog/discover.rs |
| `walkdir` for recursive directory walking | `Cargo.toml` — skill file listing |
| `dunce` for canonical paths | `src/sanitize.rs`, `src/project_root.rs` — symlink-aware paths |
| `regex` via `once_cell::sync::Lazy` | `src/catalog/discover.rs` — compiled regex at module level |
| `ratatui` for TUI | `Cargo.toml` — `crossterm` for terminal I/O |
| `tempfile` as dev-dependency | `Cargo.toml` — test isolation |
| `libc` for `flock(2)` | `Cargo.toml` — lockfile write serialization |
