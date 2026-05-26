# ai-workflow-skills

Rust CLI to install and manage **AI agent skills** from a versioned local or Git catalog. It follows the same idea as [agent-skills](https://github.com/tech-leads-club/agent-skills): skills are stored as folders with `SKILL.md`, and project installs are tracked in `.agents/.skill-lock.json`.

Skills are installed for **Cursor** under `.cursor/skills/` by default. Other agents are supported when you pass `--agent` or when the tool detects them in your project or home directory.

---

## Install

This repository is **private**. You cannot pipe `install.sh` from `raw.githubusercontent.com` without authentication (that URL returns 404 for private repos). **Clone the repo first**, then install from the checkout.

Set your org/repo once (default in `install.sh` is `watasabi/ai-workflow-skills`):

```bash
export GITHUB_REPOSITORY="YOUR_ORG/ai-workflow-skills"
```

### Authentication (private GitHub)

| Method | Use for |
|--------|---------|
| **SSH** (`git@github.com:ORG/ai-workflow-skills.git`) | `git clone`, `cargo install --git git@github.com:…` — recommended if you already use SSH keys |
| **HTTPS + token** | `install.sh` (`-t` / `GITHUB_TOKEN`), release downloads, `cargo install --git https://…` |

For HTTPS, use a [fine-grained](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens) or classic PAT with at least **Contents: Read** on this repository. Export it (do not commit it):

```bash
export GITHUB_TOKEN="ghp_…"   # or GH_TOKEN; also used by install.sh
```

`install.sh` and the CLI use the token only locally (API, release assets, optional `git clone` on macOS). For **remote skill catalogs** over HTTPS, see [`AI_WORKFLOW_SKILLS_GIT_TOKEN`](#configure-the-catalog) below.

### Option A: Clone + Cargo (recommended)

Works on every platform (Linux x86_64, ARM64, macOS) and does not depend on GitHub Releases.

```bash
# SSH (recommended)
git clone git@github.com:YOUR_ORG/ai-workflow-skills.git
cd ai-workflow-skills

# Or HTTPS with a PAT embedded once for clone (prefer SSH or gh auth instead)
# git clone https://github.com/YOUR_ORG/ai-workflow-skills.git

cargo install --path . --locked
ai-workflow-skills --version
```

Binary: `~/.cargo/bin/ai-workflow-skills` (ensure `~/.cargo/bin` is on your `PATH`).

**Install from Git without a full clone** (HTTPS private repo):

```bash
export GITHUB_TOKEN="ghp_…"
cargo install --git "https://github.com/${GITHUB_REPOSITORY}.git" --locked
```

Or with SSH:

```bash
cargo install --git "git@github.com:YOUR_ORG/ai-workflow-skills.git" --locked
```

### Option B: Clone + install script

Run [`install.sh`](install.sh) from a local checkout. On **Linux x86_64** it downloads a release tarball; on **macOS** it clones the tagged repo and runs `cargo build --release`. Both paths need a token for a **private** repo when resolving releases or cloning.

```bash
git clone git@github.com:YOUR_ORG/ai-workflow-skills.git
cd ai-workflow-skills

export GITHUB_REPOSITORY="YOUR_ORG/ai-workflow-skills"
export GITHUB_TOKEN="ghp_…"    # required for private API + release assets

./install.sh -t "$GITHUB_TOKEN"
```

| Flag | Meaning |
|------|---------|
| `-v TAG` | Install a specific release (e.g. `v0.3.0`). Default: latest GitHub release. |
| `-p DIR` | Install prefix; binaries go to `DIR/bin`. Default: `~/.local`. |
| `-t TOKEN` | Same as `GITHUB_TOKEN` / `GH_TOKEN` (required for private repos on Linux release download). |
| `-c URL` | Writes `export AI_WORKFLOW_SKILLS_CATALOG="URL"` to your shell rc (optional). |

Point the catalog at this repo’s bundled skills (local path, no Git token needed):

```bash
./install.sh -t "$GITHUB_TOKEN" \
  -c "$(pwd)/catalog"
```

On **Linux ARM64** (and other arches without a release tarball), use [Option A](#option-a-clone--cargo-recommended) instead.

### Option C: Build from source (no install)

```bash
git clone git@github.com:YOUR_ORG/ai-workflow-skills.git
cd ai-workflow-skills
cargo build --release --locked
# ./target/release/ai-workflow-skills
```

### Public fork only: pipe install script

If you maintain a **public** fork, you may still use:

```bash
export GITHUB_REPOSITORY="YOUR_ORG/ai-workflow-skills"
curl -fsSL "https://raw.githubusercontent.com/${GITHUB_REPOSITORY}/main/install.sh" | bash -s --
```

For the private upstream, always use Options A–C.

---

## Configure the catalog

Every command that reads skills needs to know **where the catalog is**: a **local directory** or a **Git URL**. Remote catalogs are cloned or updated under `~/.cache/ai-workflow-skills/catalog-git/<hash>/`.

| Source | Environment variable | Notes |
|--------|----------------------|--------|
| Single catalog | `AI_WORKFLOW_SKILLS_CATALOG` | Same as `--catalog`. |
| Multiple catalogs | `AI_WORKFLOW_SKILLS_CATALOGS` | Entries separated by `|||` or newlines. Used when `--catalog` is not passed. |

You can also pass **`--catalog`** one or more times on the CLI; that overrides the env-based resolution for that invocation.

### This repository’s catalog (local, private-friendly)

Skills ship under [`catalog/`](catalog/) (tooling + project specs from `.specs`). No Git token required if you use a filesystem path:

```bash
export AI_WORKFLOW_SKILLS_CATALOG="/path/to/ai-workflow-skills/catalog"
```

Add that line to `~/.zshrc` or `~/.bashrc` after clone.

### Remote catalogs (including private Git)

For a **private** skills repo over HTTPS, set `AI_WORKFLOW_SKILLS_GIT_TOKEN`; the CLI injects `https://oauth2:<token>@…` into the remote URL.

```bash
export AI_WORKFLOW_SKILLS_CATALOG="https://github.com/YOUR_ORG/your-skill-catalog.git"
export AI_WORKFLOW_SKILLS_GIT_TOKEN="ghp_…"
```

For `git@github.com:ORG/repo.git` catalog URLs, use SSH keys (no token env var).

**Note:** Cloning [tech-leads-club/agent-skills](https://github.com/tech-leads-club/agent-skills) requires the catalog root `packages/skills-catalog` inside the clone (not the repo root). This repo’s layout is already `catalog/skills/…`.

---

## How to use

Check the binary:

```bash
ai-workflow-skills --version
ai-workflow-skills --help
```

### Interactive UI (TUI)

When **stdin and stdout are a TTY** and you run **no subcommand**, the tool opens the interactive manager:

```bash
export AI_WORKFLOW_SKILLS_CATALOG="/path/to/ai-workflow-skills/catalog"
ai-workflow-skills
```

| Key | Action |
|-----|--------|
| **↑** / **↓** | Move in list |
| **p** | Install to project |
| **g** | Install globally (`~`) |
| **u** | Update |
| **r** | Remove |
| **c** | Clear action on row |
| **Enter** | Apply |
| **q** / **Esc** | Quit |

Non-interactive environments (CI, pipes) print help and exit; use the subcommands below instead.

### CLI subcommands

| Command | Purpose |
|---------|---------|
| `list` | List skills from the catalog, or use `--installed` for the lockfile. |
| `install` | Install one or more skills (`-s` / `--skill`, repeatable). |
| `update` | Refresh installed skills when the catalog changed. |
| `remove` / `rm` | Remove installed skills. |
| `cache` | Inspect or clear cache under `~/.cache/ai-workflow-skills`. |
| `audit` | Show recent audit log entries (`~/.ai-workflow-skills/audit.log`). |
| `generate-registry` | Emit `skills-registry.json` from the catalog. |

Common flags:

- **`--catalog PATH_OR_URL`**: repeat to merge several catalogs (order matters if names collide).
- **`list --installed`**: show what this project has recorded in `.agents/.skill-lock.json`.
- **`list --tag TAG`**: filter catalog skills by tag (kebab-case).
- **`install`**: `-s my-skill` (required in non-interactive mode); `-g` / `--global` for user-wide install; `-a` / `--agent` for targets (e.g. `cursor`, `claude-code`); `--symlink` vs copy; `-f` / `--force` to reinstall.
- **`remove`**: `-s skill1 -s skill2`; optional `-a` / `--agent`, `-g` for global removal semantics.

If the same skill name appears in more than one catalog, specify **`catalog-label/skill-name`** (as shown by `list`).

### Example session

From a clone of this private repo:

1. Point at the local catalog (or pass `--catalog` on each command).

```bash
cd ai-workflow-skills
export AI_WORKFLOW_SKILLS_CATALOG="$(pwd)/catalog"
```

2. List available skills.

```bash
ai-workflow-skills list
# e.g. tool-github-cli, tool-databricks, proj-specs-testing, …
```

3. Install skills into the current project (from the repo root).

```bash
ai-workflow-skills install -s tool-github-cli -s tool-databricks -a cursor
# project specs (split by topic):
ai-workflow-skills install \
  -s proj-specs-architecture -s proj-specs-conventions \
  -s proj-specs-testing -s proj-specs-debug \
  -s proj-specs-changes -s proj-specs-audits \
  -a cursor
```

4. List what is installed according to the lockfile.

```bash
ai-workflow-skills list --installed
```

5. Update after the catalog changes.

```bash
ai-workflow-skills update --skill "my-skill-name"
```

6. Remove a skill.

```bash
ai-workflow-skills remove --skill "my-skill-name"
```

---

## Notes

- The CLI supports local directories and Git-backed catalogs.
- Installed skills are tracked in `.agents/.skill-lock.json`.
- Use `AI_WORKFLOW_SKILLS_CATALOGS` to combine multiple catalogs with `|||` or newline separators.

## License

MIT OR Apache-2.0
