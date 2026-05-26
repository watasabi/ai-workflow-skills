# Plan — MR !1 review fixes

Single atomic commit (scope: Small). All changes verifiable with `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.

## Task A — R1 + R4: cross-catalog cleanup guard + format! nit
**File:** `src/ui/app.rs`
- In `apply_install`, widen cleanup guard to fire on `replacing_from.is_some()` as well.
- Refactor to a single `needs_cleanup` computation for clarity.
- Add short comment explaining why (stale agent-dirs when previous lock entry listed agents not targeted by the new install).
- Switch the trailing `format!` to fully named args (`name = …, prev = …, novo = …`).

## Task B — R2: find_skill prefixed branch consistency
**File:** `src/cli.rs`
- `find_skill`: prefixed branch collects into `Vec<&SkillInfo>` and errors on `len() != 1`, mirroring the non-prefixed branch.
- Add a unit test `find_skill_with_catalog_prefix_errors_on_duplicates` in `src/cli.rs::tests`.

## Task C — R5: `catalog: _catalog` nit
**File:** `src/cli.rs`
- Remove dispatch arm: `catalog: _catalog` → `catalog: _`.

## Task D — R3: regression test for copy_dir_all cleanup
**File:** `tests/cli_lifecycle.rs`
- In `cli_install_same_name_from_two_catalogs_flips_provenance`, write a `cat1_only.txt` inside cat1's `demo-skill` folder before the first install.
- After the cat2 install, assert `.cursor/skills/demo-skill/cat1_only.txt` does not exist.

## Verification

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Commit

Single commit:
```
fix(multi-catalog): cross-catalog replacement cleanup + find_skill strict prefix

- apply_install: trigger pre-install cleanup when cross-catalog replacement
  is detected (replacing_from.is_some()), so stale agent-dirs from the
  previous catalog don't outlive the provenance flip.
- find_skill: prefixed branch now errors on duplicate (label, name),
  matching the non-prefixed branch's strictness.
- tests: regression test for copy_dir_all cleanup on cross-catalog replace.
- minor: format! named args consistency, drop _catalog alias.
```

## Risks

- **Cleanup call path under global replacement**: `remove_skill_scope_before_install` reads both local and global lockfiles (already) and selects the scope based on `global`. Fix is purely additive — no new code paths in the removal logic.
- **Test portability**: `cat1_only.txt` inside `(demo)/demo-skill/` — `copy_dir_all` copies all files, so the file will land in the installed dir on first install; on second install, `remove_dir_all(dst)` should delete it. No assumption about filesystem semantics beyond what the existing tests already rely on.
