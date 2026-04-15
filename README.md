# ai-workflow-skills

Rust CLI to install and manage **AI agent skills** from a versioned local or Git catalog. It follows the same idea as [agent-skills](https://github.com/tech-leads-club/agent-skills): skills are stored as folders with `SKILL.md`, and project installs are tracked in `.agents/.skill-lock.json`.

Skills are installed for **Cursor** under `.cursor/skills/` by default. Other agents are supported when you pass `--agent` or when the tool detects them in your project or home directory.

---

## Install

### Option A: Install script (recommended)

The script downloads a **Linux x86_64** release tarball from GitHub Releases, or on **macOS** clones the repo and runs `cargo build --release` (Rust required). It does **not** download a prebuilt binary for **Linux on ARM** (e.g. `aarch64`); on those machines use [Option B: Cargo](#option-b-cargo-from-this-repository) or [Option C: Build from source](#option-c-build-from-source) so the tool is compiled for your CPU.

**Requirements:** `curl` and `python3` (to resolve the latest release). On macOS you also need `git` and `cargo`.

```bash
export GITHUB_REPOSITORY="watasabi/ai-workflow-skills"   # use your fork if needed
export GITHUB_TOKEN="<your-token>"                     # optional: private repo or API rate limits
curl -fsSL "https://raw.githubusercontent.com/watasabi/ai-workflow-skills/main/install.sh" | bash -s --
```

| Flag | Meaning |
|------|---------|
| `-v TAG` | Install a specific release (e.g. `v0.3.0`). Default: latest GitHub release. |
| `-p DIR` | Install prefix; binaries go to `DIR/bin`. Default: `~/.local`. |
| `-t TOKEN` | Same as `GITHUB_TOKEN` / `GH_TOKEN` for the GitHub API or private clones. |
| `-c URL` | If set, appends `export AI_WORKFLOW_SKILLS_CATALOG="URL"` to your shell rc (and may add `PREFIX/bin` to `PATH`). |

**Custom install directory:**

```bash
PREFIX="${HOME}/.local"
curl -fsSL "https://raw.githubusercontent.com/watasabi/ai-workflow-skills/main/install.sh" | bash -s -- -p "${PREFIX}"
```

Ensure `${PREFIX}/bin` is on your `PATH` (the script can add it when `-c` is used).

### Option B: Cargo (from this repository)

Use this on **Linux ARM64** and other platforms where the install script is not applicable, or whenever you prefer Rust to manage the binary.

```bash
cargo install --path .              # from a clone
# or
cargo install --git https://github.com/watasabi/ai-workflow-skills.git --locked
```

The binary is `ai-workflow-skills` (e.g. under `~/.cargo/bin` if that directory is on your `PATH`).

### Option C: Build from source

```bash
git clone https://github.com/watasabi/ai-workflow-skills.git
cd ai-workflow-skills
cargo build --release --locked
# binary: target/release/ai-workflow-skills
```

---

## Configure the catalog

Every command that reads skills needs to know **where the catalog is**: a **local directory** or a **Git URL**. Remote catalogs are cloned or updated under `~/.cache/ai-workflow-skills/catalog-git/<hash>/`.

| Source | Environment variable | Notes |
|--------|----------------------|--------|
| Single catalog | `AI_WORKFLOW_SKILLS_CATALOG` | Same as `--catalog`. |
| Multiple catalogs | `AI_WORKFLOW_SKILLS_CATALOGS` | Entries separated by `|||` or newlines. Used when `--catalog` is not passed. |

You can also pass **`--catalog`** one or more times on the CLI; that overrides the env-based resolution for that invocation.

For **private HTTPS** Git remotes, set `AI_WORKFLOW_SKILLS_GIT_TOKEN`; the CLI injects `https://oauth2:<token>@…` into the remote URL. For `git@…` remotes, use SSH keys.

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
export AI_WORKFLOW_SKILLS_CATALOG="https://github.com/your-org/your-skill-catalog.git"
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

1. Point at a catalog (or use `--catalog` on each command).

```bash
export AI_WORKFLOW_SKILLS_CATALOG="https://github.com/your-org/your-skill-catalog.git"
```

2. List available skills.

```bash
ai-workflow-skills list
```

3. Install a skill into the current project (from the project root).

```bash
ai-workflow-skills install --skill "my-skill-name"
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
