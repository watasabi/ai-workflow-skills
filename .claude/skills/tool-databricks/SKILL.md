---
name: tool-databricks
description: Databricks CLI and aitools for auth, profiles, data exploration, jobs, and bundles. Use when the user mentions Databricks CLI, workspaces, Unity Catalog, SQL warehouses, or databricks commands. Requires databricks CLI installed and authenticated. Do NOT use for generic Spark/Python without Databricks context.
---

# Databricks CLI for agents

Condensed from [databricks/databricks-agent-skills `databricks-core`](https://github.com/databricks/databricks-agent-skills/tree/main/skills/databricks-core). Detailed guides are in `references/`.

## Prerequisites

1. **CLI installed**: `databricks -v` (or `databricks --version`). Require **>= v0.292.0**.
   - If missing or outdated: read [references/databricks-cli-install.md](references/databricks-cli-install.md). In sandboxed IDEs, present install commands for the user to run in their own terminal.
2. **Authenticated**: `databricks auth profiles`
   - If none or user needs a new workspace: read [references/databricks-cli-auth.md](references/databricks-cli-auth.md).

## Profile selection — critical

**Never auto-select a profile.**

1. Run `databricks auth profiles`
2. Present **all** profiles with workspace URLs and validity
3. Let the user choose (even if only one exists)
4. Use `--profile <name>` on every command

## Agent shells (Cursor, etc.)

Each command may run in a **separate shell**. Environment variables do not persist.

```bash
# WORKS
databricks apps list --profile my-workspace

# WORKS
export DATABRICKS_CONFIG_PROFILE=my-workspace && databricks apps list

# DOES NOT WORK across separate commands
export DATABRICKS_CONFIG_PROFILE=my-workspace
databricks apps list   # profile not set
```

## Data exploration — prefer aitools

Use these instead of manually walking catalogs/schemas/tables:

```bash
databricks experimental aitools tools discover-schema catalog.schema.table --profile <PROFILE>
databricks experimental aitools tools query "SELECT * FROM t LIMIT 10" --profile <PROFILE>
databricks experimental aitools tools get-default-warehouse --profile <PROFILE>
```

See [references/data-exploration.md](references/data-exploration.md).

## Quick reference

```bash
databricks current-user me --profile <PROFILE>
databricks apps list --profile <PROFILE>
databricks jobs list --profile <PROFILE>
databricks clusters list --profile <PROFILE>
databricks warehouses list --profile <PROFILE>
databricks pipelines list --profile <PROFILE>
databricks serving-endpoints list --profile <PROFILE>

# Unity Catalog — positional args (NOT --catalog flags)
databricks catalogs list --profile <PROFILE>
databricks schemas list <CATALOG> --profile <PROFILE>
databricks tables list <CATALOG> <SCHEMA> --profile <PROFILE>
databricks tables get <CATALOG>.<SCHEMA>.<TABLE> --profile <PROFILE>

# Bundles
databricks bundle init --profile <PROFILE>
databricks bundle validate --profile <PROFILE>
databricks bundle deploy -t <TARGET> --profile <PROFILE>
databricks bundle run <RESOURCE> -t <TARGET> --profile <PROFILE>
```

**Common mistakes**

- `databricks schemas list --catalog-name X` — wrong; use positional catalog name.
- `databricks sql-warehouses list` — use `warehouses list`.
- `databricks execute-statement` — use `experimental aitools tools query`.

When unsure: `databricks <command> --help`.

## Troubleshooting

| Error | Action |
|-------|--------|
| `cannot configure default credentials` | List profiles; retry with `--profile` or run OAuth login per references |
| `configuration does not support OAuth tokens` | Re-auth with `databricks auth login` for that profile |
| `PERMISSION_DENIED` | Check workspace / UC permissions |
| `RESOURCE_DOES_NOT_EXIST` | Verify resource name/id and profile |

## Required reading by task

| Task | Reference |
|------|-----------|
| Install / upgrade CLI | [databricks-cli-install.md](references/databricks-cli-install.md) |
| Auth / profiles | [databricks-cli-auth.md](references/databricks-cli-auth.md) |
| Schema discovery / SQL | [data-exploration.md](references/data-exploration.md) |

## Attribution

Reference content derived from [databricks/databricks-agent-skills](https://github.com/databricks/databricks-agent-skills). Follow upstream license terms for copied reference files.

## When not to use

- Generic Python/Spark work with no Databricks workspace or CLI context.
- Tasks better covered by project-specific bundle or app skills.
