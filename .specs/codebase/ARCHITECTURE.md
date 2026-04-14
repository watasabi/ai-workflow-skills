# Architecture & Data Model

> **Analysis date:** 2026-04-07 (drift sync: 2026-04-09 — múltiplos catálogos: `resolve_all_catalogs`, `load_merged_catalog_skills`, `generate_registry_merged`)
> **Project root:** (local clone path)
> **Sources:** Direct reading of 14 source files

---

## 1. Architectural Pattern

**Layered CLI architecture** — not clean architecture, not hexagonal, not event-driven. Each layer has a distinct concern but there is no dependency inversion. Lower layers are called directly.

```
┌──────────────────────────────────────────────────────┐
│  CLI Entry Layer (main.rs → lib.rs → cli.rs)          │ Layer 1
│  - Parses clap arguments (derive-based)               │
│  - Resolves catalog path(s) via ENV / repeatable --catalog │
│    (local path OR remote Git URL via catalog/remote); merge │
│    ordenado com precedência do primeiro em nomes duplicados │
│  - Dispatches to command handlers                    │
└──────────────────────┬───────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────┐
│  UI Layer (ui/)                                      │ Layer 2
│  - ratatui TUI for interactive skill/agent selection │ (optional, activated
│  - Activated when no subcommand + TTY detected        │ when no subcommand
│  - Falls back to help text if not a TTY              │ + TTY available)
└──────────────────────┬───────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────┐
│  Business Logic Layer (core/)                         │ Layer 3
│  - installer.rs: copy/symlink, path validation        │
│  - lockfile.rs: read/write skill lockfile (v2 JSON)   │
│  - audit.rs: append-only NDJSON audit log             │
│  - cache.rs: registry cache + general cache           │
└──────────────────────┬───────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────┐
│  Catalog Layer (catalog/)                             │ Layer 4
│  - discover.rs: filesystem → SkillInfo structs        │
│  - registry.rs: generate skills-registry.json          │
│  - remote.rs: Git clone/pull for remote catalog URLs   │
└──────────────────────┬───────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────┐
│  Domain / Utility Layer (types.rs, agents.rs,          │ Layer 5
│  sanitize.rs, project_root.rs, constants.rs)           │
└──────────────────────────────────────────────────────┘
```

**Module organization** (`src/`):

| Module | File | Responsibility |
|--------|------|----------------|
| `cli` | `cli.rs` | Command dispatch, catalog resolution, progress bar orchestration |
| `ui` | `ui/app.rs`, `ui/state.rs`, `ui/widgets.rs` | Ratatui TUI, key event handling, skill row rendering |
| `core` | `core/installer.rs`, `core/lockfile.rs`, `core/audit.rs`, `core/cache.rs` | Install/remove logic, lockfile persistence, audit log, cache management |
| `catalog` | `catalog/discover.rs`, `catalog/registry.rs`, `catalog/remote.rs` | Skill discovery, frontmatter parsing, SHA-256 hashing, registry generation, remote catalog Git sync |
| `agents` | `agents.rs` | 17 AI agent definitions with `skills_dir` conventions |
| `types` | `types.rs` | All domain structs: `SkillLockFile`, `SkillLockEntry`, `SkillInfo`, `AuditEntry`, `InstallOptions`, etc. |
| `sanitize` | `sanitize.rs` | Skill name sanitization, path safety checks, slug generation, cache path helpers |
| `project_root` | `project_root.rs` | Project root detection (walks up for `package.json` or `.git`) |
| `constants` | `constants.rs` | Path constants, env vars, file names, branding |

**No dependency inversion.** `cli.rs` calls `core::installer::install_skills`, `core::lockfile::*`, and `catalog::*` directly. No trait abstractions. Concrete structs flow through all layers.

**No domain layer separation.** Types that are clearly domain models (`SkillLockEntry`, `AuditEntry`, `SkillInfo`) live in `types.rs` rather than a dedicated domain module. This is acceptable for a CLI tool of this scale — the absence of a domain layer is a structural observation, not a critique.

---

## 2. Data Model

### 2.1 `SkillLockFile` (lockfile — installed-state ledger)

**Defined in:** `src/types.rs:10–13`

```rust
pub struct SkillLockFile {
    pub version: i32,   // Always 2 (LOCKFILE_VERSION)
    pub skills: HashMap<String, SkillLockEntry>,
}
```

One lockfile per scope:
- **Local:** `<project>/.agents/.skill-lock.json`
- **Global:** `~/.ai-workflow-skills/.agents/.skill-lock.json`

The `HashMap` key is the skill name string. Agents are merged per skill name.

### 2.2 `SkillLockEntry` (per-skill lock entry)

**Defined in:** `src/types.rs:15–32`

| Field | Type | Purpose |
|-------|------|---------|
| `name` | `String` | Skill display name |
| `source` | `String` | `"local"` or catalog-relative path; defaults to `"local"` on migration |
| `content_hash` | `Option<String>` | SHA-256 of all skill files; `None` if not yet computed |
| `installed_at` | `String` | RFC3339 timestamp; **preserved** on re-install from existing entry |
| `updated_at` | `String` | RFC3339 timestamp; **always refreshed** on any write |
| `agents` | `Option<Vec<String>>` | Agent IDs the skill is installed for |
| `method` | `Option<String>` | `"copy"` or `"symlink"`; defaults to `"copy"` on migration from v1 |
| `global` | `Option<bool>` | `true` → global scope, `false` → project-local; defaults to `false` on migration |
| `version` | `Option<String>` | SemVer from `skill.manifest.json` |

**Migration path:** `src/core/lockfile.rs:migrate()` upgrades lockfiles with `version < 2` by setting `method` to `"copy"` and `global` to `false` for every entry. Invalid JSON in the lockfile is treated as empty (silent degradation with a warning to stderr).

**Update trigger:** `content_hash` comparison between catalog and lockfile. If hashes differ, the skill is out-of-date.

### 2.3 `SkillInfo` (discovered skill — in-memory only)

**Defined in:** `src/types.rs` (`SkillInfo`)

```rust
pub struct SkillInfo {
    pub name: String,
    pub catalog_label: String,     // short label from the catalog root path (set in load_catalog_skills)
    pub description: String,
    pub catalog_version: String,   // SemVer from skill.manifest.json
    pub tags: Vec<String>,         // kebab-case tags from manifest
    pub path: std::path::PathBuf,  // absolute path on disk
    pub category: Option<String>,  // "(category-id)" or None for uncategorized
}
```

Produced by `catalog/discover.rs:discover_skills()` from the catalog filesystem; `catalog_label` is filled in `cli.rs::load_catalog_skills()` from the catalog root path. Never serialized to disk in this form. Carries an **absolute** path — catalog relocation breaks this.

### 2.4 `AuditEntry` (audit log line — NDJSON)

**Defined in:** `src/types.rs:82–94`

```rust
pub struct AuditEntry {
    pub action: String,               // "install" | "remove"
    pub skill_name: String,          // Comma-joined for multi-skill operations
    pub agents: Vec<String>,          // Display names (e.g., "Cursor", "Claude Code")
    pub success: i32,
    pub failed: i32,
    pub forced: Option<bool>,         // Present only for --force installs/removes
    pub details: Option<serde_json::Value>,  // Array of per-agent/per-skill results
    pub timestamp: Option<String>,   // Set by log_audit() to UTC RFC3339
}
```

Stored as **one JSON object per line** (NDJSON) at `~/.ai-workflow-skills/audit.log`. Malformed lines are silently skipped with a warning. No schema version on the log itself.

### 2.5 Supporting Types

| Type | Location | Role |
|------|----------|------|
| `InstallOptions` | `src/types.rs:46–54` | Parameters: `global`, `method`, `agents`, `skills`, `force_update`, `audit_forced` |
| `InstallResult` | `src/types.rs:62–72` | Per-agent, per-skill install outcome |
| `RemoveOptions` | `src/types.rs:74–77` | `global`, `force` |
| `RemoveResult` | `src/types.rs:79–84` | Per-agent removal outcome |
| `InstallMethod` | `src/types.rs:58–61` | `Copy \| Symlink` enum |
| `AgentDef` | `src/agents.rs` | Per-agent static config: `id`, `skills_dir`, `global_skills_dir`, `detect` closure |
| `SkillsRegistry` | `src/catalog/registry.rs` | Generated JSON registry: version, categories, skills, deprecated |
| `SkillMetadata` | `src/catalog/registry.rs` | Per-skill metadata for registry output |

---

## 3. Key Data Flows

### 3.1 Install Flow

```
cli.rs:dispatch_command(Commands::Install)
  ├── gather_catalog_inputs() / resolve_all_catalogs(home, cli_catalogs)
  │     → Vec<ResolvedCatalog { path, label, remote }>
  │     └── cada URL → catalog/remote.rs::resolve_remote_or_local()
  │           └── sync_git_catalog() → git clone (first time) or git pull --ff-only
  │     └── catalog_label_for(): local → file_name(); remoto → último segmento da URL sem `.git`
  ├── load_merged_catalog_skills(catalogs)      → Vec<SkillInfo> (nomes podem repetir-se entre catálogos)
  │     └── load_catalog_skills(path, label): discover_skills + preenche `SkillInfo.catalog_label`
  │           └── try_read_skill(): parses SKILL.md frontmatter + skill.manifest.json
  ├── find_skill(catalog, name) → Result<&SkillInfo>
  │     • sem `/`: match por nome; se >1 match → Err com lista de candidatos `catalogo/skill`
  │     • com `/`: match exato por `(catalog_label, name)`
  ├── detect_installed_agents() or default     → Vec<String>
  ├── install_with_progress_bar()
  │     └── install_skills(project_root, home, skills, opts, on_step)
  │           ├── for each (agent, skill):
  │           │     install_one(skill, agent, method, global)
  │           │       ├── get_def(agent) → AgentDef
  │           │       ├── validate_install_path() — is_path_safe check
  │           │       ├── local symlink: copy to .agents/skills/ first
  │           │       └── copy_dir_all() OR symlink_skill()
  │           ├── files_and_hash(skill.path) inline → content_hash
  │           ├── lockfile::add_skill_to_lock(catalog_label=...)   ← writes .skill-lock.json
  │           └── log_audit()                     ← appends to audit.log (NDJSON)
  └── prints ok/fail counts, exits 1 if any failure
```

**Lockfile schema v3** (`LOCKFILE_VERSION = 3`): `SkillLockEntry.catalog_label: Option<String>` records which catalog installed each skill (provenance). Legacy v2 entries migrate with `catalog_label = None`; the TUI grants these a best-effort match to the first row of that name. The lockfile is still keyed by bare skill name — only **one physical installation per skill name** can exist on disk because agents (Cursor, Claude Code, Windsurf) expect a flat `<agents-dir>/<name>/` layout. Installing the "other" catalog's row for a colliding name explicitly replaces the physical skill and flips the provenance, surfaced as `substituído: <name> (catálogo X → Y)` in the TUI flash message.

**Lockfile merge behavior:** `add_skill_to_lock()` preserves `installed_at` from any existing entry. Agents are deduplicated — no duplicate agent entries per skill. If `content_hash` is already present, it is not overwritten.

**Symlink strategy (local install):**
1. Copy skill to `.agents/skills/<name>/` (canonical store)
2. Symlink `<agent>/skills/<name>` → `.agents/skills/<name>/`
3. If symlink fails, fall back to direct copy (marks `symlink_failed: true`)

**Symlink strategy (global install):**
1. Symlink `~/.cursor/skills/<name>` → catalog path
2. If symlink fails (cross-filesystem), fall back to copy

### 3.2 Update Flow

```
cli.rs:dispatch_command(Commands::Update)
  ├── resolve_all_catalogs() / load_merged_catalog_skills()
  ├── lockfile::read_skill_lock()         ← current installed state
  ├── for each (name, lock_entry) in lock:
  │     find_skill(catalog, name)         ← look up in catalog by name or slug
  │     if skill_filter is Some → skip if name not in filter
  │     files_and_hash()                  ← compute catalog hash
  │     compare catalog_hash vs lock_entry.content_hash
  │     if changed → add to refresh list
  └── install_with_progress_bar() on changed skills
        └── same as install flow with force_update=true
```

**Hash comparison is the update trigger.** If `content_hash` differs, the skill is reinstalled. If `content_hash` matches but version differs, the install still proceeds (overwrites with same content).

**Global vs. local:** `use_global_lock` parameter selects which lockfile to read and write. The `Update` command does not check both lockfiles — it operates on one scope at a time.

### 3.3 Remove Flow

```
cli.rs:dispatch_command(Commands::Remove)
  ├── lockfile::get_skill_from_lock()  ← checks local lock only (agent inference)
  ├── for each (skill, agent):
  │     remove_skills(project_root, home, name, agents, opts)
  │           ├── get_skill_from_lock() — reads both local and global
  │           ├── if !force && not in either lock → returns error per agent
  │           ├── resolve local + global paths per agent
  │           ├── is_path_safe()       ← security check on each path
  │           ├── fs::remove_dir_all() on each path
  │           ├── lockfile::remove_agent_from_lock()  ← removes agent from entry
  │           │     └── if agents list empty → deletes skill entry from lock
  │           └── if symlink was used → delete canonical .agents/skills/<name>
  └── log_audit()                      ← appends to audit.log
```

**Without `--force`:** If skill is not in either lockfile, returns error without deleting anything.

**With symlink install:** The canonical copy under `.agents/skills/<name>/` is removed only after all agents referencing it are gone.

### 3.4 TTY Interactive Flow

```
cli.rs:run() → no subcommand + TTY detected
  └── ui::run(home, cwd, catalog_roots: &[PathBuf])
        ├── load_merged_catalog_skills(catalog_roots)
        ├── lockfile::read_skill_lock() × 2 (local + global)
        ├── AppState::new() + load_catalog_hashes()
        ├── enable_raw_mode() + enter_alternate_screen
        └── run_tui() — event loop:
              ├── terminal.draw(|frame| app.ui(frame))
              ├── event::poll(TICK_RATE_MS) → key event
              └── handle_key_event():
                    q/Esc → quit
                    Up/Down → cursor movement
                    p → set PendingAction::InstallProject
                    g → set PendingAction::InstallGlobal
                    u → set PendingAction::Update
                    r → set PendingAction::Remove
                    c → clear pending
                    Enter → apply_pending():
                          ├── suppress_stdout() → prevents TUI corruption
                          ├── dispatch_command(Commands::Install/Remove/Update)
                          └── refresh_locks() → re-read lockfiles, update rows
```

**stdout suppression:** Uses `libc::dup/dup2` to redirect stdout to `/dev/null` during `dispatch_command` calls, preventing progress bars and print statements from corrupting the TUI render.

### 3.5 Registry Generation Flow

```
cli.rs:dispatch_command(Commands::GenerateRegistry)
  └── generate_registry(catalog_arg)
        ├── resolve_skills_root()
        ├── discover_skills()           → Vec<SkillInfo>
        ├── load_category_metadata()   ← reads _category.json from skills root
        ├── load_deprecated()          ← reads deprecated.yaml from skills root
        ├── for each skill:
        │     files_and_hash()         ← includes skill.manifest.json in hash
        │     to_slug()                ← normalizes name to kebab-case
        │     skill_relative_path()    ← builds "(category)/name" path
        └── serialize SkillsRegistry to JSON
```

Output: `skills-registry.json` with `version: "1.0.0"`, `categories{}`, `skills[]`, optional `deprecated[]`.

---

## 4. Storage Layout

| Path | Scope | Purpose |
|------|-------|---------|
| `<catalog>/skills/<name>/` | Catalog | Source skill files (SKILL.md, skill.manifest.json, assets) |
| `<catalog>/skills/` | Catalog | Alternate skills root (if `skills/` subdirectory exists) |
| `~/.cache/ai-workflow-skills/catalog-git/<hash>/` | Global | Cloned remote Git catalog (SHA-256 of URL, first 16 hex chars) |
| `<project>/.agents/.skill-lock.json` | Project | Local install lockfile |
| `<project>/.agents/skills/<name>/` | Project | Canonical copy for local symlink installs |
| `<agent-dir>/skills/<name>/` | Project | Actual installed skill (copy or symlink target) |
| `~/.ai-workflow-skills/.agents/.skill-lock.json` | Global | Global install lockfile |
| `~/.ai-workflow-skills/audit.log` | Global | Audit log (append-only NDJSON) |
| `~/.cache/ai-workflow-skills/registry.json` | Global | Cached skills registry |

**Agent-specific install dirs** (from `src/agents.rs`):

| Agent | Local Skills Dir | Global Skills Dir |
|-------|-----------------|-------------------|
| Cursor | `.cursor/skills/` | `~/.cursor/skills/` |
| Claude Code | `.claude/skills/` | `~/.claude/skills/` |
| Windsurf | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| Cline | `.claude/skills/` | `~/.claude/skills/` |
| Goose | `.config/goose/skills/` | `~/.config/goose/skills/` |
| Continue | `.continue/skills/` | `~/.continue/skills/` |
| Devin | `.devin/skills/` | `~/.devin/skills/` |
| Aider | `.aider.chat.d/` | `~/.aider.chat.d/` |
| Roo Code | `.continue/skills/` | `~/.continue/skills/` |
| Copilot | (custom per-user) | (custom per-user) |
| Zed | `.zed/skills/` | `~/.zed/skills/` |
| Augment | `.augment/skill/` | `~/.augment/skill/` |
| Sourcegraph | `.sourcegraph/cody/commands/` | `~/.sourcegraph/cody/commands/` |
| Codeium | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| Tabnine | `.tabnine/skills/` | `~/.tabnine/skills/` |
| JetBrains | `.idea/Cody/skills/` | `~/.idea/Cody/skills/` |
| Neovim | `.config/nvim/cody/` | `~/.config/nvim/cody/` |

---

## 5. Module Boundaries and Coupling

### 5.1 CLI → Core / Catalog

`cli.rs` depends directly on `core::installer`, `core::lockfile`, `core::audit`, `core::cache`, and `catalog::remote`. No intermediate abstractions.

```
cli.rs
├── core/installer.rs   install_skills(), remove_skills()
├── core/lockfile.rs    read_skill_lock(), write_skill_lock(), add_skill_to_lock(), remove_agent_from_lock(), get_skill_from_lock()
├── core/audit.rs       log_audit(), read_audit_log()
├── core/cache.rs       cache_dir(), registry_cache_path(), clear_all_cache(), clear_registry_cache()
└── catalog/remote.rs   resolve_remote_or_local(), is_remote_catalog(), sync_git_catalog()
```

### 5.2 Core → Catalog

`core/installer.rs` depends on `catalog/discover.rs` only indirectly — `SkillInfo` is passed in but the installer does not call catalog functions. The lockfile module has no catalog dependency.

```
core/installer.rs
└── catalog/discover.rs  (only via SkillInfo type, not function calls)
```

**CLI → Remote catalog:** `cli.rs` calls `catalog/remote.rs` to resolve remote Git URLs before catalog operations.

```
cli.rs
└── catalog/remote.rs  resolve_remote_or_local(), is_remote_catalog(), sync_git_catalog()
```

### 5.3 UI → CLI

`ui/app.rs` calls `dispatch_command()` directly for install/remove operations. It re-reads lockfiles to refresh state after operations.

```
ui/app.rs
├── cli.rs             dispatch_command(), install_with_progress_bar()
├── core/lockfile.rs  read_skill_lock()
└── core/installer.rs remove_skills()
```

### 5.4 Shared Type Flow

```
types.rs (SkillInfo, InstallOptions, InstallResult, AuditEntry, etc.)
    ↑ flows into all layers
```

`types.rs` is the only type module. It is imported by every other module. There is no separation between domain types and cross-cutting types.

---

## 6. Security Model

**Path traversal guard** — `src/sanitize.rs:is_path_safe()`:
- Para cada caminho, resolve o maior prefixo que já existe em disco, `dunce::canonicalize` nesse prefixo e reaplica o sufixo (permite destinos cuja pasta final ainda não existe, ex.: nova skill).
- Compara os dois caminhos lógicos com `starts_with` (trecho existente continua com resolução de symlinks).
- Fallback (caminhos inválidos ou sem prefixo canonicalizável): prefixo literal em stderr e verificação fraca — caso raro.
- Prevents installing or removing skills outside the intended agent directory

**Install path validation** — `src/core/installer.rs:validate_install_path()`:
- Enforces `is_path_safe(target_dir, skill_target)` before any filesystem operation
- Enforces `is_path_safe(project_root, skill_target)` as a second boundary
- Returns an error string (not a bool) describing the security violation

**Remove path validation** — same check in `remove_skills()` for each path before removal.

**Skill name sanitization** — `src/sanitize.rs:sanitize_name()`:
- Strips `/`, `\`, `*`, `:`, `?`, `"`, `<`, `>`, `|`, NUL
- Removes `..` sequences iteratively
- Trims leading/trailing `.` and whitespace
- Limits to 255 characters
- Returns `"unnamed-skill"` for empty result

**Network access via git subprocess only.** The tool does not make HTTP requests directly. Remote catalog URLs are resolved by spawning `git clone --depth 1` (first time) or `git pull --ff-only` (subsequent) as a subprocess. Optional `AI_WORKFLOW_SKILLS_GIT_TOKEN` is injected into HTTPS URLs for private repositories.

**Hash coverage:** `content_hash` (SHA-256) covers all skill files including `skill.manifest.json`. Modifying an installed skill file changes the hash — but the installed copy is **not** auto-updated.

---

## 7. Concurrency and Consistency

**Transactional boundary: none.** If `lockfile::add_skill_to_lock()` fails after `copy_dir_all()` succeeds, the filesystem has the skill but the lockfile does not record it. Conversely, if the lockfile write succeeds but the audit log write fails, the lockfile is updated but the audit entry is lost.

**Write locking via flock.** `src/core/lockfile.rs:flock_exclusive()` acquires an exclusive `flock(2)` around the entire read-modify-write cycle during `write_skill_lock()`. The read path (`read_skill_lock()`) is not locked — concurrent reads during writes may observe partial state.

**Backup on write** — `lockfile.rs:write_skill_lock()` copies the existing file to `.skill-lock.json.backup` before writing the new version.

---

## 8. Design Observations

| Observation | Impact | Location |
|-------------|--------|----------|
| `cli.rs` handles catalog resolution, agent detection, progress bars, and output formatting in addition to command dispatch | Large function with many responsibilities; difficult to test in isolation | `src/cli.rs` (~450 lines) |
| `add_skill_to_lock()` uses `unwrap_or_default()` for `agents` field | Null/missing `agents` field in lockfile is silently treated as empty; no schema validation | `src/core/lockfile.rs` |
| Lockfile key is skill name only | Two skills with the same name in different categories would overwrite each other in the lockfile | `src/types.rs:10–13` |
| `dispatch_command()` uses `std::process::exit(1)` on failure | Prevents callers (TUI, tests) from handling failures gracefully; cannot be used as a library | `src/cli.rs` |
| `AuditEntry` has no schema version in the log | New struct fields are silently ignored on read; removed fields cause parse failures (skipped with warning) | `src/core/audit.rs:read_audit_log()` |
| `SkillInfo.path` is absolute | Catalog relocation breaks discovered skills; no catalog relocation safety | `src/catalog/discover.rs:try_read_skill()` |
| UI calls `dispatch_command()` for installs | TTY mode re-enters the CLI command layer rather than calling `install_skills()` directly | `src/ui/app.rs:apply_install()` |
| `lib.rs` re-exports all types and modules | Acts as a facade; no deep hierarchy but makes the public API explicit | `src/lib.rs` |
| `SKILL.md` frontmatter parsed with regex | Regex assumes specific YAML structure; malformed frontmatter silently degrades | `src/catalog/discover.rs:parse_frontmatter()` |
| Remote catalog sync uses `git` subprocess | No in-process HTTP client; relies on system `git` binary; catalog cloned to cache | `src/catalog/remote.rs` |
