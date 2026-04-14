# ai-workflow-skills

Rust CLI to install and manage **AI agent skills** from a versioned catalog (same idea as [agent-skills](https://github.com/tech-leads-club/agent-skills): folders with `SKILL.md`, project lockfile under `.agents/.skill-lock.json`).

Skills are installed for **Cursor** under `.cursor/skills/` (and other agents depending on your configuration).

## Build from source

```bash
cargo build --release
# binary: target/release/ai-workflow-skills
```

## Catalog path

Point the tool at a **local directory** or a **Git URL** (clone/pull into `~/.cache/ai-workflow-skills/catalog-git/<hash>/`):

| Source | Environment variable | Notes |
|--------|----------------------|--------|
| Single catalog | `AI_WORKFLOW_SKILLS_CATALOG` | Same as `--catalog` |
| Multiple catalogs | `AI_WORKFLOW_SKILLS_CATALOGS` | Entries separated by `\|\|\|` or newlines |

For **private HTTPS** Git remotes, set `AI_WORKFLOW_SKILLS_GIT_TOKEN`; the CLI injects `https://oauth2:<token>@…` into the URL (common pattern for private Git over HTTPS). For `git@…` remotes, use your SSH keys.

## Optional: install script

`install.sh` downloads release assets from **GitHub Releases** (public repos work without a token; private repos need `GITHUB_TOKEN` or `GH_TOKEN`). Configure your fork:

```bash
export GITHUB_REPOSITORY="watasabi/ai-workflow-skills"
curl -fsSL "https://raw.githubusercontent.com/watasabi/ai-workflow-skills/main/install.sh" | bash -s -- ...
```

The script can set `AI_WORKFLOW_SKILLS_CATALOG` to your skills catalog URL (`-c`).

## Interactive mode (TUI)

```bash
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

## License

MIT OR Apache-2.0
