# Structural Duplication & Consolidation

> Persistent record of duplication analysis and consolidation opportunities.
> Updated by each audit run.

---

## Common Namespace Patterns

| Pattern | Components | Bounded Context | Notes |
|---------|-----------|----------------|-------|
| `get_def` | `agents.rs`, `installer.rs` | AgentContext | Different usage — AgentContext definition vs InstallationContext lookup |
| `to_slug` | `sanitize.rs`, `catalog/discover.rs`, `catalog/registry.rs`, `cli.rs` | Cross-context utility | Shared utility, acceptable |

---

## Shared Classes

| Class | Used By | Type | Notes |
|-------|---------|------|-------|
| `SkillInfo` | cli, catalog, ui, types | Domain | Core domain type, shared by design |
| `SkillLockFile` | cli, core/lockfile, ui, types | Domain | Lockfile domain type |
| `AuditEntry` | core/installer, core/audit, types | Domain | Audit domain type |
| `InstallOptions` | cli, core/installer, ui, types | Domain | Domain config type |
| `InstallResult` | cli, core/installer, types | Domain | Domain result type |
| `sanitize_name` | cli, core/installer, sanitize | Utility | Cross-context utility |
| `to_slug` | cli, catalog, sanitize | Utility | Cross-context utility |
| `is_path_safe` | core/installer, sanitize | Security | Security utility |

---

## Consolidation Opportunities

| Functionality | Components | Current CA | After CA | Feasibility | Approach | Claim |
|---------------|-----------|------------|----------|-------------|----------|-------|
| Error handling consistency | `audit.rs`, `lockfile.rs` | 2 | 2 | Medium | Note | No — different domains |
| Path validation | `sanitize.rs`, `installer.rs` | 2 | 1 | High | Merge | No — acceptable separation |

**Analysis:**

1. **Error patterns** (`eprintln!` for warnings vs `Result` propagation): Different bounded contexts (AuditContext vs InstallationContext) — **tolerated**
2. **Path validation**: Already in `sanitize.rs`, consumed by `installer.rs` — acceptable separation, not duplicate logic
3. **File operations**: No structural duplication — each module has distinct file operations appropriate to its domain

---

## Tolerated Duplication

| Components | Reason |
|-----------|--------|
| `catalog/discover.rs` vs `catalog/registry.rs` | Different bounded contexts (Skill Catalog vs CI/Registry generation) |
| `core/audit.rs` vs `core/lockfile.rs` | Different bounded contexts (Audit Trail vs Installation State) |
| `cli.rs` vs `ui/app.rs` | Different bounded contexts (CLI Interface vs TUI Interface) — different entry points |

---

## Coupling Analysis

### Afferent Coupling (Incoming Dependencies)

| Module | CA | Notes |
|--------|-----|-------|
| `types.rs` | 9 | All domain types — expected for a single-binary CLI |
| `sanitize.rs` | 3 | Consumed by installer, cli, catalog |
| `agents.rs` | 2 | Consumed by installer, cli |
| `constants.rs` | 8 | Consumed by all modules — acceptable for constants |

### Coupling Balance Assessment

Using Khononov's 3D model:

| Module | Strength | Distance | Volatility | Assessment |
|--------|----------|----------|------------|------------|
| `types.rs` | High | Low | Low | **Balanced** — internal module, high cohesion |
| `constants.rs` | Medium | Low | Low | **Balanced** — acceptable for config |
| `sanitize.rs` | Medium | Medium | Low | **Balanced** — small utility |
| `agents.rs` | High | Medium | Low | **Watch** — if agent count grows, consider plugin pattern |

---

## No Structural Duplication Claims

**Rationale:**
- This is a small CLI tool (~21 source files)
- Domain types in `types.rs` are shared by design, not duplication
- Utility functions in `sanitize.rs` are consumed appropriately
- No semantic duplication found across bounded contexts
- Rule of Three does not apply — no pattern repeated 3+ times with shared volatility

---

## Update Log

| Run | Date | Changes |
|-----|------|---------|
| audit-001 | 2026-04-07 | Initial consolidation analysis |
