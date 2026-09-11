# Integrations — `ai-workflow-skills`

> Documented from code inspection of `src/constants.rs`, `src/core/lockfile.rs`, `src/core/audit.rs`, `src/core/cache.rs`, `src/cli.rs`, `src/types.rs`, and `src/catalog/discover.rs`.

---

## 1. External Services

**Git (subprocess)** when `AI_WORKFLOW_SKILLS_CATALOG` / `--catalog` is a remote URL (`https://`, `git@`, etc.): `git clone` / `git pull --ff-only` into `~/.cache/ai-workflow-skills/catalog-git/<hash>/`. No in-process HTTP client; network access is via the `git` binary.

Optional **`AI_WORKFLOW_SKILLS_GIT_TOKEN`**: injected into HTTPS URLs for private Git remotes (`oauth2:` token prefix; `src/catalog/remote.rs`).

Evidence: `src/catalog/remote.rs`; `Cargo.toml` — still no `reqwest`/`http` crates.

---

## 2. Environment Variables

| Variable | Purpose | CLI Arg | Evidence |
|---|---|---|---|
| `AI_WORKFLOW_SKILLS_CATALOG` | Local path **or** Git URL of the skills catalog | `--catalog` (`PATH_OR_URL`) | `src/constants.rs`, `src/cli.rs`, `src/catalog/remote.rs` |
| `AI_WORKFLOW_SKILLS_GIT_TOKEN` | Optional; for HTTPS private Git remotes (`oauth2:<token>@` injection) | — | `src/catalog/remote.rs` |

Other env vars: third-party crates (`dirs`, `clap`).

Evidence: `grep -n "env::var\|ENV_" src/catalog/remote.rs`, `src/cli.rs`.

---

## 3. Filesystem Interactions

### 3.1 Home Directory (`$HOME`)

The tool requires `HOME` to be set. Used for:

| Path | Purpose | Code |
|---|---|---|
| `~/.ai-workflow-skills/audit.log` | Audit trail (JSONL, one `AuditEntry` per line) | `src/constants.rs:19`, `src/core/audit.rs:7` |
| `~/.cache/ai-workflow-skills/registry.json` | Cached catalog registry | `src/constants.rs:21–22`, `src/core/cache.rs:14` |
| `~/.agents/.skill-lock.json` | Global lockfile (v2) | `src/constants.rs:14,18`, `src/core/lockfile.rs:9–12` |

**Fallback:** If `HOME` is unset, the CLI errors with `"HOME não definido"`.

Evidence: `src/cli.rs:267` — `dirs::home_dir().ok_or_else(...)`.

### 3.2 Project / Working Directory

| Path | Purpose | Code |
|---|---|---|
| `<cwd>/.agents/.skill-lock.json` | Local lockfile (per-project) | `src/core/lockfile.rs:13` |
| `<cwd>/.agents/skills/` | Canonical copy of skills (intermediate staging for symlink installs) | `src/constants.rs:15` |
| `<agent>/skills/` | Agent-specific skill installations | `src/core/installer.rs` |

Lockfile resolution (global vs. local) is determined by the `--global` flag.

Evidence: `src/core/lockfile.rs:9–14` — `lock_path` branches on `global: bool`.

### 3.3 Catalog Directory

The catalog is a user-supplied local directory (defaulted via `--catalog` or `AI_WORKFLOW_SKILLS_CATALOG`). Expected layout:

```
<catalog>/                    OR  <catalog>/skills/
├── _category.json            ├── _category.json
├── (category-id)/            ├── (category-id)/
│   └── <skill>/             │   └── <skill>/
│       ├── SKILL.md             │   ├── SKILL.md
│       └── skill.manifest.json │   └── skill.manifest.json
├── deprecated.yaml
└── <uncategorized-skills>/
    └── <skill>/
```

The tool resolves `skills/` subdirectory or uses the directory directly if it matches the above layout.

Evidence: `src/catalog/discover.rs:63–80` — `resolve_skills_root` function.

### 3.4 Agent Skill Installation Targets

Skills are installed into agent-specific directories. Supported agents are hardcoded in `src/agents.rs`. Default is **cursor** if no agents are detected or specified.

Evidence: `src/cli.rs:196–200`, `src/agents.rs`.

### 3.5 Ignored Files in Content Hash

The following files are excluded from SHA-256 content hashing when computing `content_hash`:

| File | Reason |
|---|---|
| `.DS_Store` | macOS filesystem artifact |
| `.gitkeep` | Version control placeholder |
| `Thumbs.db` | Windows filesystem artifact |
| `.gitignore` | VCS metadata |

Evidence: `src/catalog/discover.rs:39` — `const IGNORED: &[&str]`.

---

## 4. External Tooling Integration

**NONE.** No git hooks, no shell completions, no wrapper scripts, no plugin interfaces.

The tool only:
1. Reads from the filesystem (catalog, `HOME`, working directory).
2. Writes to the filesystem (installation targets, lockfile, audit log, cache).
3. Exposes a CLI with no daemon or background component.

Evidence: `src/cli.rs` — no subprocess spawning, no external binary invocation.

---

## 5. Lockfile Format and Schema

**File:** `.skill-lock.json` (local in `<cwd>/.agents/`, global in `~/.agents/`)

**Version:** `2` (integer, constant `LOCKFILE_VERSION`)

### Top-Level (`SkillLockFile`)

```json
{
  "version": 2,
  "skills": { "<skill-name>": <SkillLockEntry> }
}
```

### Entry (`SkillLockEntry`)

```json
{
  "name": "string",
  "source": "string",
  "content_hash": "sha256-hex-string | null",
  "installed_at": "RFC3339 timestamp",
  "updated_at": "RFC3339 timestamp",
  "agents": ["string"] | null,
  "method": "\"copy\" | \"symlink\" | null",
  "global": "boolean | null",
  "version": "SemVer string | null"
}
```

| Field | Required | Default | Notes |
|---|---|---|---|
| `version` | Yes | — | Always `2` in current code. Auto-migrates v1 → v2. |
| `name` | Yes | — | Skill name (slug). |
| `source` | No | `"local"` | [INFERRED] Origin of the skill in the catalog. |
| `content_hash` | No | `null` | SHA-256 hex of skill content. Compared against catalog hash to detect updates. |
| `installed_at` | Yes | — | ISO 8601 / RFC3339 timestamp of initial installation. |
| `updated_at` | Yes | — | RFC3339 timestamp of last update. |
| `agents` | No | `null` | List of agent names the skill is installed for. |
| `method` | No | `"copy"` (after migration) | `copy` or `symlink`. |
| `global` | No | `false` (after migration) | Whether installed globally vs. per-project. |
| `version` | No | `null` | Semantic version from `skill.manifest.json`. |

### Backup

On write, the previous lockfile is backed up to `.skill-lock.json.backup` in the same directory.

Evidence: `src/core/lockfile.rs:17–22` — `backup_path`, `src/core/lockfile.rs:74–75` — copy before rename.

### Migration

The code migrates v1 lockfiles (missing `method`/`global` fields) on read by injecting defaults. This is a one-way upgrade.

Evidence: `src/core/lockfile.rs:44–54` — `migrate` function.

---

## 6. Audit Log Format

**File:** `~/.ai-workflow-skills/audit.log`

**Format:** Line-delimited JSON (JSONL), one `AuditEntry` per line.

```json
{
  "action": "install | remove | update",
  "skillName": "string",
  "agents": ["string"],
  "success": 1,
  "failed": 0,
  "forced": "boolean | null",
  "details": "JSON value | null",
  "timestamp": "RFC3339 string | null"
}
```

Evidence: `src/types.rs:67–78` — `AuditEntry` struct with `#[serde(rename_all = "camelCase")]`.

Invalid lines are skipped with a warning to stderr. Empty log returns an empty vector.

Evidence: `src/core/audit.rs:32–39` — skips `Err(_)` parse failures.

---

## 7. Generated Registry Format

The `generate-registry` command produces a `skills-registry.json` file (default path `skills-registry.json`):

```json
{
  "version": "1.0.0",
  "categories": {
    "<category-id>": { "name": "string", "description": "string | null" }
  },
  "skills": [
    {
      "name": "string",
      "description": "string",
      "category": "string",
      "path": "string",
      "files": ["string"],
      "author": "string | null",
      "version": "string | null",
      "tags": ["string"] | null,
      "contentHash": "sha256-hex-string"
    }
  ],
  "deprecated": [{ "name": "...", "message": "...", "alternatives": [...] }] | null
}
```

Evidence: `src/catalog/registry.rs:18–35` — `SkillsRegistry` and `SkillMetadata` structs.

---

## 8. Skill Manifest Schema

Each skill must contain `skill.manifest.json`:

```json
{
  "version": "1.0.0",
  "tags": ["devops", "api-rest"]
}
```

| Field | Required | Validation |
|---|---|---|
| `version` | Yes | Must be valid SemVer. |
| `tags` | No | Kebab-case ASCII strings, max 48 chars each. |

Evidence: `src/catalog/discover.rs:45–55` — `SkillManifest` struct; `src/catalog/discover.rs:58–72` — `normalize_tags`.

---

## 9. Summary

`ai-workflow-skills` is a self-contained local tool. All integration surface is filesystem-based:

| Surface | Type | Bounded by |
|---|---|---|
| `AI_WORKFLOW_SKILLS_CATALOG` | Env var | User-supplied path |
| `$HOME/.ai-workflow-skills/` | Directory | Configurable via `HOME` |
| `$HOME/.cache/ai-workflow-skills/` | Directory | Configurable via `HOME` |
| `$HOME/.agents/` | Directory | Configurable via `HOME` |
| `<cwd>/.agents/` | Directory | Found via project root detection |
| `<agent>/skills/` | Directory | Depends on detected/requested agents |

No external services, no network, no external tooling. The only runtime dependency on external state is `HOME` and the catalog directory.
