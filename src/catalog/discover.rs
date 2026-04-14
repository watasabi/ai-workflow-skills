use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::constants::{DEFAULT_CATEGORY_ID, SKILL_MANIFEST_FILE};
use crate::sanitize::to_slug;
use crate::types::SkillInfo;

pub(crate) static CATEGORY_FOLDER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\(([a-z][a-z0-9-]*)\)$").expect("regex"));

/// Pastas `(example)/` são só para demonstração do layout no repositório; não entram em `list`, instalação nem `generate-registry`.
const EXCLUDED_CATALOG_CATEGORY_IDS: &[&str] = &["example"];
static FRONTMATTER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)^---\n([\s\S]*?)\n---").expect("regex"));
static NAME_LINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^name:\s*(.+)$").expect("regex"));
static DESC_LINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^description:\s*(.+)$").expect("regex"));

/// Ficheiros ignorados no `content_hash` (lixo de SO / placeholders). Tudo o resto sob a pasta
/// da skill entra no digest, **incluindo** `skill.manifest.json`.
const IGNORED: &[&str] = &[".DS_Store", ".gitkeep", "Thumbs.db", ".gitignore"];

static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("regex"));

#[derive(Debug, Deserialize)]
struct SkillManifest {
    version: String,
    #[serde(default)]
    tags: Vec<String>,
}

/// Conteúdo validado de `skill.manifest.json` (versão + tags opcionais).
#[derive(Debug, Clone)]
pub struct SkillManifestData {
    pub version: String,
    pub tags: Vec<String>,
}

fn normalize_tags(raw: Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    for t in raw {
        let s = t.trim().to_lowercase();
        if s.is_empty() {
            continue;
        }
        if s.len() > 48 {
            anyhow::bail!("tag muito longa (máx. 48): {s}");
        }
        if !TAG_RE.is_match(&s) {
            anyhow::bail!(
                "tag inválida `{s}` — use kebab-case ASCII (ex.: devops, api-rest, internal-only)"
            );
        }
        seen.insert(s);
    }
    Ok(seen.into_iter().collect())
}

/// Normaliza uma tag para uso em `--tag` / filtros (mesmas regras que no manifest).
pub fn normalize_tag_filter(input: &str) -> anyhow::Result<String> {
    let s = input.trim().to_lowercase();
    if s.is_empty() {
        anyhow::bail!("tag vazia");
    }
    if s.len() > 48 {
        anyhow::bail!("tag muito longa (máx. 48 caracteres)");
    }
    if !TAG_RE.is_match(&s) {
        anyhow::bail!("tag inválida — use kebab-case ASCII (ex.: devops, api-rest)");
    }
    Ok(s)
}

/// Lê e valida `skill.manifest.json` (SemVer + tags opcionais).
pub fn read_skill_manifest(path: &Path) -> anyhow::Result<SkillManifestData> {
    let raw = fs::read_to_string(path).with_context(|| format!("ler {}", path.display()))?;
    let m: SkillManifest = serde_json::from_str(&raw)
        .with_context(|| format!("JSON inválido em {}", path.display()))?;
    semver::Version::parse(m.version.trim())
        .map_err(|e| anyhow::anyhow!("version SemVer inválida em {}: {}", path.display(), e))?;
    let tags = normalize_tags(m.tags)?;
    Ok(SkillManifestData {
        version: m.version.trim().to_string(),
        tags,
    })
}

/// Lê só a versão SemVer (atalho sobre [`read_skill_manifest`]).
pub fn read_skill_manifest_version(path: &Path) -> anyhow::Result<String> {
    Ok(read_skill_manifest(path)?.version)
}

/// Resolve raiz do diretório `skills` a partir do catálogo informado.
pub fn resolve_skills_root(catalog_arg: &Path) -> std::io::Result<PathBuf> {
    let skills = catalog_arg.join("skills");
    if skills.is_dir() {
        return Ok(skills);
    }
    if catalog_arg.join("_category.json").exists() || category_subdirs_exist(catalog_arg)? {
        return Ok(catalog_arg.to_path_buf());
    }
    if catalog_arg.join("SKILL.md").exists() {
        return Ok(catalog_arg.to_path_buf());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "não foi possível localizar pasta de skills em: {}",
            catalog_arg.display()
        ),
    ))
}

fn category_subdirs_exist(dir: &Path) -> std::io::Result<bool> {
    if !dir.is_dir() {
        return Ok(false);
    }
    for e in fs::read_dir(dir)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().to_string();
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir && CATEGORY_FOLDER.is_match(&name) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>) {
    let fm = FRONTMATTER
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str());
    let Some(fm) = fm else {
        return (None, None);
    };
    let name = NAME_LINE
        .captures(fm)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let desc = DESC_LINE
        .captures(fm)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    (name, desc)
}

fn try_read_skill(skill_path: &Path, category_id: &str) -> anyhow::Result<Option<SkillInfo>> {
    let md = skill_path.join("SKILL.md");
    if !md.is_file() {
        return Ok(None);
    }
    let manifest_path = skill_path.join(SKILL_MANIFEST_FILE);
    if !manifest_path.is_file() {
        anyhow::bail!(
            "skill em {}: falta {} (versão obrigatória no manifest; o SKILL.md não declara versão)",
            skill_path.display(),
            SKILL_MANIFEST_FILE
        );
    }
    let manifest = read_skill_manifest(&manifest_path)?;
    let content = fs::read_to_string(&md).with_context(|| format!("ler {}", md.display()))?;
    let (name, desc) = parse_frontmatter(&content);
    if name.is_none() || desc.is_none() {
        eprintln!(
            "aviso: frontmatter não encontrado ou incompleto em {}",
            md.display()
        );
    }
    let folder = skill_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    Ok(Some(SkillInfo {
        name: name.unwrap_or(folder),
        catalog_label: String::new(),
        description: desc.unwrap_or_else(|| "No description".into()),
        catalog_version: manifest.version,
        tags: manifest.tags,
        path: skill_path.to_path_buf(),
        category: Some(category_id.to_string()),
    }))
}

fn scan_category(dir: &Path, category_id: &str) -> anyhow::Result<Vec<SkillInfo>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return Ok(out);
    };
    for e in rd.flatten() {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }
        if let Some(s) = try_read_skill(&e.path(), category_id)? {
            out.push(s);
        }
    }
    Ok(out)
}

/// Descobre skills no diretório raiz (layout agent-skills). Falha se faltar `skill.manifest.json` ou versão inválida.
pub fn discover_skills(skills_root: &Path) -> anyhow::Result<Vec<SkillInfo>> {
    let mut skills = Vec::new();
    if !skills_root.is_dir() {
        return Ok(skills);
    }
    let Ok(rd) = fs::read_dir(skills_root) else {
        return Ok(skills);
    };
    for e in rd.flatten() {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if let Some(cap) = CATEGORY_FOLDER.captures(&name) {
            let cat = cap.get(1).unwrap().as_str();
            if EXCLUDED_CATALOG_CATEGORY_IDS.contains(&cat) {
                continue;
            }
            skills.extend(scan_category(&e.path(), cat)?);
        } else if let Some(s) = try_read_skill(&e.path(), DEFAULT_CATEGORY_ID)? {
            skills.push(s);
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    let mut seen: std::collections::HashMap<String, std::path::PathBuf> =
        std::collections::HashMap::new();
    for s in &skills {
        let key = to_slug(&s.name);
        if let Some(prev) = seen.get(&key) {
            anyhow::bail!(
                "catálogo: nome de skill duplicado (slug `{}`) em {} e {}",
                key,
                prev.display(),
                s.path.display()
            );
        }
        seen.insert(key, s.path.clone());
    }
    Ok(skills)
}

fn list_skill_files(skill_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(skill_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.starts_with('.') || IGNORED.contains(&name.as_ref()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(skill_dir)
            .ok()
            .and_then(|p| p.to_str())
            .map(|s| s.trim_start_matches('/').to_string());
        if let Some(r) = rel {
            if !r.is_empty() {
                files.push(r);
            }
        }
    }
    files.sort();
    files
}

/// SHA-256 do conteúdo da skill: caminhos relativos ordenados + bytes por ficheiro (inclui `skill.manifest.json`).
pub fn compute_skill_hash(skill_dir: &Path, files: &[String]) -> anyhow::Result<String> {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.as_bytes());
        let p = skill_dir.join(file);
        if p.is_file() {
            let bytes =
                fs::read(&p).with_context(|| format!("ler {} para content_hash", p.display()))?;
            hasher.update(bytes);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Lista arquivos relativos e hash para um diretório de skill.
pub fn files_and_hash(skill_dir: &Path) -> anyhow::Result<(Vec<String>, String)> {
    let files = list_skill_files(skill_dir);
    let hash = compute_skill_hash(skill_dir, &files)?;
    Ok((files, hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_minimal_skill(dir: &std::path::Path, name: &str) {
        fs::create_dir_all(dir).unwrap();
        let mut f = fs::File::create(dir.join("SKILL.md")).unwrap();
        writeln!(f, "---\nname: {name}\ndescription: t\n---\n\n# {name}\n").unwrap();
        fs::write(dir.join(SKILL_MANIFEST_FILE), r#"{"version": "1.0.0"}"#).unwrap();
    }

    #[test]
    fn discover_skips_example_category() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_minimal_skill(&root.join("(example)/excluded"), "excluded");
        write_minimal_skill(&root.join("(demo)/kept"), "kept");
        let list = discover_skills(root).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "kept");
        assert_eq!(list[0].category.as_deref(), Some("demo"));
    }

    #[test]
    fn normalize_tag_trims_and_lowercases() {
        assert_eq!(normalize_tag_filter("  DevOps ").unwrap(), "devops");
    }

    #[test]
    fn normalize_tag_rejects_empty() {
        assert!(normalize_tag_filter("   ").is_err());
        assert!(normalize_tag_filter("").is_err());
    }

    #[test]
    fn normalize_tag_rejects_invalid_chars() {
        assert!(normalize_tag_filter("bad_tag").is_err());
        assert!(normalize_tag_filter("spa ces").is_err());
    }

    #[test]
    fn normalize_tag_rejects_too_long() {
        let s = "a".repeat(49);
        assert!(normalize_tag_filter(&s).is_err());
    }
}
