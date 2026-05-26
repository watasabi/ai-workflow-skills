# Domain Map

> Persistent domain mapping for this codebase.
> Updated by each audit run.

---

## Subdomains

| Subdomain | Type | Cohesion | Key Concepts | Bounded Context |
|-----------|------|----------|--------------|-----------------|
| Skill Catalog | Core | 9/10 | skill, catalog, category, manifest, tag | CatalogContext |
| Skill Installation | Core | 8/10 | install, copy, symlink, lockfile, method | InstallationContext |
| Agent Configuration | Supporting | 7/10 | agent, skills_dir, global, detection | AgentContext |
| Audit Trail | Supporting | 6/10 | audit, log, timestamp, action | AuditContext |
| CLI Interface | Interface | 8/10 | command, subcommand, argument, TTY | CliContext |
| Cache Management | Supporting | 5/10 | cache, registry, path | CacheContext |

---

## Bounded Contexts

### CatalogContext

**Subdomains:** Skill Catalog
**Contains:** Skill discovery, metadata parsing, content hashing, category organization
**Language:**
- `Skill` — A deployable unit with `SKILL.md` and `skill.manifest.json`
- `Catalog` — Directory containing skills organized by category
- `Category` — Logical grouping (e.g., `(devops)/`, `(security)/`)
- `Manifest` — JSON file with SemVer and tags
- `Content Hash` — SHA-256 of all skill files

**Integration:**
- Consumes: Nothing (reads from filesystem)
- Publishes: `SkillInfo` structs to InstallationContext

---

### InstallationContext

**Subdomains:** Skill Installation
**Contains:** Copy/symlink logic, path validation, lockfile management
**Language:**
- `Install` — Copy or symlink skill to agent's skills directory
- `Lockfile` — JSON ledger of installed skills with metadata
- `Method` — Either `Copy` or `Symlink`
- `Global` — Installation scope (true = `~/.cursor/skills/`, false = `./.cursor/skills/`)
- `Update` — Reinstall when catalog content hash differs from lockfile

**Integration:**
- Consumes: `SkillInfo` from CatalogContext
- Publishes: Lockfile state changes
- Depends on: AgentContext (for target directories)

---

### AgentContext

**Subdomains:** Agent Configuration
**Contains:** Agent definitions, detection logic, directory conventions
**Language:**
- `Agent` — An AI tool with skills directory convention (Cursor, Claude Code, etc.)
- `Skills Dir` — Convention path for an agent's skills (e.g., `.cursor/skills/`)
- `Detection` — Heuristic to find installed agents

**Integration:**
- Consumed by: InstallationContext (to resolve target paths)
- No external dependencies

---

### AuditContext

**Subdomains:** Audit Trail
**Contains:** Audit log append, entry serialization
**Language:**
- `Audit Entry` — JSON object with action, skill_name, agents, success/failed counts
- `Action` — One of: `install`, `remove`

**Integration:**
- Consumed by: InstallationContext (to record events)
- No external dependencies

---

### CliContext

**Subdomains:** CLI Interface
**Contains:** Command parsing, TUI rendering, command dispatch
**Language:**
- `Command` — Clap-enumerated subcommands (list, install, update, remove, cache, audit, generate-registry)
- `TTY Mode` — Interactive terminal UI using ratatui

**Integration:**
- Orchestrates all other contexts
- Sole entry point for user interaction

---

### CacheContext

**Subdomains:** Cache Management
**Contains:** Registry caching, cache path resolution, cache clearing
**Language:**
- `Cache` — Optional local copy of catalog registry
- `Cache Path` — `~/.cache/ai-workflow-skills/`

**Integration:**
- Consumed by: CLI (for `cache` command)
- No dependencies on other domains

---

## Cross-Domain Cohesion

| Domain A | Domain B | Cohesion | Issue | Recommendation |
|----------|----------|----------|-------|----------------|
| CatalogContext | InstallationContext | 9/10 | OK — natural flow | Keep as separate modules |
| InstallationContext | AgentContext | 8/10 | OK — Agent provides paths | Keep integration via `agents.rs` |
| InstallationContext | AuditContext | 7/10 | OK — installs trigger audits | Keep as separate modules |
| CliContext | InstallationContext | 6/10 | UI calls dispatch_command (cross-layer) | Known issue, CONCERNS.md |
| InstallationContext | CacheContext | 4/10 | Weak coupling via CLI | Acceptable separation |

---

## Low Cohesion Issues

| Location | Issue | Concepts Involved | Recommendation |
|----------|-------|-------------------|----------------|
| `src/cli.rs:172` | dispatch_command handles catalog resolution, agent detection, AND command execution | Multiple domains | Consider extracting catalog/agent resolution |
| `src/ui/app.rs` | TUI calls `dispatch_command` re-entrantly | CliContext → InstallationContext | Direct calls to core::installer preferred |

---

## Update Log

| Run | Date | Changes |
|-----|------|---------|
| audit-001 | 2026-04-07 | Initial domain map created |
