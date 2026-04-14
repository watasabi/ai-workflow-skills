use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use crate::agents::{default_agents_for_install, detect_installed_agents, validate_agents};
use crate::catalog::{
    discover_skills, generate_registry_merged, normalize_tag_filter, resolve_skills_root,
};
use crate::constants::{ENV_CATALOG, ENV_CATALOGS};
use crate::core::audit::{audit_log_path, read_audit_log};
use crate::core::cache::{cache_dir, clear_all_cache, clear_registry_cache, registry_cache_path};
use crate::core::installer::{install_skills, remove_skills};
use crate::core::lockfile;
use crate::project_root::find_project_root;
use crate::sanitize::{sanitize_name, to_slug};
use crate::types::{InstallMethod, InstallOptions, InstallResult, RemoveOptions, SkillInfo};

#[derive(Parser)]
#[command(name = "ai-workflow-skills")]
#[command(
    about = "Instala e gerencia skills para agentes de IA a partir de catálogo (pasta local ou URL Git com cache em ~/.cache/.../catalog-git/). Sem subcomando e em terminal TTY, abre o modo interativo."
)]
#[command(version)]
pub struct Cli {
    /// Sem subcomando: modo gerenciamento interativo (requer TTY); noutros casos mostra a ajuda.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Lista skills disponíveis no catálogo (ou instaladas com --installed)
    List {
        /// Diretório local ou URL Git; repita `--catalog` para unir vários catálogos (ordem importa se o nome da skill repetir).
        #[arg(long, value_name = "PATH_OR_URL", action = clap::ArgAction::Append)]
        catalog: Vec<String>,
        #[arg(long)]
        installed: bool,
        /// Filtra skills do catálogo que incluem esta tag (kebab-case; ex.: devops)
        #[arg(long)]
        tag: Option<String>,
    },
    /// Instala skills nos agentes alvo (padrão Cursor: `.cursor/skills/`; ver `cursor.com/docs/skills`)
    Install {
        /// Sem valores: lista unificada em TTY (teclas p/g/u/h/r + Enter); fora de TTY use -s/--skill.
        #[arg(short = 's', long = "skill")]
        skill: Vec<String>,
        /// Agentes destino (ex.: claude-code, windsurf). Sem esta flag: detecção ou só **cursor**.
        #[arg(short = 'a', long = "agent", num_args = 1..)]
        agent: Option<Vec<String>>,
        #[arg(short = 'g', long)]
        global: bool,
        #[arg(long)]
        symlink: bool,
        /// Reinstala com `force_update` e regista `forced` no audit log.
        #[arg(short = 'f', long)]
        force: bool,
        /// Catálogo(s); repita `--catalog` para unir vários (ver `list`).
        #[arg(long, value_name = "PATH_OR_URL", action = clap::ArgAction::Append)]
        catalog: Vec<String>,
    },
    /// Atualiza skills instaladas quando o hash do catálogo mudou
    Update {
        #[arg(short = 's', long)]
        skill: Option<String>,
        #[arg(long, value_name = "PATH_OR_URL", action = clap::ArgAction::Append)]
        catalog: Vec<String>,
        #[arg(short = 'g', long)]
        global: bool,
        #[arg(long)]
        symlink: bool,
    },
    /// Remove skills instaladas
    #[command(alias = "rm")]
    Remove {
        #[arg(short = 's', long = "skill", num_args = 1..)]
        skill: Vec<String>,
        #[arg(short = 'a', long = "agent", num_args = 1..)]
        agent: Option<Vec<String>>,
        #[arg(short = 'g', long)]
        global: Option<bool>,
        #[arg(short = 'f', long)]
        force: bool,
        #[arg(long, value_name = "PATH_OR_URL", action = clap::ArgAction::Append)]
        catalog: Vec<String>,
    },
    /// Gerencia cache local (~/.cache/ai-workflow-skills)
    Cache {
        #[arg(long)]
        clear: bool,
        #[arg(long)]
        clear_registry: bool,
        #[arg(long)]
        path: bool,
    },
    /// Exibe trilha de auditoria
    Audit {
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        path: bool,
    },
    /// Gera skills-registry.json a partir do catálogo
    GenerateRegistry {
        #[arg(long, value_name = "PATH_OR_URL", action = clap::ArgAction::Append)]
        catalog: Vec<String>,
        #[arg(short = 'o', long, default_value = "skills-registry.json")]
        output: PathBuf,
    },
}

fn parse_catalog_list_env(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for segment in raw.split("|||") {
        for line in segment.lines() {
            let t = line.trim();
            if !t.is_empty() {
                out.push(t.to_string());
            }
        }
    }
    out
}

/// Resolve entradas de catálogo: `--catalog` (repetível), depois `ENV_CATALOGS`, depois `ENV_CATALOG`.
pub(crate) fn gather_catalog_inputs(cli: &[String]) -> Result<Vec<String>> {
    if !cli.is_empty() {
        return Ok(cli.to_vec());
    }
    if let Ok(s) = std::env::var(ENV_CATALOGS) {
        let v = parse_catalog_list_env(&s);
        if !v.is_empty() {
            return Ok(v);
        }
    }
    if let Ok(s) = std::env::var(ENV_CATALOG) {
        return Ok(vec![s]);
    }
    Err(anyhow!(
        "informe --catalog (uma ou mais vezes) ou a variável de ambiente {ENV_CATALOG} ou {ENV_CATALOGS}"
    ))
}

/// Catálogo resolvido: caminho local, rótulo humano e (se aplicável) URL remota original.
#[derive(Debug, Clone)]
pub struct ResolvedCatalog {
    pub path: PathBuf,
    pub label: String,
    pub remote: Option<String>,
}

/// Resolve cada entrada (local ou URL Git com cache em `~/.cache/.../catalog-git/`).
pub(crate) fn resolve_all_catalogs(home: &Path, cli: &[String]) -> Result<Vec<ResolvedCatalog>> {
    let inputs = gather_catalog_inputs(cli)?;
    let mut out = Vec::with_capacity(inputs.len());
    for raw in inputs {
        let (path, remote) = crate::catalog::remote::resolve_remote_or_local(home, &raw)?;
        let label = catalog_label_for(&raw, &path, remote.as_deref());
        out.push(ResolvedCatalog {
            path,
            label,
            remote,
        });
    }
    Ok(out)
}

/// Deriva um rótulo humano:
/// - remoto: último segmento da URL sem `.git` (fallback: nome da pasta de cache).
/// - local: `file_name()` do caminho (fallback: entrada textual).
fn catalog_label_for(raw_input: &str, resolved_path: &Path, remote: Option<&str>) -> String {
    if let Some(url) = remote {
        if let Some(label) = remote_label_from_url(url) {
            return label;
        }
    }
    if let Some(fname) = resolved_path.file_name().and_then(|n| n.to_str()) {
        if !fname.is_empty() {
            return fname.to_string();
        }
    }
    raw_input.to_string()
}

fn remote_label_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let last = trimmed
        .rsplit(['/', ':'])
        .find(|s| !s.is_empty())?
        .trim_end_matches(".git");
    if last.is_empty() {
        None
    } else {
        Some(last.to_string())
    }
}

/// Carrega skills de várias raízes de catálogo (todas as entradas; nomes podem repetir-se entre catálogos).
pub(crate) fn load_merged_catalog_skills(catalogs: &[ResolvedCatalog]) -> Result<Vec<SkillInfo>> {
    let mut merged = Vec::new();
    for c in catalogs {
        merged.extend(load_catalog_skills(&c.path, &c.label)?);
    }
    Ok(merged)
}

pub(crate) fn load_catalog_skills(catalog: &Path, label: &str) -> Result<Vec<SkillInfo>> {
    let root = resolve_skills_root(catalog)
        .with_context(|| format!("catálogo inválido: {}", catalog.display()))?;
    let mut skills = discover_skills(&root)
        .with_context(|| format!("catálogo inválido: {}", catalog.display()))?;
    for s in &mut skills {
        s.catalog_label = label.to_string();
    }
    Ok(skills)
}

/// Resolve uma skill do catálogo unificado.
///
/// Sintaxe suportada:
/// - `skill` — match por nome. Se ambíguo (>1 catálogo expõe o mesmo nome), retorna erro
///   listando candidatos como `catalogo/skill`.
/// - `catalogo/skill` — match exato por `(catalog_label, name)`.
pub(crate) fn find_skill<'a>(catalog: &'a [SkillInfo], name: &str) -> Result<&'a SkillInfo> {
    let want = name.trim();
    if let Some((prefix, suffix)) = want.split_once('/') {
        let prefix = prefix.trim();
        let suffix = suffix.trim();
        let candidates: Vec<&SkillInfo> = catalog
            .iter()
            .filter(|s| {
                s.catalog_label == prefix
                    && (s.name == suffix
                        || to_slug(&s.name) == suffix
                        || sanitize_name(&s.name) == sanitize_name(suffix))
            })
            .collect();
        return match candidates.len() {
            0 => Err(anyhow!(
                "skill '{suffix}' não encontrada no catálogo '{prefix}' — verifique o rótulo com `list`"
            )),
            1 => Ok(candidates[0]),
            _ => Err(anyhow!(
                "skill '{prefix}/{suffix}' aparece {} vezes no mesmo catálogo — catálogo duplicado?",
                candidates.len()
            )),
        };
    }

    let matches: Vec<&SkillInfo> = catalog
        .iter()
        .filter(|s| {
            s.name == want
                || to_slug(&s.name) == want
                || sanitize_name(&s.name) == sanitize_name(want)
        })
        .collect();
    match matches.len() {
        0 => Err(anyhow!("Skill não encontrada no catálogo: {want}")),
        1 => Ok(matches[0]),
        _ => {
            let candidates: Vec<String> = matches
                .iter()
                .map(|s| format!("{}/{}", s.catalog_label, s.name))
                .collect();
            Err(anyhow!(
                "skill '{want}' é ambígua entre catálogos. Use a sintaxe `catálogo/skill`. Candidatos: {}",
                candidates.join(", ")
            ))
        }
    }
}

pub(crate) fn install_with_progress_bar(
    project_root: &Path,
    home: &Path,
    selected: &[SkillInfo],
    opts: &InstallOptions,
) -> Result<Vec<InstallResult>, std::io::Error> {
    install_skills(project_root, home, selected, opts, || ())
}

pub fn run(cli: Cli) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("HOME não definido"))?;
    let cwd = std::env::current_dir()?;

    match cli.command {
        None => {
            if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                let resolved = resolve_all_catalogs(&home, &[])?;
                crate::ui::run(&home, &cwd, &resolved)
            } else {
                let mut cmd = Cli::command();
                cmd.print_help().map_err(|e| anyhow!("{e}"))?;
                std::process::exit(2);
            }
        }
        Some(cmd) => dispatch_command(cmd, &home, &cwd),
    }
}

pub(crate) fn dispatch_command(
    cmd: Commands,
    home: &std::path::Path,
    cwd: &std::path::Path,
) -> Result<()> {
    match cmd {
        Commands::List {
            catalog,
            installed,
            tag,
        } => {
            let project_root = find_project_root(Some(cwd));
            if installed {
                let lock = lockfile::read_skill_lock(&project_root, home, false);
                if lock.skills.is_empty() {
                    println!("Nenhuma skill no lockfile local (.agents/.skill-lock.json).");
                    return Ok(());
                }
                println!("Skills registradas no lockfile:");
                for (name, e) in lock.skills.iter() {
                    let ag = e.agents.clone().unwrap_or_default().join(", ");
                    let ver = e.version.as_deref().unwrap_or("?");
                    let cat = e.catalog_label.as_deref().unwrap_or("—");
                    println!(
                        "  • {name}  v{ver}  (catálogo: {cat})  [agentes: {ag}]  method={:?}",
                        e.method
                    );
                }
                return Ok(());
            }
            let resolved = resolve_all_catalogs(home, catalog.as_slice())?;
            let mut skills = load_merged_catalog_skills(&resolved)?;
            let filter_note = if let Some(ref t) = tag {
                let want = normalize_tag_filter(t)?;
                skills.retain(|s| s.tags.contains(&want));
                Some(want)
            } else {
                None
            };
            println!("Catálogos ({}):", resolved.len());
            for c in &resolved {
                if let Some(url) = &c.remote {
                    println!("  • {} (remoto: {url})", c.label);
                    println!("    cache: {}", c.path.display());
                } else {
                    println!("  • {} (local: {})", c.label, c.path.display());
                }
            }
            println!();
            if let Some(ref w) = filter_note {
                println!("(filtrado por tag: {w})\n");
            }
            for s in &skills {
                let cat_label = s.category.as_deref().unwrap_or("?");
                if s.tags.is_empty() {
                    println!(
                        "  • {}  (catálogo: {})  [{}] v{}\n    {}",
                        s.name, s.catalog_label, cat_label, s.catalog_version, s.description
                    );
                } else {
                    println!(
                        "  • {}  (catálogo: {})  [{}] v{}\n    tags: {}\n    {}",
                        s.name,
                        s.catalog_label,
                        cat_label,
                        s.catalog_version,
                        s.tags.join(", "),
                        s.description
                    );
                }
            }
            println!("\nTotal: {} skill(s)", skills.len());
            Ok(())
        }
        Commands::Install {
            skill,
            agent,
            global,
            symlink,
            force,
            catalog,
        } => {
            let resolved = resolve_all_catalogs(home, catalog.as_slice())?;
            let catalog_skills = load_merged_catalog_skills(&resolved)?;
            let project_root = find_project_root(Some(cwd));

            let mut selected: Vec<SkillInfo> = Vec::new();
            if skill.is_empty() {
                if !std::io::stdout().is_terminal() {
                    return Err(anyhow!(
                        "informe ao menos uma skill com -s/--skill (modo não interativo)"
                    ));
                }
                crate::ui::run(home, cwd, &resolved)?;
                return Ok(());
            } else {
                for name in &skill {
                    match find_skill(&catalog_skills, name) {
                        Ok(s) => selected.push(s.clone()),
                        Err(e) => eprintln!("{e}"),
                    }
                }
                if selected.is_empty() {
                    return Err(anyhow!("nenhuma skill válida para instalar"));
                }
            }

            let agents = if let Some(a) = agent {
                validate_agents(&a)?
            } else if skill.is_empty() {
                default_agents_for_install()
            } else {
                let detected = detect_installed_agents(home, &project_root);
                if detected.is_empty() {
                    default_agents_for_install()
                } else {
                    detected
                }
            };
            let method = if symlink {
                InstallMethod::Symlink
            } else {
                InstallMethod::Copy
            };
            let opts = InstallOptions {
                global,
                method,
                agents: agents.clone(),
                skills: selected.iter().map(|s| s.name.clone()).collect(),
                force_update: force,
                audit_forced: force,
            };
            let results = install_with_progress_bar(&project_root, home, &selected, &opts)?;
            let ok = results.iter().filter(|r| r.success).count();
            let bad = results.len() - ok;
            println!("Concluído: {ok} ok, {bad} falha(s).");
            if bad > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        Commands::Update {
            skill,
            catalog,
            global,
            symlink,
        } => {
            let resolved = resolve_all_catalogs(home, catalog.as_slice())?;
            let project_root = find_project_root(Some(cwd));

            // Load catalog skills for update
            let catalog_skills = load_merged_catalog_skills(&resolved)?;

            // Determine which skills to update
            let skills_to_update: Vec<SkillInfo> = if let Some(skill_name) = skill {
                match find_skill(&catalog_skills, &skill_name) {
                    Ok(s) => vec![s.clone()],
                    Err(e) => {
                        eprintln!("{e}");
                        return Ok(());
                    }
                }
            } else {
                // Update all installed skills. For each lock entry, prefer the row in the
                // matching catalog_label; fall back to name-only match for legacy entries.
                let lock = lockfile::read_skill_lock(&project_root, home, false);
                let mut to_update = Vec::new();
                for (skill_name, entry) in lock.skills.iter() {
                    let by_provenance = entry.catalog_label.as_deref().and_then(|label| {
                        catalog_skills
                            .iter()
                            .find(|s| s.catalog_label == label && s.name == *skill_name)
                    });
                    if let Some(s) = by_provenance {
                        to_update.push(s.clone());
                    } else if let Ok(s) = find_skill(&catalog_skills, skill_name) {
                        to_update.push(s.clone());
                    }
                }
                to_update
            };

            if skills_to_update.is_empty() {
                println!("Nenhuma skill para atualizar.");
                return Ok(());
            }

            let method = if symlink {
                InstallMethod::Symlink
            } else {
                InstallMethod::Copy
            };

            let opts = InstallOptions {
                global,
                method,
                agents: vec!["cursor".to_string()], // Default for update
                skills: skills_to_update.iter().map(|s| s.name.clone()).collect(),
                force_update: true,
                audit_forced: false,
            };

            let results = install_with_progress_bar(&project_root, home, &skills_to_update, &opts)?;
            let ok = results.iter().filter(|r| r.success).count();
            let bad = results.len() - ok;
            println!("Concluído: {ok} atualizadas, {bad} falha(s).");
            if bad > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        Commands::Remove {
            skill,
            agent,
            global,
            force,
            catalog: _,
        } => {
            let project_root = find_project_root(Some(cwd));
            let agents = if let Some(a) = agent {
                validate_agents(&a)?
            } else {
                let lock = lockfile::read_skill_lock(&project_root, home, false);
                let mut set = std::collections::HashSet::new();
                for name in &skill {
                    if let Some(e) = lock.skills.get(name) {
                        for ag in e.agents.clone().unwrap_or_default() {
                            set.insert(ag);
                        }
                    }
                }
                if set.is_empty() {
                    default_agents_for_install()
                } else {
                    set.into_iter().collect()
                }
            };
            let opts = RemoveOptions { global, force };
            let mut remove_failed = false;
            for name in &skill {
                let results = remove_skills(&project_root, home, name, &agents, &opts);
                if results.iter().any(|r| !r.success) {
                    remove_failed = true;
                }
            }
            if remove_failed {
                std::process::exit(1);
            }
            Ok(())
        }
        Commands::Cache {
            clear,
            clear_registry,
            path,
        } => {
            if clear {
                clear_all_cache(home)?;
                println!("Cache limpo.");
            } else if clear_registry {
                clear_registry_cache(home)?;
                println!("Cache do registry limpo.");
            } else if path {
                println!("{}", cache_dir(home).display());
            } else {
                println!("Diretório de cache: {}", cache_dir(home).display());
                println!("Registry em cache: {}", registry_cache_path(home).display());
                println!(
                    "Catálogo remoto (URL Git): clones em {}/*/ (subpastas por URL)",
                    cache_dir(home).join("catalog-git").display()
                );
            }
            Ok(())
        }
        Commands::Audit { limit, path } => {
            if path {
                println!("{}", audit_log_path(home).display());
            } else {
                let entries = read_audit_log(home, Some(limit));
                for e in entries {
                    println!("{}", serde_json::to_string_pretty(&e)?);
                }
            }
            Ok(())
        }
        Commands::GenerateRegistry { catalog, output } => {
            let resolved = resolve_all_catalogs(home, catalog.as_slice())?;
            let paths: Vec<PathBuf> = resolved.iter().map(|c| c.path.clone()).collect();
            let reg = generate_registry_merged(&paths)?;
            let json = serde_json::to_string_pretty(&reg)?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output, json)?;
            println!(
                "Escrito: {} ({} skills)",
                output.display(),
                reg.skills.len()
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_minimal_catalog(root: &Path) {
        let skills = root.join("skills").join("(demo)").join("demo-skill");
        fs::create_dir_all(&skills).unwrap();
        fs::write(
            skills.join("SKILL.md"),
            "---\nname: demo-skill\ndescription: d\n---\n# demo\n",
        )
        .unwrap();
        fs::write(
            skills.join("skill.manifest.json"),
            r#"{"version": "1.0.0"}"#,
        )
        .unwrap();
    }

    #[test]
    fn load_catalog_skills_sets_catalog_label() {
        let tmp = tempdir().unwrap();
        write_minimal_catalog(tmp.path());
        let skills = load_catalog_skills(tmp.path(), "my-label").unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].catalog_label, "my-label");
        assert_eq!(skills[0].name, "demo-skill");
    }

    #[test]
    fn remote_label_from_url_strips_dot_git() {
        assert_eq!(
            remote_label_from_url("https://github.com/g/my-catalog.git").as_deref(),
            Some("my-catalog")
        );
        assert_eq!(
            remote_label_from_url("git@github.com:g/other.git").as_deref(),
            Some("other")
        );
        assert_eq!(
            remote_label_from_url("https://example.com/repo/").as_deref(),
            Some("repo")
        );
    }

    #[test]
    fn find_skill_ambiguous_errors_with_candidates() {
        let skills = vec![
            SkillInfo {
                name: "dup".into(),
                catalog_label: "cat-a".into(),
                description: "d".into(),
                catalog_version: "1.0.0".into(),
                tags: vec![],
                path: PathBuf::from("/tmp/a/dup"),
                category: None,
            },
            SkillInfo {
                name: "dup".into(),
                catalog_label: "cat-b".into(),
                description: "d".into(),
                catalog_version: "1.0.0".into(),
                tags: vec![],
                path: PathBuf::from("/tmp/b/dup"),
                category: None,
            },
        ];
        let err = find_skill(&skills, "dup").unwrap_err().to_string();
        assert!(err.contains("ambígua"), "{err}");
        assert!(err.contains("cat-a/dup"), "{err}");
        assert!(err.contains("cat-b/dup"), "{err}");
    }

    #[test]
    fn find_skill_with_catalog_prefix_resolves() {
        let skills = vec![
            SkillInfo {
                name: "dup".into(),
                catalog_label: "cat-a".into(),
                description: "d".into(),
                catalog_version: "1.0.0".into(),
                tags: vec![],
                path: PathBuf::from("/tmp/a/dup"),
                category: None,
            },
            SkillInfo {
                name: "dup".into(),
                catalog_label: "cat-b".into(),
                description: "d".into(),
                catalog_version: "1.0.0".into(),
                tags: vec![],
                path: PathBuf::from("/tmp/b/dup"),
                category: None,
            },
        ];
        let found = find_skill(&skills, "cat-b/dup").unwrap();
        assert_eq!(found.catalog_label, "cat-b");
    }

    #[test]
    fn find_skill_with_catalog_prefix_errors_on_duplicates() {
        let skills = vec![
            SkillInfo {
                name: "dup".into(),
                catalog_label: "cat-a".into(),
                description: "d".into(),
                catalog_version: "1.0.0".into(),
                tags: vec![],
                path: PathBuf::from("/tmp/a1/dup"),
                category: None,
            },
            SkillInfo {
                name: "dup".into(),
                catalog_label: "cat-a".into(),
                description: "d".into(),
                catalog_version: "1.0.0".into(),
                tags: vec![],
                path: PathBuf::from("/tmp/a2/dup"),
                category: None,
            },
        ];
        let err = find_skill(&skills, "cat-a/dup").unwrap_err().to_string();
        assert!(err.contains("cat-a/dup"), "{err}");
        assert!(err.contains("duplicado"), "{err}");
    }
}
