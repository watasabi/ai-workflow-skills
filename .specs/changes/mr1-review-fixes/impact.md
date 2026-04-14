# Impact — MR !1 review fixes

## Key finding (rewrite of R1 rationale)

`src/core/installer.rs::copy_dir_all` **already** does `fs::remove_dir_all(dst)` before copying (`src/core/installer.rs:17-20`). So stale files **within a given agent's target_dir** do not persist across reinstalls, cross-catalog or otherwise.

The actual latent bug in `apply_install` cross-catalog replacement is narrower:

1. `add_skill_to_lock` (`src/core/lockfile.rs:161-169`) **merges** agent lists (old + new), it does not overwrite.
2. On cross-catalog replacement, the TUI skips `remove_skill_scope_before_install` because `row.local`/`row.global` is `None` (join by `(name, catalog_label)` doesn't match).
3. Consequence: if cat1 installed for agents `[cursor, claude-code]` and the cat2 replacement install only targets `[cursor]` (because `detect_installed_agents` returned only cursor at that moment), the lockfile entry ends with `agents = [cursor, claude-code]`, `catalog_label = cat2`, `content_hash = cat2's hash` — but `claude-code`'s target_dir still holds cat1's physical files, and the lock's hash doesn't match any agent's content.

The fix is simple: also fire `remove_skill_scope_before_install` when `replacing_from.is_some()`. That cleans cat1's agent-dirs before cat2 installs fresh.

This path is hard to test without a PTY harness (TUI event loop is not reachable from integration tests). We'll add a **copy_dir_all cleanup regression** at the CLI level to protect the simpler invariant (a file exclusive to cat1's skill folder does not survive a cat2 reinstall), and document the TUI-specific fix in the commit message.

## R2 — find_skill prefixed branch

`src/cli.rs::find_skill` uses `candidates.next()` in the prefixed branch. For true consistency with the non-prefixed branch (which errors on ambiguity), the prefixed branch should `collect()` and match on `len()`. Duplicate `(catalog_label, name)` is unusual in practice but possible if a catalog is passed twice; erring is cleaner than silent first-pick.

## R3 — regression test

Extend `cli_install_same_name_from_two_catalogs_flips_provenance` in `tests/cli_lifecycle.rs`:
- Write a `cat1_only.txt` file inside cat1's `demo-skill` folder before installing from cat1.
- After installing from cat2, assert the installed skill dir does **not** contain `cat1_only.txt`.

This test protects `copy_dir_all`'s cleanup invariant at the CLI path.

## R4 — format! style nit

`src/ui/app.rs::apply_install` trailing `format!` mixes `{}` (positional) with `{prev}`/`{novo}` (named). Switch to fully named for clarity.

## R5 — unused `_catalog` binding

`src/cli.rs` Remove arm: `catalog: _catalog` → `catalog: _` (idiomatic, no underscore-prefixed unused binding).

## Files touched

| File | What changes | Why |
|------|-------------|-----|
| `src/ui/app.rs` | Cleanup condition includes `replacing_from.is_some()`; format! named args | R1, R4 |
| `src/cli.rs` | `find_skill` prefixed branch collects + errors on ambiguity; `_catalog` → `_` | R2, R5 |
| `tests/cli_lifecycle.rs` | Add cat1_only.txt regression to `cli_install_same_name_from_two_catalogs_flips_provenance` | R3 |

No new specs drift detected — ARCHITECTURE.md already documents the "single physical install per name" invariant. No spec update needed.
