//! Fixtures compartilhados entre testes de integração.

use std::fs;
use std::io::Write;
use std::path::Path;

pub fn write_skill(dir: &Path, name: &str, category: Option<&str>, skill_folder: &str) {
    let base = match category {
        Some(c) => dir.join(format!("({c})")).join(skill_folder),
        None => dir.join(skill_folder),
    };
    fs::create_dir_all(&base).unwrap();
    let mut f = fs::File::create(base.join("SKILL.md")).unwrap();
    writeln!(
        f,
        "---\nname: {name}\ndescription: Test skill\n---\n\n# {name}\n"
    )
    .unwrap();
    fs::write(
        base.join("skill.manifest.json"),
        r#"{"version": "1.0.0", "tags": ["demo", "test"]}"#,
    )
    .unwrap();
    // Paridade com a doc do Cursor (skills com pasta opcional `scripts/` — ver cursor-docs-local/03-skills.md).
    let scripts = base.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("noop.sh"), "#!/bin/sh\nexit 0\n").unwrap();
}

/// Catálogo mínimo válido: `skills/(demo)/demo-skill` com manifest e SKILL.md.
pub fn minimal_catalog_root(catalog: &Path) {
    let skills = catalog.join("skills");
    fs::create_dir_all(skills.join("(demo)")).unwrap();
    write_skill(&skills, "demo-skill", Some("demo"), "demo-skill");
}

/// Marca diretório como raiz de projeto para `find_project_root` (ex.: `package.json`).
#[allow(dead_code)] // nem todos os crates de teste importam esta função
pub fn project_with_marker(project: &Path) {
    fs::write(project.join("package.json"), "{}\n").unwrap();
}
