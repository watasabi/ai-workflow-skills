# ai-workflow-skills — matriz de comandos (paridade principal com agent-skills)

Fonte do catálogo: **pasta local** ou **URL Git** (`--catalog` / `AI_WORKFLOW_SKILLS_CATALOG`). Com URL, o CLI sincroniza via `git` para `~/.cache/.../catalog-git/<hash>/`.

| Comando | Flags principais | Comportamento |
|--------|------------------|---------------|
| *(default)* | — | Mostra ajuda (sem TUI interativo; use subcomandos explícitos). |
| `list` / `ls` | `--catalog`, `--installed`, `--tag` | Lista skills do catálogo (com `tags:` por skill); `--tag` filtra por etiqueta (kebab-case). Com `--installed`, lista o lockfile (sem `--tag`). |
| `install` | `-s/--skill`, `-a/--agent`, `-g/--global`, `--symlink`, `-f/--force`, `--catalog` | Instala skills a partir do disco do catálogo. **Padrão Cursor** (`.cursor/skills/`) quando não há `-a` e não há detecção; caso contrário use `-a` (ex.: `claude-code`, `windsurf`). |
| `update` | `-s/--skill`, `--catalog` | Compara `content_hash` (e versão do manifest) do catálogo com o lockfile; reinstala quando o hash diverge. Avisa se o hash coincide mas a versão no lock difere do catálogo. |
| `remove` / `rm` | `-s/--skill`, `-a/--agent`, `-g/--global`, `-f/--force`, `--catalog` | Remove instalação e atualiza lockfile (com guardrails de path). |
| `cache` | `--clear`, `--clear-registry`, `--path` | Cache em `~/.cache/ai-workflow-skills/` (registry espelhado + metadados). |
| `audit` | `-n/--limit`, `--path` | Log em `~/.ai-workflow-skills/audit.log` (JSON lines, append-only). |
| `generate-registry` | `--catalog`, `-o/--output` | Gera `skills-registry.json` (mesmo modelo conceitual do agent-skills) para CI/review. |

Compatibilidade de artefatos com agent-skills: lockfile em `.agents/.skill-lock.json` (schema v2), skills canônicas em `.agents/skills/<name>` para symlink local.
