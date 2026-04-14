use std::fs;
use std::path::{Path, PathBuf};

use pathdiff::diff_paths;

use crate::agents::get_def;
use crate::catalog::files_and_hash;
use crate::constants::{AGENTS_DIR, CANONICAL_SKILLS_DIR as CANONICAL_SKILL_FOLDER};
use crate::core::audit::log_audit;
use crate::core::lockfile::{self, AddSkillOpts};
use crate::sanitize::{is_path_safe, sanitize_name};
use crate::types::AuditEntry;
use crate::types::{
    InstallMethod, InstallOptions, InstallResult, RemoveOptions, RemoveResult, SkillInfo,
};

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    if let Some(p) = dst.parent() {
        fs::create_dir_all(p)?;
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let s = entry.path();
        let d = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&s, &d)?;
        } else {
            fs::copy(&s, &d)?;
        }
    }
    Ok(())
}

fn symlink_skill(target: &Path, link: &Path) -> std::io::Result<()> {
    let parent = link
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no parent"))?;
    fs::create_dir_all(parent)?;
    let rel = diff_paths(target, parent)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "pathdiff failed"))?;
    if link.exists() || link.symlink_metadata().is_ok() {
        let _ = fs::remove_file(link);
        let _ = fs::remove_dir_all(link);
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&rel, link)
    }
    #[cfg(windows)]
    {
        if target.is_dir() {
            std::os::windows::fs::symlink_dir(&rel, link)
        } else {
            std::os::windows::fs::symlink_file(&rel, link)
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "symlink not supported",
        ))
    }
}

fn validate_install_path(
    target_dir: &Path,
    skill_target: &Path,
    project_root: &Path,
    global: bool,
) -> Option<String> {
    if global {
        return None;
    }
    if is_path_safe(target_dir, skill_target) {
        return None;
    }
    if is_path_safe(project_root, skill_target) {
        return None;
    }
    Some("Security: Invalid skill destination path".into())
}

fn install_one(
    skill: &SkillInfo,
    agent_id: &str,
    project_root: &Path,
    home: &Path,
    method: InstallMethod,
    global: bool,
) -> InstallResult {
    let Some(def) = get_def(agent_id) else {
        return InstallResult {
            agent: agent_id.to_string(),
            skill: skill.name.clone(),
            path: PathBuf::new(),
            method,
            success: false,
            error: Some("unknown agent".into()),
            used_global_symlink: false,
            symlink_failed: false,
        };
    };

    let target_dir = if global {
        (def.global_skills_dir)(home)
    } else {
        project_root.join(def.skills_dir)
    };

    let safe_name = sanitize_name(&skill.name);
    let skill_target = target_dir.join(&safe_name);

    if let Some(err) = validate_install_path(&target_dir, &skill_target, project_root, global) {
        return InstallResult {
            agent: def.display_name.to_string(),
            skill: skill.name.clone(),
            path: skill_target,
            method,
            success: false,
            error: Some(err),
            used_global_symlink: false,
            symlink_failed: false,
        };
    }

    let canonical_dir = project_root
        .join(AGENTS_DIR)
        .join(CANONICAL_SKILL_FOLDER)
        .join(&safe_name);

    match method {
        InstallMethod::Copy => {
            if let Err(e) = copy_dir_all(&skill.path, &skill_target) {
                return fail_result(def.display_name, &skill.name, skill_target, method, e);
            }
            ok_result(
                def.display_name,
                &skill.name,
                skill_target,
                method,
                false,
                false,
            )
        }
        InstallMethod::Symlink if global => {
            if symlink_skill(&skill.path, &skill_target).is_ok() {
                ok_result(
                    def.display_name,
                    &skill.name,
                    skill_target,
                    InstallMethod::Symlink,
                    false,
                    false,
                )
            } else if copy_dir_all(&skill.path, &skill_target).is_ok() {
                ok_result(
                    def.display_name,
                    &skill.name,
                    skill_target,
                    InstallMethod::Copy,
                    false,
                    true,
                )
            } else {
                fail_result(
                    def.display_name,
                    &skill.name,
                    skill_target,
                    method,
                    std::io::Error::other("symlink and copy failed"),
                )
            }
        }
        InstallMethod::Symlink => {
            if let Err(e) = copy_dir_all(&skill.path, &canonical_dir) {
                return fail_result(def.display_name, &skill.name, skill_target, method, e);
            }
            if symlink_skill(&canonical_dir, &skill_target).is_ok() {
                ok_result(
                    def.display_name,
                    &skill.name,
                    skill_target,
                    InstallMethod::Symlink,
                    false,
                    false,
                )
            } else if copy_dir_all(&skill.path, &skill_target).is_ok() {
                ok_result(
                    def.display_name,
                    &skill.name,
                    skill_target,
                    InstallMethod::Copy,
                    false,
                    true,
                )
            } else {
                fail_result(
                    def.display_name,
                    &skill.name,
                    skill_target,
                    method,
                    std::io::Error::other("symlink and copy failed"),
                )
            }
        }
    }
}

fn ok_result(
    agent: &str,
    skill: &str,
    path: PathBuf,
    method: InstallMethod,
    used_global_symlink: bool,
    symlink_failed: bool,
) -> InstallResult {
    InstallResult {
        agent: agent.to_string(),
        skill: skill.to_string(),
        path,
        method,
        success: true,
        error: None,
        used_global_symlink,
        symlink_failed,
    }
}

fn fail_result(
    agent: &str,
    skill: &str,
    path: PathBuf,
    method: InstallMethod,
    e: std::io::Error,
) -> InstallResult {
    InstallResult {
        agent: agent.to_string(),
        skill: skill.to_string(),
        path,
        method,
        success: false,
        error: Some(e.to_string()),
        used_global_symlink: false,
        symlink_failed: false,
    }
}

pub fn install_skills(
    project_root: &Path,
    home: &Path,
    skills: &[SkillInfo],
    options: &InstallOptions,
    mut on_step: impl FnMut(),
) -> Result<Vec<InstallResult>, std::io::Error> {
    let method = options.method;
    let mut results = Vec::new();

    for agent in &options.agents {
        for skill in skills {
            let r = install_one(skill, agent, project_root, home, method, options.global);
            if r.success {
                let hash = files_and_hash(&skill.path).ok().map(|(_, h)| h);
                let method_str = match r.method {
                    InstallMethod::Copy => "copy",
                    InstallMethod::Symlink => "symlink",
                };
                let catalog_label = if skill.catalog_label.is_empty() {
                    None
                } else {
                    Some(skill.catalog_label.clone())
                };
                lockfile::add_skill_to_lock(
                    project_root,
                    home,
                    &skill.name,
                    std::slice::from_ref(agent),
                    AddSkillOpts {
                        source: Some("local".into()),
                        content_hash: hash,
                        method: Some(method_str.into()),
                        global: Some(options.global),
                        version: Some(skill.catalog_version.clone()),
                        catalog_label,
                    },
                )?;
            }
            results.push(r);
            on_step();
        }
    }

    let success = results.iter().filter(|r| r.success).count() as i32;
    let failed = results.iter().filter(|r| !r.success).count() as i32;
    let details = serde_json::json!(results
        .iter()
        .map(|r| {
            serde_json::json!({
                "skill": r.skill,
                "agent": r.agent,
                "success": r.success,
                "error": r.error,
                "path": r.path,
            })
        })
        .collect::<Vec<_>>());

    let _ = log_audit(
        home,
        AuditEntry {
            action: "install".into(),
            skill_name: skills
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            agents: options
                .agents
                .iter()
                .filter_map(|a| get_def(a).map(|d| d.display_name.to_string()))
                .collect(),
            success,
            failed,
            forced: if options.audit_forced {
                Some(true)
            } else {
                None
            },
            details: Some(details),
            timestamp: None,
        },
    );

    Ok(results)
}

pub fn remove_skills(
    project_root: &Path,
    home: &Path,
    skill_name: &str,
    agents: &[String],
    options: &RemoveOptions,
) -> Vec<RemoveResult> {
    let safe = sanitize_name(skill_name);
    let mut out = Vec::new();

    let local_lock = lockfile::get_skill_from_lock(project_root, home, skill_name, false);
    let global_lock = lockfile::get_skill_from_lock(project_root, home, skill_name, true);
    let lock_entry_before = local_lock.clone();
    if !options.force && local_lock.is_none() && global_lock.is_none() {
        return agents
            .iter()
            .map(|a| {
                let disp = get_def(a).map(|d| d.display_name).unwrap_or(a.as_str());
                RemoveResult {
                    skill: skill_name.to_string(),
                    agent: disp.to_string(),
                    success: false,
                    error: Some("Skill not found in lockfile".into()),
                }
            })
            .collect();
    }

    for agent in agents {
        let Some(def) = get_def(agent) else {
            out.push(RemoveResult {
                skill: skill_name.to_string(),
                agent: agent.clone(),
                success: false,
                error: Some("unknown agent".into()),
            });
            continue;
        };

        let local_path = project_root.join(def.skills_dir).join(&safe);
        let global_path = (def.global_skills_dir)(home).join(&safe);

        let paths: Vec<(PathBuf, bool)> = match options.global {
            Some(true) => vec![(global_path, true)],
            Some(false) => vec![(local_path, false)],
            None => vec![(local_path, false), (global_path, true)],
        };

        let mut removed = false;
        let mut last_err: Option<String> = None;
        let mut removed_local = false;
        let mut removed_global = false;

        for (path, is_global) in paths {
            let base = if is_global {
                (def.global_skills_dir)(home)
            } else {
                project_root.join(def.skills_dir)
            };
            if !is_path_safe(&base, &path) {
                last_err = Some("Security: Invalid removal path".into());
                continue;
            }
            match fs::symlink_metadata(&path) {
                Ok(_) => {
                    if fs::remove_dir_all(&path).is_ok() {
                        removed = true;
                        if is_global {
                            removed_global = true;
                        } else {
                            removed_local = true;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => last_err = Some(e.to_string()),
            }
        }

        out.push(RemoveResult {
            skill: skill_name.to_string(),
            agent: def.display_name.to_string(),
            success: removed,
            error: if removed {
                None
            } else {
                Some(last_err.unwrap_or_else(|| "Skill not found".into()))
            },
        });

        if removed {
            if removed_local {
                let _ =
                    lockfile::remove_agent_from_lock(project_root, home, skill_name, agent, false);
            }
            if removed_global {
                let _ =
                    lockfile::remove_agent_from_lock(project_root, home, skill_name, agent, true);
            }
        }
    }

    // Remove canonical copy for local symlink installs
    let lock_after = lockfile::get_skill_from_lock(project_root, home, skill_name, false);
    let had_symlink =
        lock_entry_before.as_ref().and_then(|e| e.method.as_deref()) == Some("symlink");
    if had_symlink
        && lock_after
            .map(|e| e.agents.unwrap_or_default().is_empty())
            .unwrap_or(true)
    {
        let canonical = project_root
            .join(AGENTS_DIR)
            .join(CANONICAL_SKILL_FOLDER)
            .join(&safe);
        let _ = fs::remove_dir_all(&canonical);
    }

    let success = out.iter().filter(|r| r.success).count() as i32;
    let failed = out.iter().filter(|r| !r.success).count() as i32;
    if let Err(e) = log_audit(
        home,
        AuditEntry {
            action: "remove".into(),
            skill_name: skill_name.to_string(),
            agents: agents
                .iter()
                .filter_map(|a| get_def(a).map(|d| d.display_name.to_string()))
                .collect(),
            success,
            failed,
            forced: Some(options.force),
            details: Some(serde_json::to_value(&out).unwrap_or_default()),
            timestamp: None,
        },
    ) {
        eprintln!("aviso: falha ao registar auditoria — {e}");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn install_one_unknown_agent_returns_error() {
        let tmp = tempdir().unwrap();
        let project = tmp.path().join("proj");
        let home = tmp.path().join("home");
        let catalog = tmp.path().join("cat");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&catalog).unwrap();

        let skill_info = SkillInfo {
            name: "test-skill".into(),
            catalog_label: String::new(),
            description: "test".into(),
            catalog_version: "1.0.0".into(),
            tags: vec![],
            path: catalog.clone(),
            category: None,
        };

        install_one(
            &skill_info,
            "not-a-real-agent",
            &project,
            &home,
            InstallMethod::Copy,
            false,
        );
    }
}
