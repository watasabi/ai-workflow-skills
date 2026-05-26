---
name: proj-specs-testing
description: Testing specs for ai-workflow-skills (.specs/codebase/TESTING.md and test-audits). Use when adding tests, fixing CI, running cargo test/clippy/fmt, or addressing test audit claims. Do NOT use for feature design without tests.
---

# Project testing specs

Live source: `.specs/codebase/TESTING.md`, `.specs/test-audits/`

## References

| Topic | File |
|-------|------|
| Test layout, fixtures, CI parity | [TESTING.md](references/TESTING.md) |
| Open test-audit claims | [test-audits-claims.md](references/test-audits-claims.md) |
| Test-audit history digest | [test-audits-DIGEST.md](references/test-audits-DIGEST.md) |

## CI parity (before push)

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Sync

```bash
DEST=catalog/skills/\(development\)/proj-specs-testing/references
cp .specs/codebase/TESTING.md $DEST/
cp .specs/test-audits/claims.md $DEST/test-audits-claims.md
cp .specs/test-audits/DIGEST.md $DEST/test-audits-DIGEST.md
```
