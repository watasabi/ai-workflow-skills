use std::collections::HashMap;
use std::fs;
use std::io::{Seek, Write};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use crate::constants::{AGENTS_DIR, LOCK_FILE, LOCK_FILE_BACKUP};
use crate::project_root::find_project_root;
use crate::types::{SkillLockEntry, SkillLockFile, LOCKFILE_VERSION};

fn lock_path(project_root: &Path, global: bool, home: &Path) -> PathBuf {
    if global {
        home.join(AGENTS_DIR).join(LOCK_FILE)
    } else {
        project_root.join(AGENTS_DIR).join(LOCK_FILE)
    }
}

fn backup_path(project_root: &Path, global: bool, home: &Path) -> PathBuf {
    if global {
        home.join(AGENTS_DIR).join(LOCK_FILE_BACKUP)
    } else {
        project_root.join(AGENTS_DIR).join(LOCK_FILE_BACKUP)
    }
}

pub fn read_skill_lock(project_root: &Path, home: &Path, global: bool) -> SkillLockFile {
    let path = lock_path(project_root, global, home);
    let content = fs::read_to_string(&path).ok();
    let Some(content) = content else {
        return empty_lock();
    };
    match serde_json::from_str::<SkillLockFile>(&content) {
        Ok(f) => migrate(f),
        Err(e) => {
            eprintln!(
                "aviso: lockfile JSON inválido em {} — {}. Estado tratado como vazio.",
                path.display(),
                e
            );
            empty_lock()
        }
    }
}

fn migrate(mut f: SkillLockFile) -> SkillLockFile {
    if f.version < LOCKFILE_VERSION {
        f.version = LOCKFILE_VERSION;
        for e in f.skills.values_mut() {
            if e.method.is_none() {
                e.method = Some("copy".into());
            }
            if e.global.is_none() {
                e.global = Some(false);
            }
            // `catalog_label` stays None for legacy entries — provenance unknown; the TUI grants
            // these a best-effort match to the first row of that name.
        }
    }
    f
}

fn empty_lock() -> SkillLockFile {
    SkillLockFile {
        version: LOCKFILE_VERSION,
        skills: HashMap::new(),
    }
}

#[cfg(unix)]
fn flock_exclusive(file: &std::fs::File) -> std::io::Result<()> {
    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn funlock(file: &std::fs::File) -> std::io::Result<()> {
    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_UN) };
    if ret != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(unix))]
fn flock_exclusive(_file: &std::fs::File) -> std::io::Result<()> {
    Ok(()) // No-op on non-Unix
}

#[cfg(not(unix))]
fn funlock(_file: &std::fs::File) -> std::io::Result<()> {
    Ok(())
}

pub fn write_skill_lock(
    project_root: &Path,
    home: &Path,
    global: bool,
    lock: &SkillLockFile,
) -> std::io::Result<()> {
    let path = lock_path(project_root, global, home);
    let backup = backup_path(project_root, global, home);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Open the destination file for locking (must exist first)
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    // Acquire exclusive lock
    flock_exclusive(&f)?;

    // Ensure we unlock no matter what
    let result = (|| -> std::io::Result<()> {
        // Backup current content if it exists
        if let Ok(content) = fs::read_to_string(&path) {
            if !content.is_empty() {
                if let Err(e) = fs::copy(&path, &backup) {
                    eprintln!("aviso: backup do lockfile falhou — {e}");
                }
            }
        }

        let json = serde_json::to_string_pretty(lock)?;
        // Write directly to locked file, truncating first
        f.set_len(0)?;
        f.seek(std::io::SeekFrom::Start(0))?;
        f.write_all(json.as_bytes())?;
        f.sync_all()?;
        Ok(())
    })();

    funlock(&f)?;

    result
}

pub fn add_skill_to_lock(
    project_root: &Path,
    home: &Path,
    skill_name: &str,
    agents: &[String],
    opts: AddSkillOpts,
) -> std::io::Result<()> {
    let global = opts.global.unwrap_or(false);
    let mut lock = read_skill_lock(project_root, home, global);
    let now = chrono::Utc::now().to_rfc3339();
    let existing = lock.skills.get(skill_name).cloned();
    let mut merged_agents: Vec<String> = existing
        .as_ref()
        .and_then(|e| e.agents.clone())
        .unwrap_or_default();
    for a in agents {
        if !merged_agents.contains(a) {
            merged_agents.push(a.clone());
        }
    }

    lock.skills.insert(
        skill_name.to_string(),
        SkillLockEntry {
            name: skill_name.to_string(),
            source: opts.source.unwrap_or_else(|| "local".into()),
            content_hash: opts
                .content_hash
                .or_else(|| existing.as_ref().and_then(|e| e.content_hash.clone())),
            installed_at: existing
                .as_ref()
                .map(|e| e.installed_at.clone())
                .unwrap_or_else(|| now.clone()),
            updated_at: now,
            agents: Some(merged_agents),
            method: Some(opts.method.unwrap_or_else(|| "copy".into())),
            global: Some(global),
            version: opts
                .version
                .or_else(|| existing.as_ref().and_then(|e| e.version.clone())),
            catalog_label: opts.catalog_label,
        },
    );

    write_skill_lock(project_root, home, global, &lock)
}

pub struct AddSkillOpts {
    pub source: Option<String>,
    pub content_hash: Option<String>,
    pub method: Option<String>,
    pub global: Option<bool>,
    pub version: Option<String>,
    /// Rótulo do catálogo de origem; gravado no `SkillLockEntry` para provenance.
    pub catalog_label: Option<String>,
}

pub fn get_skill_from_lock(
    project_root: &Path,
    home: &Path,
    skill_name: &str,
    global: bool,
) -> Option<SkillLockEntry> {
    let lock = read_skill_lock(project_root, home, global);
    lock.skills.get(skill_name).cloned()
}

pub fn remove_agent_from_lock(
    project_root: &Path,
    home: &Path,
    skill_name: &str,
    agent: &str,
    global: bool,
) -> std::io::Result<bool> {
    let mut lock = read_skill_lock(project_root, home, global);
    let Some(entry) = lock.skills.get_mut(skill_name) else {
        return Ok(false);
    };
    let prev = entry.agents.clone().unwrap_or_default();
    let updated: Vec<String> = prev.iter().filter(|a| *a != agent).cloned().collect();
    if updated.len() == prev.len() {
        return Ok(false);
    }
    if updated.is_empty() {
        lock.skills.remove(skill_name);
    } else {
        entry.agents = Some(updated);
        entry.updated_at = chrono::Utc::now().to_rfc3339();
    }
    write_skill_lock(project_root, home, global, &lock)?;
    Ok(true)
}

/// Raiz do projeto para lockfile local.
pub fn project_root_for_lock(cwd: Option<&Path>) -> PathBuf {
    find_project_root(cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillLockEntry, SkillLockFile, LOCKFILE_VERSION};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn read_missing_lock_is_empty() {
        let root = tempdir().unwrap();
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let lock = read_skill_lock(root.path(), &home, false);
        assert!(lock.skills.is_empty());
    }

    #[test]
    fn write_read_roundtrip_local() {
        let root = tempdir().unwrap();
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let mut lock = SkillLockFile {
            version: LOCKFILE_VERSION,
            skills: HashMap::new(),
        };
        lock.skills.insert(
            "my-skill".into(),
            SkillLockEntry {
                name: "my-skill".into(),
                source: "local".into(),
                content_hash: Some("hash1".into()),
                installed_at: "t0".into(),
                updated_at: "t1".into(),
                agents: Some(vec!["cursor".into()]),
                method: Some("copy".into()),
                global: Some(false),
                version: Some("1.0.0".into()),
                catalog_label: Some("cat-a".into()),
            },
        );
        write_skill_lock(root.path(), &home, false, &lock).unwrap();
        let read = read_skill_lock(root.path(), &home, false);
        assert_eq!(read.skills.len(), 1);
        let entry = read.skills.get("my-skill").unwrap();
        assert_eq!(entry.content_hash.as_deref(), Some("hash1"));
        assert_eq!(entry.catalog_label.as_deref(), Some("cat-a"));
    }

    #[test]
    fn migrate_v2_keeps_legacy_catalog_label_none() {
        let root = tempdir().unwrap();
        let home = root.path().join("home");
        std::fs::create_dir_all(home.join(crate::constants::AGENTS_DIR)).unwrap();
        let legacy = r#"{
          "version": 2,
          "skills": {
            "legacy-skill": {
              "name": "legacy-skill",
              "source": "local",
              "installed_at": "t0",
              "updated_at": "t0",
              "agents": ["cursor"]
            }
          }
        }"#;
        let lock_path = home
            .join(crate::constants::AGENTS_DIR)
            .join(crate::constants::LOCK_FILE);
        std::fs::write(&lock_path, legacy).unwrap();
        let lock = read_skill_lock(root.path(), &home, true);
        assert_eq!(lock.version, LOCKFILE_VERSION);
        let entry = lock.skills.get("legacy-skill").unwrap();
        assert!(entry.catalog_label.is_none());
        assert_eq!(entry.method.as_deref(), Some("copy"));
        assert_eq!(entry.global, Some(false));
    }

    #[test]
    fn get_skill_from_lock_missing_returns_none() {
        let root = tempdir().unwrap();
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let entry = get_skill_from_lock(root.path(), &home, "nonexistent", false);
        assert!(entry.is_none());
    }

    #[test]
    fn remove_agent_from_lock_unknown_agent_ok() {
        let root = tempdir().unwrap();
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let mut lock = SkillLockFile {
            version: LOCKFILE_VERSION,
            skills: HashMap::new(),
        };
        lock.skills.insert(
            "my-skill".into(),
            SkillLockEntry {
                name: "my-skill".into(),
                source: "local".into(),
                content_hash: None,
                installed_at: "t0".into(),
                updated_at: "t0".into(),
                agents: Some(vec!["cursor".into()]),
                method: Some("copy".into()),
                global: Some(false),
                version: None,
                catalog_label: None,
            },
        );
        write_skill_lock(root.path(), &home, false, &lock).unwrap();
        let result =
            remove_agent_from_lock(root.path(), &home, "my-skill", "unknown-agent", false).unwrap();
        assert!(!result);
        let entry = get_skill_from_lock(root.path(), &home, "my-skill", false).unwrap();
        assert!(entry.agents.as_ref().unwrap().contains(&"cursor".into()));
    }
}
