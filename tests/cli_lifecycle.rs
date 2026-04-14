//! Instalação, listagem instalada, atualização e remoção via CLI.

mod common;

use assert_cmd::Command;
use ai_workflow_skills::core::lockfile;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn cli_install_without_skills_noninteractive_errors() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["install", "--catalog", cat.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("skill")
                .or(predicate::str::contains("TTY"))
                .or(predicate::str::contains("terminal")),
        );
}

#[test]
fn cli_install_without_agent_defaults_to_cursor() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());
    // Sem pastas de agente detectáveis: destino padrão é só Cursor → `.cursor/skills/`.
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "--catalog",
            cat.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(project
        .path()
        .join(".cursor/skills/demo-skill/SKILL.md")
        .is_file());
}

#[test]
fn cli_install_list_installed_remove_roundtrip() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let skill_root = project.path().join(".cursor/skills/demo-skill");
    assert!(skill_root.join("SKILL.md").is_file());
    assert!(skill_root.join("scripts/noop.sh").is_file());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["list", "--installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo-skill"));

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["remove", "-s", "demo-skill", "-a", "cursor"])
        .assert()
        .success();

    assert!(!skill_root.join("SKILL.md").exists());
}

#[test]
fn cli_update_after_catalog_hash_change() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let skill_md = cat.path().join("skills/(demo)/demo-skill/SKILL.md");
    let mut body = fs::read_to_string(&skill_md).unwrap();
    body.push_str("\n<!-- test bump -->\n");
    fs::write(&skill_md, &body).unwrap();
    fs::write(
        cat.path()
            .join("skills/(demo)/demo-skill/skill.manifest.json"),
        r#"{"version": "1.0.1", "tags": ["demo", "test"]}"#,
    )
    .unwrap();

    // Capture lockfile timestamp before update
    let lock_before = lockfile::read_skill_lock(project.path(), home.path(), false);
    let ts_before = lock_before
        .skills
        .get("demo-skill")
        .map(|e| e.updated_at.clone());

    // Update command
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["update", "--catalog", cat.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("atualiz"));

    // Verify lockfile was updated after the catalog hash change
    let lock = lockfile::read_skill_lock(project.path(), home.path(), false);
    let entry = lock
        .skills
        .get("demo-skill")
        .expect("demo-skill should be in lockfile");
    if let Some(ts_before) = ts_before {
        assert!(
            entry.updated_at.as_str() > ts_before.as_str(),
            "updated_at should be refreshed after update"
        );
    } else {
        panic!("demo-skill missing from lockfile before update");
    }
}

#[test]
fn cli_update_with_skill_filter() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());

    // Install first
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Update with --skill filter
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "update",
            "--catalog",
            cat.path().to_str().unwrap(),
            "-s",
            "demo-skill",
        ])
        .assert()
        .success();
}

#[test]
fn cli_update_catalog_unchanged_is_noop() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());

    // Install first
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let lock_before = lockfile::read_skill_lock(project.path(), home.path(), false);
    let hash_before = lock_before
        .skills
        .get("demo-skill")
        .and_then(|e| e.content_hash.clone());

    // Update with unchanged catalog — same catalog bytes ⇒ mesmo content_hash no lockfile
    let out = Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["update", "--catalog", cat.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "update should succeed: {:?}",
        out.status
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.to_lowercase().contains("erro"),
        "unexpected error on stderr: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Concluído"),
        "expected install summary on stdout: {stdout}"
    );

    let lock_after = lockfile::read_skill_lock(project.path(), home.path(), false);
    let hash_after = lock_after
        .skills
        .get("demo-skill")
        .and_then(|e| e.content_hash.clone());
    assert_eq!(
        hash_before, hash_after,
        "content_hash should match when catalog is unchanged"
    );
}

#[test]
fn cli_remove_nonexistent_skill_fails() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    common::project_with_marker(project.path());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["remove", "-s", "does-not-exist", "-a", "cursor"])
        .assert()
        .failure();
}

#[test]
fn cli_install_and_update_with_two_catalog_flags() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat1 = tempdir().unwrap();
    let cat2 = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat1.path());
    let skills2 = cat2.path().join("skills");
    fs::create_dir_all(skills2.join("(demo)")).unwrap();
    common::write_skill(&skills2, "other-skill", Some("demo"), "other-skill");

    let c1 = cat1.path().to_str().unwrap();
    let c2 = cat2.path().to_str().unwrap();

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            c1,
            "--catalog",
            c2,
        ])
        .assert()
        .success();

    let lock_after_install = lockfile::read_skill_lock(project.path(), home.path(), false);
    let hash_after_install = lock_after_install
        .skills
        .get("demo-skill")
        .and_then(|e| e.content_hash.clone());

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["update", "--catalog", c1, "--catalog", c2])
        .assert()
        .success();

    let lock_after_update = lockfile::read_skill_lock(project.path(), home.path(), false);
    let hash_after_update = lock_after_update
        .skills
        .get("demo-skill")
        .and_then(|e| e.content_hash.clone());
    assert_eq!(
        hash_after_install, hash_after_update,
        "content_hash unchanged when catalog files unchanged (multi-catalog)"
    );
}

// T021 — dois catálogos expõem o mesmo nome; `install -s demo-skill` (sem prefixo) falha com lista
// de candidatos; com prefixo `catalogo/skill` funciona.
#[test]
fn cli_install_ambiguous_name_errors_with_candidates() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let parent = tempdir().unwrap();
    common::project_with_marker(project.path());

    let cat1 = parent.path().join("cat-alpha");
    let cat2 = parent.path().join("cat-beta");
    fs::create_dir_all(&cat1).unwrap();
    fs::create_dir_all(&cat2).unwrap();
    common::minimal_catalog_root(&cat1);
    common::minimal_catalog_root(&cat2);

    // Ambiguous — no prefix.
    let out = Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat1.to_str().unwrap(),
            "--catalog",
            cat2.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "expected failure on ambiguous name");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("ambígua"),
        "missing ambiguity hint: {stderr}"
    );
    assert!(
        stderr.contains("cat-alpha/demo-skill") && stderr.contains("cat-beta/demo-skill"),
        "candidates list missing: {stderr}"
    );

    // Prefixed — unambiguous, installs from cat-alpha.
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "cat-alpha/demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat1.to_str().unwrap(),
            "--catalog",
            cat2.to_str().unwrap(),
        ])
        .assert()
        .success();

    let lock = lockfile::read_skill_lock(project.path(), home.path(), false);
    assert_eq!(
        lock.skills
            .get("demo-skill")
            .and_then(|e| e.catalog_label.as_deref()),
        Some("cat-alpha"),
    );
}

// T020 — instalar a mesma skill a partir de dois catálogos diferentes: a segunda instalação
// substitui a primeira no disco e no lockfile; a provenance (`catalog_label`) reflete o novo catálogo.
#[test]
fn cli_install_same_name_from_two_catalogs_flips_provenance() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let parent = tempdir().unwrap();
    common::project_with_marker(project.path());

    let cat1 = parent.path().join("cat-alpha");
    let cat2 = parent.path().join("cat-beta");
    fs::create_dir_all(&cat1).unwrap();
    fs::create_dir_all(&cat2).unwrap();
    common::minimal_catalog_root(&cat1);
    common::minimal_catalog_root(&cat2);
    // Differentiate the two catalogs' content so hashes diverge.
    fs::write(
        cat2.join("skills/(demo)/demo-skill/SKILL.md"),
        "---\nname: demo-skill\ndescription: beta\n---\n\n# beta\n",
    )
    .unwrap();
    // File exclusive to cat1 — must not survive a replacement install from cat2.
    fs::write(
        cat1.join("skills/(demo)/demo-skill/cat1_only.txt"),
        "only in cat1",
    )
    .unwrap();

    // Install from cat1 first.
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat1.to_str().unwrap(),
        ])
        .assert()
        .success();

    let lock_after_first = lockfile::read_skill_lock(project.path(), home.path(), false);
    let first_entry = lock_after_first.skills.get("demo-skill").unwrap();
    assert_eq!(
        first_entry.catalog_label.as_deref(),
        Some("cat-alpha"),
        "first install records cat-alpha provenance"
    );
    let hash_alpha = first_entry.content_hash.clone().unwrap();

    // Install same skill name from cat2 — should flip provenance and content.
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat2.to_str().unwrap(),
        ])
        .assert()
        .success();

    let lock_after_second = lockfile::read_skill_lock(project.path(), home.path(), false);
    let second_entry = lock_after_second.skills.get("demo-skill").unwrap();
    assert_eq!(
        second_entry.catalog_label.as_deref(),
        Some("cat-beta"),
        "second install flips provenance to cat-beta"
    );
    assert_ne!(
        second_entry.content_hash.as_deref().unwrap(),
        hash_alpha.as_str(),
        "content_hash should change after replacement"
    );

    // Disk content should match cat-beta.
    let installed =
        fs::read_to_string(project.path().join(".cursor/skills/demo-skill/SKILL.md")).unwrap();
    assert!(
        installed.contains("beta"),
        "installed skill should be cat-beta's content: {installed}"
    );
    // cat1_only.txt was copied on the first install; `copy_dir_all` must wipe the
    // destination before writing cat2's content, so the stale file should be gone.
    let stale = project
        .path()
        .join(".cursor/skills/demo-skill/cat1_only.txt");
    assert!(
        !stale.exists(),
        "stale file from cat1 should not survive replacement install from cat2"
    );
}

// T022 — robustez: path de catálogo não canonicalizado (`./cat-rel`) deve instalar e registar content_hash.
#[test]
fn cli_install_with_relative_catalog_path_records_hash() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let parent = tempdir().unwrap();
    common::project_with_marker(project.path());
    let cat_abs = parent.path().join("cat-rel");
    fs::create_dir_all(&cat_abs).unwrap();
    common::minimal_catalog_root(&cat_abs);

    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(parent.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            "./cat-rel",
        ])
        // `find_project_root` falls back to cwd when no marker exists, so install lands in `parent`.
        .assert()
        .success();

    let lock = lockfile::read_skill_lock(parent.path(), home.path(), false);
    let entry = lock
        .skills
        .get("demo-skill")
        .expect("demo-skill should be in lockfile");
    assert!(
        entry.content_hash.is_some(),
        "content_hash must be recorded even with non-canonical catalog path"
    );
}

#[test]
fn cli_remove_with_force_removes_without_lock() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let cat = tempdir().unwrap();
    common::project_with_marker(project.path());
    common::minimal_catalog_root(cat.path());

    // Install first
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args([
            "install",
            "-s",
            "demo-skill",
            "-a",
            "cursor",
            "--catalog",
            cat.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // --force remove works
    Command::cargo_bin("ai-workflow-skills")
        .unwrap()
        .current_dir(project.path())
        .env("HOME", home.path())
        .args(["remove", "-s", "demo-skill", "-a", "cursor", "--force"])
        .assert()
        .success();
}
