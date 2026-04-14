# Contributing to ai-workflow-skills

## Prerequisites

- Rust (see `rust-version` in `Cargo.toml`; CI uses the same toolchain)
- `cargo`, `rustfmt`, `clippy`

## Clone and test

```bash
git clone https://github.com/watasabi/ai-workflow-skills.git
cd ai-workflow-skills
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Commits

This repository follows [Conventional Commits](https://www.conventionalcommits.org/). The `.githooks/commit-msg` hook validates the format if you enable hooks:

```bash
git config core.hooksPath .githooks
```

To skip the pre-commit hook when needed: `AI_WORKFLOW_SKILLS_HOOK_SKIP=1 git commit …` or `git commit --no-verify`.

## CI

GitHub Actions runs `fmt`, `clippy`, and `tests` on each push and pull request (see `.github/workflows/ci.yml`).
