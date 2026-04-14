# Release checklist

- [ ] `README.md` / `AGENTS.md` / `install.sh` — version strings and URLs still accurate
- [ ] `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`
- [ ] `Cargo.toml` version bumped; `Cargo.lock` updated if dependencies changed
- [ ] Push to `main` and confirm GitHub Actions is green
- [ ] Tag `vX.Y.Z` and push; create GitHub Release with notes if you publish binaries
