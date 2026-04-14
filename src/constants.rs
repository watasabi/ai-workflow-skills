//! Constantes de branding e caminhos (alinhados ao modelo agent-skills).

/// Diretório de estado do projeto para lockfile e skills canônicas.
pub const AGENTS_DIR: &str = ".agents";
/// Cópia local intermediária para symlink em instalação local.
pub const CANONICAL_SKILLS_DIR: &str = "skills";
pub const LOCK_FILE: &str = ".skill-lock.json";
pub const LOCK_FILE_BACKUP: &str = ".skill-lock.json.backup";

/// Config global do usuário para auditoria (`~/.ai-workflow-skills/`).
pub const GLOBAL_CONFIG_DIR: &str = ".ai-workflow-skills";
pub const AUDIT_LOG_FILE: &str = "audit.log";

pub const CACHE_BASE_DIR: &str = ".cache";
pub const CACHE_NAMESPACE: &str = "ai-workflow-skills";
pub const REGISTRY_CACHE_FILENAME: &str = "registry.json";

pub const DEFAULT_CATEGORY_ID: &str = "uncategorized";
pub const CATEGORY_METADATA_FILE: &str = "_category.json";
/// Metadados de versão da skill (SemVer); não duplicar versão no `SKILL.md`.
pub const SKILL_MANIFEST_FILE: &str = "skill.manifest.json";

pub const PACKAGE_DISPLAY_NAME: &str = "ai-workflow-skills";
/// Caminho local do catálogo **ou** URL Git (`https://…`, `git@…`); no segundo caso o CLI
/// mantém uma cópia em `~/.cache/.../catalog-git/`.
pub const ENV_CATALOG: &str = "AI_WORKFLOW_SKILLS_CATALOG";
/// Vários catálogos: entradas separadas por `|||` ou por linha (após trim). Usada só se
/// `--catalog` não for passado nenhuma vez e `AI_WORKFLOW_SKILLS_CATALOG` não estiver definida.
pub const ENV_CATALOGS: &str = "AI_WORKFLOW_SKILLS_CATALOGS";
