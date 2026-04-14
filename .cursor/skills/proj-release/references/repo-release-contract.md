# CI contract — ai-workflow-skills

Facts taken from `.github/workflows/ci.yml` and `Cargo.toml`. Update this file if CI changes.

## Workflow: `ci.yml` (on push/PR to `main` or `master`)

1. **Checkout** repository
2. **Rust toolchain** — 1.88.0 with `rustfmt`, `clippy`
3. **cargo fetch --locked**
4. **cargo fmt --check**
5. **cargo clippy --all-targets --all-features -- -D warnings**
6. **cargo test --all-features**

## Releases

There is **no** automated publish job in this open fork: tagging and attaching binaries to GitHub Releases (or another registry) is manual or added per fork.

## Variables

- Optional `GITHUB_TOKEN` / `GH_TOKEN` for private clones in `install.sh` — user-side, not CI.

## Manifest

- `Cargo.toml` `repository` / `homepage` — `https://github.com/watasabi/ai-workflow-skills`
