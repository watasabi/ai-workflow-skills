use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::catalog::discover::{discover_skills, files_and_hash, resolve_skills_root};
use crate::constants::{CATEGORY_METADATA_FILE, DEFAULT_CATEGORY_ID};
use crate::sanitize::to_slug;
use crate::types::SkillInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub path: String,
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedEntry {
    pub name: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsRegistry {
    pub version: String,
    pub categories: HashMap<String, CategoryMeta>,
    pub skills: Vec<SkillMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Vec<DeprecatedEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMeta {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn load_category_metadata(skills_root: &Path) -> HashMap<String, CategoryMeta> {
    let p = skills_root.join(CATEGORY_METADATA_FILE);
    if !p.is_file() {
        return HashMap::new();
    }
    let Ok(raw) = fs::read_to_string(&p) else {
        return HashMap::new();
    };
    let Ok(json) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (folder, val) in json {
        let Some(cap) = crate::catalog::discover::CATEGORY_FOLDER.captures(&folder) else {
            continue;
        };
        let id = cap.get(1).unwrap().as_str().to_string();
        let name = val
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let mut c = id.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            });
        let description = val
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);
        out.insert(id.clone(), CategoryMeta { name, description });
    }
    out
}

fn load_deprecated(skills_root: &Path) -> Vec<DeprecatedEntry> {
    let p = skills_root.join("deprecated.yaml");
    if !p.is_file() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&p) else {
        return Vec::new();
    };
    serde_yaml::from_str::<Vec<DeprecatedEntry>>(&content).unwrap_or_default()
}

fn skill_relative_path(_skills_root: &Path, skill: &SkillInfo) -> String {
    let cat = skill.category.as_deref().unwrap_or(DEFAULT_CATEGORY_ID);
    let folder = skill
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if cat == DEFAULT_CATEGORY_ID {
        folder
    } else {
        format!("({cat})/{folder}")
    }
}

/// Gera registry a partir do diretório de catálogo (pai de `skills` ou próprio `skills`).
pub fn generate_registry(catalog_arg: &Path) -> anyhow::Result<SkillsRegistry> {
    let skills_root = resolve_skills_root(catalog_arg)
        .with_context(|| format!("catálogo inválido: {}", catalog_arg.display()))?;
    let mut categories = load_category_metadata(&skills_root);
    let deprecated = load_deprecated(&skills_root);
    let discovered = discover_skills(&skills_root)?;
    let mut skills: Vec<SkillMetadata> = Vec::new();

    for s in discovered {
        let rel = skill_relative_path(&skills_root, &s);
        let (files, content_hash) = files_and_hash(&s.path)?;
        let cat = s
            .category
            .clone()
            .unwrap_or_else(|| DEFAULT_CATEGORY_ID.to_string());
        let raw_name = s.name.clone();
        let skill_name = if raw_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && raw_name
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false)
        {
            raw_name.clone()
        } else {
            to_slug(&raw_name)
        };

        if !categories.contains_key(&cat) {
            categories.insert(
                cat.clone(),
                CategoryMeta {
                    name: cat
                        .chars()
                        .next()
                        .map(|c| c.to_uppercase().collect::<String>() + &cat[1..])
                        .unwrap_or_else(|| cat.clone()),
                    description: if cat == DEFAULT_CATEGORY_ID {
                        Some("Skills without a specific category".into())
                    } else {
                        None
                    },
                },
            );
        }

        let tags_opt = if s.tags.is_empty() {
            None
        } else {
            Some(s.tags.clone())
        };

        skills.push(SkillMetadata {
            name: skill_name,
            description: s.description,
            category: cat,
            path: rel,
            files,
            author: None,
            version: Some(s.catalog_version.clone()),
            tags: tags_opt,
            content_hash,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));

    let deprecated_opt = if deprecated.is_empty() {
        None
    } else {
        Some(deprecated)
    };

    Ok(SkillsRegistry {
        version: "1.0.0".into(),
        categories,
        skills,
        deprecated: deprecated_opt,
    })
}

fn merge_skills_registry(into: &mut SkillsRegistry, other: SkillsRegistry) {
    let SkillsRegistry {
        categories: oc,
        skills: os,
        deprecated: other_dep,
        version: _,
    } = other;
    for (k, v) in oc {
        into.categories.entry(k).or_insert(v);
    }
    let mut seen: HashSet<String> = into.skills.iter().map(|s| s.name.clone()).collect();
    for s in os {
        if seen.insert(s.name.clone()) {
            into.skills.push(s);
        } else {
            eprintln!(
                "aviso: entrada de registry para skill '{}' ignorada (duplicada)",
                s.name
            );
        }
    }
    into.skills.sort_by(|a, b| a.name.cmp(&b.name));

    match (into.deprecated.as_mut(), other_dep) {
        (Some(a), Some(b)) => {
            let mut dep_seen: HashSet<String> = a.iter().map(|d| d.name.clone()).collect();
            for d in b {
                if dep_seen.insert(d.name.clone()) {
                    a.push(d);
                }
            }
        }
        (None, Some(b)) if !b.is_empty() => {
            into.deprecated = Some(b);
        }
        _ => {}
    }
}

/// Gera um único `skills-registry.json` a partir de várias raízes de catálogo.
/// Ordem importa: o primeiro catálogo tem precedência em nomes de skill duplicados.
pub fn generate_registry_merged(catalog_roots: &[PathBuf]) -> anyhow::Result<SkillsRegistry> {
    if catalog_roots.is_empty() {
        anyhow::bail!("nenhum catálogo");
    }
    if catalog_roots.len() == 1 {
        return generate_registry(&catalog_roots[0]);
    }
    let mut acc = generate_registry(&catalog_roots[0])?;
    for path in &catalog_roots[1..] {
        let next = generate_registry(path)?;
        merge_skills_registry(&mut acc, next);
    }
    Ok(acc)
}
