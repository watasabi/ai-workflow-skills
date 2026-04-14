//! Testes do binário `ai-workflow-skills` (subcomandos reais).

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_no_subcommand_without_tty_prints_help_and_exits_2() {
    let home = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(
        help.contains("ai-workflow-skills") && (help.contains("list") || help.contains("List")),
        "expected CLI help on stdout, got: {help}"
    );
}

#[test]
fn cli_list_catalog_prints_skill() {
    let home = tempfile::tempdir().unwrap();
    let cat = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat.path());
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["list", "--catalog", cat.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"));
}

#[test]
fn cli_list_respects_catalog_env_var() {
    let home = tempfile::tempdir().unwrap();
    let cat = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat.path());
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .env("AI_WORKFLOW_SKILLS_CATALOG", cat.path().to_str().unwrap())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"));
}

#[test]
fn cli_list_two_catalogs_duplicate_skill_names_lists_both() {
    let home = tempfile::tempdir().unwrap();
    let cat1 = tempfile::tempdir().unwrap();
    let cat2 = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat1.path());
    common::minimal_catalog_root(cat2.path());

    let assert = Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "list",
            "--catalog",
            cat1.path().to_str().unwrap(),
            "--catalog",
            cat2.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(stdout.matches("demo-skill").count(), 2);
    assert!(
        stdout.contains("Total: 2 skill(s)"),
        "expected two merged rows: {stdout}"
    );
    // T023: each merged row must carry its own catalog label (the tempdir basenames differ).
    let label1 = cat1.path().file_name().unwrap().to_str().unwrap();
    let label2 = cat2.path().file_name().unwrap().to_str().unwrap();
    assert!(
        stdout.contains(&format!("(catálogo: {label1})")),
        "row from {label1} missing its label: {stdout}"
    );
    assert!(
        stdout.contains(&format!("(catálogo: {label2})")),
        "row from {label2} missing its label: {stdout}"
    );
}

#[test]
fn cli_list_two_catalogs_merges_skills() {
    let home = tempfile::tempdir().unwrap();
    let cat1 = tempfile::tempdir().unwrap();
    let cat2 = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat1.path());
    let skills2 = cat2.path().join("skills");
    fs::create_dir_all(skills2.join("(demo)")).unwrap();
    common::write_skill(&skills2, "other-skill", Some("demo"), "other-skill");

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "list",
            "--catalog",
            cat1.path().to_str().unwrap(),
            "--catalog",
            cat2.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"))
        .stdout(predicate::str::contains("other-skill"));
}

#[test]
fn cli_list_respects_catalogs_env_var() {
    let home = tempfile::tempdir().unwrap();
    let cat1 = tempfile::tempdir().unwrap();
    let cat2 = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat1.path());
    let skills2 = cat2.path().join("skills");
    fs::create_dir_all(skills2.join("(demo)")).unwrap();
    common::write_skill(&skills2, "other-skill", Some("demo"), "other-skill");

    let merged = format!(
        "{}|||{}",
        cat1.path().to_str().unwrap(),
        cat2.path().to_str().unwrap()
    );

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .env("AI_WORKFLOW_SKILLS_CATALOGS", merged)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"))
        .stdout(predicate::str::contains("other-skill"));
}

#[test]
fn cli_list_with_tag_filters() {
    let home = tempfile::tempdir().unwrap();
    let cat = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat.path());
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "list",
            "--catalog",
            cat.path().to_str().unwrap(),
            "--tag",
            "demo",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"));
}

#[test]
fn cli_list_invalid_tag_fails() {
    let home = tempfile::tempdir().unwrap();
    let cat = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat.path());
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "list",
            "--catalog",
            cat.path().to_str().unwrap(),
            "--tag",
            "Invalid_Tag!",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("tag inválida"));
}

#[test]
fn cli_generate_registry_writes_file() {
    let home = tempfile::tempdir().unwrap();
    let cat = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat.path());
    let out = home.path().join("out-reg.json");
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "generate-registry",
            "--catalog",
            cat.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(out.is_file());
    let raw = fs::read_to_string(&out).unwrap();
    assert!(raw.contains("demo-skill"));
}

#[test]
fn cli_generate_registry_merges_two_catalog_flags() {
    let home = tempfile::tempdir().unwrap();
    let cat1 = tempfile::tempdir().unwrap();
    let cat2 = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat1.path());
    let skills2 = cat2.path().join("skills");
    fs::create_dir_all(skills2.join("(demo)")).unwrap();
    common::write_skill(&skills2, "other-skill", Some("demo"), "other-skill");
    let out = home.path().join("merged-reg.json");
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "generate-registry",
            "--catalog",
            cat1.path().to_str().unwrap(),
            "--catalog",
            cat2.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let raw = fs::read_to_string(&out).unwrap();
    assert!(raw.contains("demo-skill"));
    assert!(raw.contains("other-skill"));
}

#[test]
fn cli_cache_path_prints_under_home() {
    let home = tempfile::tempdir().unwrap();
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["cache", "--path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".cache"));
}

#[test]
fn cli_audit_path_prints_log_path() {
    let home = tempfile::tempdir().unwrap();
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["audit", "--path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".ai-workflow-skills"));
}

// T6 — Fix T011: Add list --tag zero-results test
#[test]
fn cli_list_tag_with_no_matching_skills() {
    let home = tempfile::tempdir().unwrap();
    let cat = tempfile::tempdir().unwrap();
    common::minimal_catalog_root(cat.path());
    // demo-skill has tag "demo" — request a different tag
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args([
            "list",
            "--catalog",
            cat.path().to_str().unwrap(),
            "--tag",
            "nonexistent-tag",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("0").or(predicate::str::contains("nenhuma")));
}

// T7 — Fix T009: Add cache --clear test
#[test]
fn cli_cache_clear_removes_cache_directory() {
    let home = tempfile::tempdir().unwrap();
    // Pre-create cache with a registry file
    // Cache dir is: home/.cache/ai-workflow-skills
    let cache_dir = home.path().join(".cache").join("ai-workflow-skills");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("registry.json"), "{}").unwrap();
    assert!(cache_dir.join("registry.json").is_file());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["cache", "--clear"])
        .assert()
        .success();

    // Cache dir should be gone
    assert!(!cache_dir.exists());
}

// T7 — Fix T009: Add cache --clear-registry test
#[test]
fn cli_cache_clear_registry_removes_only_registry() {
    let home = tempfile::tempdir().unwrap();
    // Cache dir is: home/.cache/ai-workflow-skills
    let cache_dir = home.path().join(".cache").join("ai-workflow-skills");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("registry.json"), "{}").unwrap();
    std::fs::write(cache_dir.join("other-file.json"), "{}").unwrap();

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["cache", "--clear-registry"])
        .assert()
        .success();

    // Only registry.json should be deleted
    assert!(!cache_dir.join("registry.json").exists());
    assert!(cache_dir.join("other-file.json").exists());
}

// T8 — Fix T010: Add audit subcommand tests
#[test]
fn cli_audit_missing_log_does_not_crash() {
    let home = tempfile::tempdir().unwrap();
    // audit.log doesn't exist — should handle gracefully
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["audit"])
        .assert()
        .success();
}

#[test]
fn cli_audit_with_entries_shows_them() {
    let home = tempfile::tempdir().unwrap();
    // Pre-create audit.log with a valid entry
    let audit_dir = home.path().join(".ai-workflow-skills");
    std::fs::create_dir_all(&audit_dir).unwrap();
    let audit_log = audit_dir.join("audit.log");
    // Use camelCase for skillName as per #[serde(rename_all = "camelCase")]
    let entry = serde_json::json!({
        "action": "install",
        "skillName": "demo-skill",
        "agents": ["Cursor"],
        "success": 1,
        "failed": 0,
        "timestamp": "2026-04-07T10:00:00Z"
    });
    std::fs::write(&audit_log, format!("{entry}\n")).unwrap();

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["audit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"));
}

#[test]
fn cli_audit_limit_shows_n_entries() {
    let home = tempfile::tempdir().unwrap();
    let audit_dir = home.path().join(".ai-workflow-skills");
    std::fs::create_dir_all(&audit_dir).unwrap();
    let audit_log = audit_dir.join("audit.log");
    let mut lines = String::new();
    for i in 0..5 {
        let entry = serde_json::json!({
            "action": "install",
            "skillName": format!("skill-{}", i),
            "agents": ["Cursor"],
            "success": 1,
            "failed": 0,
            "timestamp": format!("2026-04-07T10:00:0{}Z", i)
        });
        lines.push_str(&format!("{}\n", serde_json::to_string(&entry).unwrap()));
    }
    std::fs::write(&audit_log, lines).unwrap();

    let out = Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .env("HOME", home.path())
        .args(["audit", "-n", "2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // read_audit_log reverses lines (newest first): last written line is skill-4, then skill-3
    assert!(
        stdout.contains("skill-4") && stdout.contains("skill-3"),
        "expected -n 2 to return the two newest entries: {stdout}"
    );
}
