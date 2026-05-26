# Project Structure

**Analyzed:** 2026-04-07 (drift sync: 2026-04-08 — added catalog/remote.rs, removed interactive_install.rs)
**Root:** (local clone path)

**Source file count:** 26 `*.rs` files under `src/` and `tests/` (library + integration tests).

## Directory Tree

```
ai-workflow-skills/
├── src/                          # Source code
│   ├── main.rs                   # Binary entry point
│   ├── lib.rs                    # Library entry point, module exports
│   ├── cli.rs                    # CLI command dispatch and argument handling
│   ├── types.rs                  # Core data types (SkillInfo, InstallOptions, etc.)
│   ├── constants.rs              # Path constants and configuration values
│   ├── agents.rs                 # AI agent definitions and detection
│   ├── sanitize.rs               # Path sanitization utilities
│   ├── project_root.rs           # Project root detection logic
│   ├── catalog/                  # Skill catalog operations
│   │   ├── mod.rs               # Module exports
│   │   ├── discover.rs           # Skill discovery, manifest parsing, hashing
│   │   ├── registry.rs          # Skills registry generation
│   │   └── remote.rs            # Git clone/pull for remote catalog URLs
│   ├── core/                     # Core business logic
│   │   ├── mod.rs               # Module exports
│   │   ├── installer.rs          # Skill installation (copy/symlink)
│   │   ├── lockfile.rs           # Lockfile read/write operations
│   │   ├── cache.rs              # Cache management
│   │   └── audit.rs              # Audit logging
│   └── ui/                       # Terminal UI
│       ├── mod.rs                # Module exports
│       ├── app.rs                # TUI application logic
│       ├── state.rs              # TUI state management
│       └── widgets.rs            # TUI widget definitions
├── tests/                        # Integration tests
│   ├── common/mod.rs             # Shared test utilities
│   ├── cli_binary.rs             # Binary smoke tests
│   ├── cli_lifecycle.rs          # Lifecycle tests
│   ├── installer_lib.rs          # Installer library tests
│   └── integration.rs            # Integration tests (catalog, sanitize, registry)
├── Cargo.toml                    # Rust dependency manifest
├── Cargo.lock                    # Locked dependency versions
├── README.md                     # Project documentation
├── SCOPE.md                      # Command matrix and behavioral spec
├── .github/workflows/ci.yml      # GitHub Actions: fmt, clippy, test
├── .githooks/                    # Local hooks — enable with: git config core.hooksPath .githooks
│   ├── commit-msg                # Conventional Commits
│   └── pre-commit                # Same as CI test:rust: fetch --locked, fmt --check, clippy -D warnings, test
└── .specs/                       # Codebase analysis specs
    └── codebase/                  # pwf-init-spec output
```

## Module Map

### CLI Entry Point

**Location:** `src/main.rs`, `src/cli.rs`
**Purpose:** Command-line interface and user interaction

**Key files:**
- `src/main.rs` — Binary entry, parses CLI, calls `cli::run()`
- `src/cli.rs` — Command enum, dispatch logic, progress bar handling

### Catalog Module

**Location:** `src/catalog/`
**Purpose:** Discover, parse, and sync skills from local catalog or remote Git repository

**Key files:**
- `src/catalog/discover.rs` — Skill discovery, YAML frontmatter parsing, SHA-256 hashing
- `src/catalog/registry.rs` — JSON registry generation for CI/review
- `src/catalog/remote.rs` — Git clone/pull for remote catalog URLs (`resolve_remote_or_local`, `sync_git_catalog`)

### Core Module

**Location:** `src/core/`
**Purpose:** Business logic for skill management

**Key files:**
- `src/core/installer.rs` — Install/remove skills (copy or symlink methods)
- `src/core/lockfile.rs` — Lockfile CRUD operations
- `src/core/cache.rs` — Cache directory management
- `src/core/audit.rs` — JSON audit log append-only writes

### UI Module

**Location:** `src/ui/`
**Purpose:** Interactive TUI for skill selection and management

**Key files:**
- `src/ui/app.rs` — TUI application state machine
- `src/ui/state.rs` — State definitions for TUI
- `src/ui/widgets.rs` — Ratatui widget implementations. Contains skill list table, description container (shows skill name + description), flash messages, and help bar.

### Agent Definitions

**Location:** `src/agents.rs`
**Purpose:** Define supported AI agents and their skill directory conventions

**Key types:**
- `AgentDef` struct — id, display_name, skills_dir, global_skills_dir, detect closure
- 17 supported agents: cursor, claude-code, github-copilot, windsurf, cline, aider, codex, gemini, antigravity, roo, kilocode, trae, amazon-q, augment, tabnine, opencode, sourcegraph, droid, kiro

## Capability Map

| Capability | Entry Point | Business Logic | Data Access | Config |
|------------|-------------|----------------|-------------|--------|
| List skills | `cli.rs::Commands::List` | `catalog/discover.rs::discover_skills()` | Filesystem reads | `--catalog` flag / env var |
| Install skills | `cli.rs::Commands::Install` | `core/installer.rs::install_skills()` | Lockfile (`core/lockfile.rs`) | `--agent`, `--global`, `--symlink` flags |
| Update skills | `cli.rs::Commands::Update` | `core/installer.rs::install_skills()` | Lockfile + catalog hash comparison | `--catalog` flag |
| Remove skills | `cli.rs::Commands::Remove` | `core/installer.rs::remove_skills()` | Lockfile updates | `--agent`, `--global` flags |
| Cache management | `cli.rs::Commands::Cache` | `core/cache.rs` | Filesystem | — |
| Audit log | `cli.rs::Commands::Audit` | `core/audit.rs` | JSON log file | — |
| Registry generation | `cli.rs::Commands::GenerateRegistry` | `catalog/registry.rs::generate_registry()` | Filesystem | `--catalog`, `--output` flags |
| Remote catalog sync | `catalog/remote.rs::resolve_remote_or_local()` | Git clone/pull | `~/.cache/ai-workflow-skills/catalog-git/` | `--catalog` URL / `AI_WORKFLOW_SKILLS_CATALOG` |
| Interactive TUI | `ui/app.rs::run()` | UI state machine | — | — |

## Infrastructure & Deploy

**Containerization:** None (CI uses upstream `rust` and `curlimages/curl` images only).
**Orchestration:** None
**CI/CD:** GitHub Actions — `.github/workflows/ci.yml` (fmt, clippy, test on push/PR).
**Environments:** N/A (standalone binary tool)
**Deploy process:** Release binaries and `install.sh` are maintained by each fork; local `cargo build`; see `README.md`

## Special Directories

**`target/`:** Cargo build output directory (`.gitignore`d)
**`tests/`:** Integration test suite using `assert_cmd`
**`.specs/codebase/`:** pwf-init-spec analysis output
**`.githooks/`:** Local git hooks (`.git/hooks` redirected here via `git config core.hooksPath .githooks`)
