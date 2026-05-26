---
name: ai-workflow-skills
description: "Rust CLI tool for managing AI agent skills from local/Git catalogs. Supports 20+ agents (Cursor, Claude Code, Copilot, Windsurf). For features: installing/listing/updating skills, lockfile tracking, audit logging, interactive TUI. See AGENTS.md for deep technical details."
---

# ai-workflow-skills — Workspace Instructions

A **Rust CLI tool** to install and manage **AI agent skills** from versioned catalogs (local directories or Git repositories). Skills are stored as folders with `SKILL.md` files, and installations are tracked in `.agents/.skill-lock.json`.

---

## Quick Start

### 1. **Install the Tool**

Choose one method:

#### Quick install (Linux x86_64, macOS):
```bash
export GITHUB_REPOSITORY="watasabi/ai-workflow-skills"   # or your fork
curl -fsSL "https://raw.githubusercontent.com/watasabi/ai-workflow-skills/main/install.sh" | bash -s --
```

#### From this repository (all platforms):
```bash
cargo install --path .              # from a clone
# or
cargo install --git https://github.com/watasabi/ai-workflow-skills.git --locked
```

**Result:** Binary installed as `ai-workflow-skills` (usually in `~/.cargo/bin` or `~/.local/bin`).

---

### 2. **Configure a Catalog**

Every command needs to know where skills are stored. Set one or more:

```bash
export AI_WORKFLOW_SKILLS_CATALOG="/path/to/local/catalog"
# or
export AI_WORKFLOW_SKILLS_CATALOG="https://github.com/org/skills-repo.git"
# or (multiple catalogs):
export AI_WORKFLOW_SKILLS_CATALOGS="/local/path|||https://remote.git"
```

Remote catalogs are cloned to `~/.cache/ai-workflow-skills/catalog-git/<hash>/`.

---

### 3. **Try the TUI (Interactive Menus)**

```bash
ai-workflow-skills tui
```

**What you'll see:**
- **Left pane**: List of available skills from the catalog
- **Right pane**: Details, metadata, and action buttons
- **Navigation**: Arrow keys to browse, Enter to select, `q` to quit
- **Commands**: Install, update, remove skills with visual feedback

This is the recommended way to explore and manage skills interactively.

---

### 4. **Common Commands**

```bash
# List all available skills in the catalog
ai-workflow-skills list

# List installed skills (from .agents/.skill-lock.json)
ai-workflow-skills list --installed

# Install a skill (default: Cursor at .cursor/skills/)
ai-workflow-skills install -s <skill-name>

# Install for a specific agent
ai-workflow-skills install -s <skill-name> -a claude-code
ai-workflow-skills install -s <skill-name> -a windsurf

# Install globally (all users)
ai-workflow-skills install -s <skill-name> --global

# Update a skill
ai-workflow-skills update -s <skill-name>

# Remove a skill
ai-workflow-skills remove -s <skill-name>

# Check activity log
ai-workflow-skills audit
```

---

## Project Structure

See [AGENTS.md](AGENTS.md) for **full technical architecture**, including:
- Data flow diagrams (CLI → catalog → installer → lockfile tracking)
- Component organization (`src/core/`, `src/catalog/`, `src/ui/`)
- Lockfile format and audit logging details
- Type system and error handling patterns

**Key directories:**
| Dir | Purpose |
|-----|---------|
| `src/` | CLI, business logic, core types |
| `src/core/` | Installer, lockfile, audit log, cache |
| `src/catalog/` | Skill discovery, registry generation |
| `src/ui/` | Interactive TUI (ratatui + crossterm) |
| `tests/` | Integration tests (assert_cmd, tempfile) |
| `.githooks/` | Git hooks: pre-commit (fmt/clippy/test), commit-msg (Conventional Commits) |

---

## Development Commands

### Setup
```bash
# Install pre-commit hooks (recommended)
git config core.hooksPath .githooks

# Build
cargo build --release        # Release binary
cargo build --locked         # Use locked dependencies (CI default)
```

### Test & Lint (CI gates)
```bash
cargo fmt --check            # Check code formatting
cargo fmt                    # Auto-format
cargo clippy --all-targets --all-features -- -D warnings
cargo test                   # Run all tests
cargo test <name>           # Run specific test
```

### Run
```bash
cargo run -- --help         # Show CLI help
cargo run -- list           # List skills from catalog
cargo run -- tui            # Launch interactive TUI
```

---

## Common Workflows

### **Explore Skills in Catalog**
```bash
# Via CLI (output to terminal)
ai-workflow-skills list --catalog /path/to/catalog

# Via TUI (interactive, recommended)
ai-workflow-skills tui
```

### **Install a Skill for Your IDE**

**For Cursor:**
```bash
ai-workflow-skills install -s my-skill
# Installs to: .cursor/skills/my-skill/
```

**For Claude Code / Other agents:**
```bash
ai-workflow-skills install -s my-skill -a claude-code
ai-workflow-skills install -s my-skill -a windsurf
```

**With custom install location:**
- Use `--catalog` to specify a different catalog source
- Use `--global` to install to `~/.ai-workflow-skills/agents/<agent>/` (all projects)
- Use `--symlink` to symlink instead of copy (prefer when possible)

### **Verify Installation**

Check what's installed:
```bash
ai-workflow-skills list --installed
```

View the lockfile:
```bash
cat .agents/.skill-lock.json
```

Check the audit log:
```bash
ai-workflow-skills audit -n 10
```

### **Update Skills**

Check for updates:
```bash
ai-workflow-skills update -s my-skill
```

The tool compares content hashes (SHA256) and reinstalls if the catalog version differs.

### **Remove Skills**

```bash
ai-workflow-skills remove -s my-skill
```

Or via TUI: navigate to the skill and press `r` (or button).

---

## Configuration

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `AI_WORKFLOW_SKILLS_CATALOG` | Single catalog source | `/home/me/catalog` or `https://github.com/org/repo.git` |
| `AI_WORKFLOW_SKILLS_CATALOGS` | Multiple catalogs (separated by `\|\|\|`) | `/local\|\|\|https://remote.git` |
| `AI_WORKFLOW_SKILLS_GIT_TOKEN` | Token for private Git remotes | `ghp_xxx...` |
| `HOME` | Used for: lockfile path, cache, audit log | (system) |

### Lockfile Schema

Location: `.agents/.skill-lock.json` (v2 schema)

```json
{
  "version": 2,
  "agent": "cursor",
  "skills": [
    {
      "name": "my-skill",
      "version": "1.2.3",
      "content_hash": "abc123...",
      "installed_at": "2025-01-15T10:30:00Z",
      "path": ".cursor/skills/my-skill"
    }
  ]
}
```

Install/remove operations atomically update this via file locking (`flock`).

### Audit Log

Location: `~/.ai-workflow-skills/audit.log` (JSON lines, append-only)

Each operation is logged with timestamp, action, agent, skill name, and install path. Use `ai-workflow-skills audit` to view.

---

## Architecture Overview

**Data flow:**
```
CLI input → dispatch_command() → catalog discovery → install/remove/update
     ↓                                    ↓                      ↓
    clap parsing          discover_skills() → Vec<SkillInfo>   lockfile update
                          (walk, parse metadata)                (atomic flock write)
                                                                    ↓
                                                              audit log entry
```

**Key components:**
- **CLI** (`src/cli.rs`): clap-based command definition
- **Catalog** (`src/catalog/`): Discovery and registry generation
- **Core** (`src/core/`): Install logic, lockfile management, audit logging
- **UI** (`src/ui/`): ratatui TUI with crossterm backend

For deep technical details, see [AGENTS.md](AGENTS.md).

---

## Conventions

### **Error Handling**
- All functions return `anyhow::Result<T>`
- Use `.context()` or `.with_context(|| "...")` for actionable messages
- anyhow handles all error types (no custom error enums)

### **Path Safety**
- Always call `sanitize_name()`, `is_path_safe()`, `to_slug()` before filesystem operations
- Symlink-aware via canonicalization

### **Commit Format**
Uses [Conventional Commits](https://www.conventionalcommits.org/):
```
<type>(<scope>)?: <description>

[optional body]
```

Valid types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `imp`, `infra`, `chore`, `breaking`

Examples:
```bash
git commit -m "feat: add Windsurf agent support"
git commit -m "fix(catalog): handle missing SKILL.md gracefully"
```

Pre-commit hook enforces this if you run: `git config core.hooksPath .githooks`

---

## Troubleshooting

### "Command not found: ai-workflow-skills"
- Ensure the binary is installed: `which ai-workflow-skills`
- Check that installation prefix is on `PATH`: `echo $PATH | grep ~/.cargo/bin` or `grep ~/.local/bin`
- Reinstall if needed: `cargo install --path .`

### "No catalog configured"
- Set `AI_WORKFLOW_SKILLS_CATALOG` or `AI_WORKFLOW_SKILLS_CATALOGS`
- Or pass `--catalog` with each command: `ai-workflow-skills list --catalog /path/to/catalog`

### "Failed to discover skills"
- Verify catalog path exists: `ls /path/to/catalog`
- Check for valid `SKILL.md` files: `find /path -name SKILL.md`
- View debug output: The tool will show which skills failed to parse

### "Install failed: path not safe"
- Verify no symlink traversals in the target install path
- Use absolute paths, avoid `../`

### "Lockfile conflict"
- The tool uses file locking to prevent concurrent writes
- If stuck, manually remove `.agents/.skill-lock.json` (it will be regenerated)

---

## Related Documentation

- **[AGENTS.md](AGENTS.md)** — Complete technical architecture, data flow, component design, API reference
- **[README.md](README.md)** — User-facing installation and configuration
- **[CONTRIBUTING.md](CONTRIBUTING.md)** — Development setup, testing, CI
- **[SCOPE.md](SCOPE.md)** — Command matrix and compatibility notes
- **Git hooks** — `.githooks/pre-commit` (fmt + clippy + test), `.githooks/commit-msg` (Conventional Commits)

---

## Next Steps

1. **Install the tool** — Pick an installation method from "Quick Start"
2. **Set up a catalog** — Configure `AI_WORKFLOW_SKILLS_CATALOG` to point to your skills
3. **Try the TUI** — Run `ai-workflow-skills tui` to see the interactive menu
4. **Install skills** — Browse and select skills from the menu, or use CLI commands
5. **Review lockfile** — Check `.agents/.skill-lock.json` to see what's installed
6. **Explore advanced features** — Use `--symlink` for development, `--global` for team installs, or `--force` to override conflicts

---

**For development:** See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for architecture details, testing strategy, and contribution guidelines.
