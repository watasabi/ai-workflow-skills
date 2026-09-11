# Tech Stack

**Analyzed:** 2026-04-07 (updated 2026-04-07 — drift sync)

## Core

- Language: Rust (edition 2021)
- **MSRV / `rust-version`:** **1.88** — set in `Cargo.toml` (`rust-version = "1.88"`). CI uses `rust:1.88`.
- Framework: Clap 4 (derive-based CLI)
- Runtime: None (standalone binary)
- Package manager: Cargo (Rust)
- Build tool: Cargo

## Getting Started

**Prerequisites:** Rust toolchain matching `rust-version` in `Cargo.toml` (currently **1.88**). `README.md` may mention additional compatibility notes for contributors.

**Install dependencies:**
```bash
cargo build
```

**Build for production:**
```bash
cargo build --release
# binary: target/release/ai-workflow-skills
```

**Run tests:**
```bash
cargo test
```

**Other useful commands:**
```bash
cargo fmt         # format code
cargo check       # type check
cargo clippy      # lint
```

## Key Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| clap | 4 | CLI argument parsing with derive macros |
| anyhow | 1 | Ergonomic error handling with `Result<T>` |
| serde | 1 | Serialization/deserialization for JSON/YAML |
| serde_json | 1 | JSON parsing |
| serde_yaml | 0.9 | YAML frontmatter parsing |
| ratatui | 0.29 | TUI framework for interactive prompts |
| crossterm | 0.28 | Terminal capabilities for TUI |
| dunce | 1 | Canonical paths on Windows-safe semantics (`sanitize`, project root) |
| sha2 | 0.10 | SHA-256 hashing for skill content verification |
| semver | 1 | Semantic version parsing and comparison |
| walkdir | 2 | Directory traversal |
| dirs | 5 | Cross-platform home directory resolution |
| chrono | 0.4 | Timestamp handling for audit logs |
| regex | 1 | Pattern matching for frontmatter/category parsing |
| pathdiff | 0.2 | Cross-platform relative path calculation |
| libc | 0.2 | Unix `flock(2)` for lockfile write serialization |
| once_cell | 1 | Static initialization for lazy regexes |

## Development Tools

- Linter: Clippy (`cargo clippy`)
- Formatter: rustfmt (`cargo fmt`)
- Type checking: Cargo check

## Dev Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| assert_cmd | 2 | CLI integration testing |
| predicates | 3 | Assertion combinators for tests |
| tempfile | 3 | Temporary directory/file creation for tests |

## CI/CD

- **GitHub Actions:** `.github/workflows/ci.yml` — fmt, clippy, `cargo test --all-features` on `ubuntu-latest` with Rust 1.88.x.

## Notes

- No Docker/containerization in-repo (CI uses `rust` official image)
- No database dependencies (filesystem-based skill catalog)
- No remote catalog API in-tool (self-contained CLI; catalog is local path)
