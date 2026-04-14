//! Testes diretos de `install_skills` / `remove_skills` (sem binário).

mod common;

use ai_workflow_skills::catalog::{discover_skills, resolve_skills_root};
use ai_workflow_skills::core::installer::{install_skills, remove_skills};
use ai_workflow_skills::core::lockfile;
use ai_workflow_skills::types::{InstallMethod, InstallOptions, RemoveOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn install_copy_then_remove_roundtrip() {
    let tmp = tempdir().unwrap();
    let project = tmp.path().join("proj");
    let home = tmp.path().join("home");
    let catalog = tmp.path().join("catalog");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    common::project_with_marker(&project);
    common::minimal_catalog_root(&catalog);

    let skills_root = resolve_skills_root(&catalog).unwrap();
    let skills = discover_skills(&skills_root).unwrap();
    assert_eq!(skills.len(), 1);

    let opts = InstallOptions {
        global: false,
        method: InstallMethod::Copy,
        agents: vec!["cursor".to_string()],
        skills: vec!["demo-skill".to_string()],
        force_update: false,
        audit_forced: false,
    };

    let results = install_skills(&project, &home, &skills, &opts, || ()).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].success, "{:?}", results[0].error);

    let installed = project.join(".cursor/skills/demo-skill/SKILL.md");
    assert!(installed.is_file());
    // Layout Cursor: pasta da skill + SKILL.md + opcional `scripts/` (cursor-docs-local/03-skills.md).
    assert!(project
        .join(".cursor/skills/demo-skill/scripts/noop.sh")
        .is_file());

    let lock = lockfile::read_skill_lock(&project, &home, false);
    assert!(lock.skills.contains_key("demo-skill"));

    let remove_opts = RemoveOptions {
        global: None,
        force: false,
    };
    let removed = remove_skills(
        &project,
        &home,
        "demo-skill",
        &["cursor".to_string()],
        &remove_opts,
    );
    assert!(removed.iter().all(|r| r.success), "{removed:?}");

    assert!(!installed.exists());
    let lock_after = lockfile::read_skill_lock(&project, &home, false);
    assert!(!lock_after.skills.contains_key("demo-skill"));
}

#[test]
fn install_skills_unknown_agent_records_failure() {
    let tmp = tempdir().unwrap();
    let project = tmp.path().join("proj");
    let home = tmp.path().join("home");
    let catalog = tmp.path().join("catalog");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    common::minimal_catalog_root(&catalog);

    let skills_root = resolve_skills_root(&catalog).unwrap();
    let skills = discover_skills(&skills_root).unwrap();

    let opts = InstallOptions {
        global: false,
        method: InstallMethod::Copy,
        agents: vec!["not-a-real-agent-id".to_string()],
        skills: vec!["demo-skill".to_string()],
        force_update: false,
        audit_forced: false,
    };

    let results = install_skills(&project, &home, &skills, &opts, || ()).unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(results[0]
        .error
        .as_deref()
        .unwrap_or("")
        .contains("unknown"));
}
