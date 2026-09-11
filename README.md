<div align="center">

# ai-workflow-skills

**A versioned catalog of reusable AI agent skills, and a fast Rust CLI to install them into any project.**

Works with Cursor, Claude Code, GitHub Copilot, Windsurf, Cline, Aider, Codex, Gemini CLI, and more.

[![CI](https://github.com/watasabi/ai-workflow-skills/actions/workflows/ci.yml/badge.svg)](https://github.com/watasabi/ai-workflow-skills/actions/workflows/ci.yml)
[![Release](https://github.com/watasabi/ai-workflow-skills/actions/workflows/release.yml/badge.svg)](https://github.com/watasabi/ai-workflow-skills/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

[Website](https://watasabi.github.io/ai-workflow-skills/) · [Quickstart](#quickstart) · [Catalog](#catalog) · [CLI reference](#cli-reference)

</div>

---

## Why

"Skills" — a folder with a `SKILL.md` describing when and how an AI agent should do something — are a great way to give coding agents durable, reviewable knowledge about a codebase or a workflow. But once you have more than one or two, you need a way to **version, share, and install** them across projects and across agents.

`ai-workflow-skills` is that layer:

- 📦 **Catalog** — skills live as plain folders (`SKILL.md` + `references/`), versioned in a local directory or a Git repo.
- 🔌 **Multi-agent** — installs to the right folder for 19+ agents (`.cursor/skills/`, `.claude/skills/`, `.github/skills/`, …), with auto-detection.
- 🔒 **Tracked installs** — every install is recorded in a project-local lockfile (`.agents/.skill-lock.json`) with content hashes, so `update` only touches what actually changed.
- 🖥️ **Interactive TUI** — browse, install, update, and remove skills without memorizing flags.
- 🧾 **Auditable** — every install/update/remove is appended to an audit log.

It follows the same model as [agent-skills](https://github.com/tech-leads-club/agent-skills): a skill is just a directory with a `SKILL.md`. Anything already following that convention works here too.

## Quickstart

```bash
# 1. Install the CLI (see all options below)
cargo install --git https://github.com/watasabi/ai-workflow-skills.git --locked

# 2. Point it at a catalog (this repo ships one)
git clone https://github.com/watasabi/ai-workflow-skills.git
export AI_WORKFLOW_SKILLS_CATALOG="$(pwd)/ai-workflow-skills/catalog"

# 3. Browse and install
ai-workflow-skills list
ai-workflow-skills install -s tool-github-cli -a cursor -a claude-code
```

Or just run `ai-workflow-skills` with no arguments in a terminal for the interactive picker.

---

## Install

### Option A — Prebuilt binary (recommended)

Downloads a release tarball for Linux (x86_64/arm64) or macOS (Intel/Apple Silicon):

```bash
curl -fsSL https://raw.githubusercontent.com/watasabi/ai-workflow-skills/main/install.sh | bash
```

Or from a local clone:

```bash
git clone https://github.com/watasabi/ai-workflow-skills.git
cd ai-workflow-skills
./install.sh
```

| Flag | Meaning |
|------|---------|
| `-v TAG` | Install a specific release (e.g. `v0.3.0`). Default: latest. |
| `-p DIR` | Install prefix; binary goes to `DIR/bin`. Default: `~/.local`. |
| `-c URL` | Writes `export AI_WORKFLOW_SKILLS_CATALOG="URL"` to your shell rc. |

Binary lands at `~/.local/bin/ai-workflow-skills` — make sure that's on your `PATH`.

### Option B — Cargo

Works on any platform Rust supports, no dependency on release tarballs:

```bash
cargo install --git https://github.com/watasabi/ai-workflow-skills.git --locked
```

Or from a local clone:

```bash
git clone https://github.com/watasabi/ai-workflow-skills.git
cd ai-workflow-skills
cargo install --path . --locked
```

### Option C — Build from source

```bash
git clone https://github.com/watasabi/ai-workflow-skills.git
cd ai-workflow-skills
cargo build --release --locked
./target/release/ai-workflow-skills --version
```

Check the install:

```bash
ai-workflow-skills --version
ai-workflow-skills --help
```

---

## Configure the catalog

Every command needs to know **where the skill catalog is**: a local directory or a Git URL.

| Source | Environment variable |
|--------|----------------------|
| Single catalog | `AI_WORKFLOW_SKILLS_CATALOG` |
| Multiple catalogs | `AI_WORKFLOW_SKILLS_CATALOGS` (separate entries with `\|\|\|` or newlines) |

`--catalog` on the CLI overrides these for a single invocation, and can be repeated to merge catalogs.

**This repo's catalog** (recommended starting point):

```bash
export AI_WORKFLOW_SKILLS_CATALOG="/path/to/ai-workflow-skills/catalog"
```

**A remote Git catalog** (your own skills repo):

```bash
export AI_WORKFLOW_SKILLS_CATALOG="https://github.com/your-org/your-skill-catalog.git"
```

Remote catalogs are cloned/updated under `~/.cache/ai-workflow-skills/catalog-git/<hash>/`. For a private catalog over HTTPS, set `AI_WORKFLOW_SKILLS_GIT_TOKEN`; for `git@github.com:...` URLs, your existing SSH key is used.

---

## Catalog

Skills bundled in this repo, grouped by category:

### 🛠️ Tooling

| Skill | Use for |
|---|---|
| `tool-github-cli` | PRs, issues, checks, and the GitHub API via `gh` |
| `tool-databricks` | Databricks CLI: auth, profiles, data exploration, jobs, bundles |

### 💻 Development *(specs for this repo)*

| Skill | Use for |
|---|---|
| `proj-specs-architecture` | Modules, data flow, install pipeline, stack |
| `proj-specs-conventions` | Rust/CLI style, error handling, test layout |
| `proj-specs-changes` | Reading/continuing planned changes in `.specs/changes/` |
| `proj-specs-audits` | Resolving audit ledger claims |
| `proj-specs-debug` | Known risks, race conditions, path-safety concerns |
| `proj-specs-testing` | Adding tests, running CI checks locally |

### 🔬 Research & Data Science

| Skill | Use for |
|---|---|
| `ds-template-workflow` | Navigating projects generated from [`ds-template-v2`](https://github.com/watasabi/ds-template-v2): folder layout, UV workspaces, Ruff conventions, commit style |
| `nursing-health-reports` | Clinical/nursing cohort analytics: descriptive stats, association tests, PhiK matrices, dual HTML reports with plain-language guides |

See what's available and their full descriptions at any time:

```bash
ai-workflow-skills list --catalog "/path/to/ai-workflow-skills/catalog"
```

Bring your own skills by adding a folder with a `SKILL.md` under `catalog/skills/(category)/your-skill/` — no code changes needed.

---

## Usage

### Interactive TUI

Run with no subcommand in a terminal:

```bash
export AI_WORKFLOW_SKILLS_CATALOG="/path/to/ai-workflow-skills/catalog"
ai-workflow-skills
```

| Key | Action |
|-----|--------|
| ↑ / ↓ | Move in list |
| **p** | Install to project |
| **g** | Install globally (`~`) |
| **u** | Update |
| **r** | Remove |
| **c** | Clear action on row |
| Enter | Apply |
| q / Esc | Quit |

Non-interactive environments (CI, pipes) print help instead — use the subcommands below.

### CLI reference

| Command | Purpose |
|---------|---------|
| `list` | List catalog skills, or `--installed` for the lockfile |
| `install` | Install one or more skills (`-s`, repeatable) |
| `update` | Refresh installed skills when the catalog changed |
| `remove` / `rm` | Remove installed skills |
| `cache` | Inspect/clear `~/.cache/ai-workflow-skills` |
| `audit` | Show recent audit log entries |
| `generate-registry` | Emit `skills-registry.json` from the catalog |

Common flags:

- `--catalog PATH_OR_URL` — repeat to merge several catalogs
- `-s / --skill NAME` — required for `install`/`remove` outside the TUI
- `-a / --agent ID` — target agent(s): `cursor`, `claude-code`, `windsurf`, `github-copilot`, … (repeatable; default: auto-detect, falling back to `cursor`)
- `-g / --global` — install/remove at the user level instead of the project
- `--symlink` — link instead of copy
- `-f / --force` — reinstall even if unchanged

If a skill name exists in more than one catalog, disambiguate with `catalog-label/skill-name`.

### Example session

```bash
git clone https://github.com/watasabi/ai-workflow-skills.git
cd ai-workflow-skills
export AI_WORKFLOW_SKILLS_CATALOG="$(pwd)/catalog"

ai-workflow-skills list

ai-workflow-skills install -s tool-github-cli -s tool-databricks -a cursor -a claude-code

ai-workflow-skills list --installed

ai-workflow-skills update -s tool-github-cli

ai-workflow-skills remove -s tool-github-cli
```

---

## Supported agents

`cursor` · `claude-code` · `github-copilot` · `windsurf` · `cline` · `aider` · `codex` · `gemini` · `antigravity` · `roo` · `kilocode` · `trae` · `kiro` · `amazon-q` · `augment` · `tabnine` · `opencode` · `sourcegraph` · `droid`

Each maps to its own project/global skills folder (e.g. `.cursor/skills/`, `.claude/skills/`, `.windsurf/skills/`) and is auto-detected from existing config folders when `-a` is omitted.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Skill authoring conventions live in [SCOPE.md](SCOPE.md) and the `proj-specs-*` skills in this catalog.

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE), at your option.
