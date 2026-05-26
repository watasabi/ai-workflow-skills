# Audit Digest

> Consolidated summary of all audit runs.
> Format: `### Run {id}` — auto-generated on completion.

---

## Audit Runs

### Run audit-001
> 2026-04-07 | Scope: STANDARD (21 source files) | Files: 9

Claims: 8 new (Logic: 6, Security: 1, Architecture: 1). Settled: 0 skipped.

Convergence: All 8 claims map to CONCERNS.md. No structural duplication found.

Observations: Well-organized domain types, clear layered architecture. Multiple `let _ =` patterns silently ignoring errors.
