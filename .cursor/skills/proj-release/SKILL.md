---
name: proj-release
description: >
  Release runbook for ai-workflow-skills: local checks aligned with GitHub Actions (fmt, clippy, test), doc review, SemVer bump in Cargo.toml, commit and push, wait for CI green, annotated tag vX.Y.Z, push tag. Use for shipping versions, changelog updates, or validating CI before tagging.
---

# proj-release — Release for this repository

Skill specific to **ai-workflow-skills**. CI facts live in `.github/workflows/ci.yml` and [`references/repo-release-contract.md`](references/repo-release-contract.md).

## Principles

1. **CI parity** — Before push/tag, run the same checks as the workflow (see contract).
2. **Branch first, tag after** — Merge to `main`, wait for a green run on that commit, then create and push the tag.
3. **SemVer** — Tag `vMAJOR.MINOR.PATCH`; `Cargo.toml` uses `MAJOR.MINOR.PATCH` without the `v` prefix.
4. **Secrets** — Never paste tokens into chat; CI secrets stay in GitHub.

## Main flow

1. Confirm target version and that work is merged to `main`.
2. Local: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`.
3. Bump `version` in `Cargo.toml`; refresh `Cargo.lock` if needed.
4. Commit, push to `main`, wait for GitHub Actions to pass.
5. `git tag -a vX.Y.Z -m "Release vX.Y.Z"` and `git push origin vX.Y.Z`.
6. Publish release assets from your fork’s process (GitHub Releases, etc.).

## References

| Topic | File |
|-------|------|
| CI steps | [references/repo-release-contract.md](references/repo-release-contract.md) |
| Checklist | [references/release-checklist.md](references/release-checklist.md) |
