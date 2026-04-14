use serde::{Deserialize, Serialize};

/// Schema do lockfile (v3). v3 adiciona `catalog_label` para gravar a origem (provenance) de cada skill instalada.
pub const LOCKFILE_VERSION: i32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLockFile {
    pub version: i32,
    pub skills: std::collections::HashMap<String, SkillLockEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLockEntry {
    pub name: String,
    #[serde(default)]
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    pub installed_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Rótulo do catálogo que instalou esta skill (provenance). `None` em lockfiles legados (pré-v3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    /// Rótulo curto do catálogo de origem (pasta do catálogo passado ao carregar).
    pub catalog_label: String,
    pub description: String,
    /// Versão SemVer lida apenas de `skill.manifest.json` (nunca do `SKILL.md`).
    pub catalog_version: String,
    /// Etiquetas opcionais do manifest (kebab-case normalizado) para filtrar e agrupar skills.
    pub tags: Vec<String>,
    pub path: std::path::PathBuf,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub global: bool,
    pub method: InstallMethod,
    pub agents: Vec<String>,
    pub skills: Vec<String>,
    pub force_update: bool,
    /// `install --force` — registra `forced` na auditoria (updates usam `force_update` sem isto).
    pub audit_forced: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    Copy,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub agent: String,
    pub skill: String,
    pub path: std::path::PathBuf,
    pub method: InstallMethod,
    pub success: bool,
    pub error: Option<String>,
    pub used_global_symlink: bool,
    pub symlink_failed: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveOptions {
    pub global: Option<bool>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveResult {
    pub skill: String,
    pub agent: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub action: String,
    pub skill_name: String,
    pub agents: Vec<String>,
    pub success: i32,
    pub failed: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forced: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}
