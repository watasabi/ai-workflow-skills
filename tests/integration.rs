//! Testes de integração leves: catálogo, sanitização e registry.

mod common;

use std::fs;

use ai_workflow_skills::catalog::{
    discover_skills, generate_registry, generate_registry_merged, resolve_skills_root,
};
use ai_workflow_skills::sanitize::{is_path_safe, sanitize_name, to_slug};
use tempfile::tempdir;

#[test]
fn sanitize_strips_traversal() {
    assert_eq!(sanitize_name("../../../etc/passwd"), "etcpasswd");
    assert_eq!(sanitize_name("my-skill"), "my-skill");
}

#[test]
fn to_slug_basic() {
    assert_eq!(to_slug("My Cool Skill"), "my-cool-skill");
}

#[test]
fn path_safe_under_base() {
    let tmp = tempdir().unwrap();
    let base = tmp.path().join("base");
    fs::create_dir_all(&base).unwrap();
    let target = base.join("child").join("x");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    assert!(is_path_safe(&base, &target));
}

#[test]
fn path_safe_rejects_sibling_outside_base() {
    let tmp = tempdir().unwrap();
    let base = tmp.path().join("base");
    let outside = tmp.path().join("outside").join("x");
    fs::create_dir_all(&base).unwrap();
    fs::create_dir_all(outside.parent().unwrap()).unwrap();
    assert!(!is_path_safe(&base, &outside));
}

#[test]
fn path_safe_rejects_path_traversal() {
    let tmp = tempdir().unwrap();
    let base = tmp.path().join("base");
    fs::create_dir_all(&base).unwrap();
    // The traversal path goes outside base via .. so it must be rejected
    let traversal = tmp.path().join("outside");
    fs::create_dir_all(&traversal).unwrap();
    assert!(!is_path_safe(&base, &traversal));
}

#[test]
fn discover_and_registry_roundtrip() {
    let tmp = tempdir().unwrap();
    common::minimal_catalog_root(tmp.path());
    let skills = tmp.path().join("skills");

    let root = resolve_skills_root(tmp.path()).unwrap();
    assert_eq!(root, skills);

    let list = discover_skills(&root).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "demo-skill");
    // `discover_skills` is the low-level path: it leaves `catalog_label` empty. The real
    // path that fills it (`load_catalog_skills`) is covered by `cli::tests::load_catalog_skills_sets_catalog_label`.
    assert!(list[0].catalog_label.is_empty());
    assert_eq!(list[0].catalog_version, "1.0.0");
    assert_eq!(list[0].tags, vec!["demo".to_string(), "test".to_string()]);

    let reg = generate_registry(tmp.path()).unwrap();
    assert_eq!(reg.skills.len(), 1);
    assert_eq!(reg.skills[0].name, "demo-skill");
    assert_eq!(reg.skills[0].version.as_deref(), Some("1.0.0"));
    assert_eq!(
        reg.skills[0].tags,
        Some(vec!["demo".to_string(), "test".to_string()])
    );
    assert!(!reg.skills[0].content_hash.is_empty());
}

#[test]
fn generate_registry_merged_combines_two_catalogs() {
    let tmp = tempdir().unwrap();
    let cat1 = tmp.path().join("cat1");
    let cat2 = tmp.path().join("cat2");
    fs::create_dir_all(&cat1).unwrap();
    fs::create_dir_all(&cat2).unwrap();
    common::minimal_catalog_root(&cat1);
    let skills2 = cat2.join("skills");
    fs::create_dir_all(skills2.join("(demo)")).unwrap();
    common::write_skill(&skills2, "other-skill", Some("demo"), "other-skill");

    let reg = generate_registry_merged(&[cat1, cat2]).unwrap();
    let names: Vec<&str> = reg.skills.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"demo-skill"));
    assert!(names.contains(&"other-skill"));
}
