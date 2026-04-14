use std::fs;
use std::path::{Path, PathBuf};

use crate::constants::{CACHE_BASE_DIR, CACHE_NAMESPACE, REGISTRY_CACHE_FILENAME};

pub fn cache_dir(home: &Path) -> PathBuf {
    home.join(CACHE_BASE_DIR).join(CACHE_NAMESPACE)
}

pub fn registry_cache_path(home: &Path) -> PathBuf {
    cache_dir(home).join(REGISTRY_CACHE_FILENAME)
}

pub fn clear_all_cache(home: &Path) -> std::io::Result<()> {
    let dir = cache_dir(home);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

pub fn clear_registry_cache(home: &Path) -> std::io::Result<()> {
    let p = registry_cache_path(home);
    if p.exists() {
        fs::remove_file(&p)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cache_paths_contain_namespace() {
        let h = tempdir().unwrap();
        let c = cache_dir(h.path());
        assert!(c.to_string_lossy().contains("ai-workflow-skills"));
        assert_eq!(
            registry_cache_path(h.path()),
            c.join(crate::constants::REGISTRY_CACHE_FILENAME)
        );
    }

    #[test]
    fn clear_registry_removes_file() {
        let h = tempdir().unwrap();
        fs::create_dir_all(cache_dir(h.path())).unwrap();
        fs::write(registry_cache_path(h.path()), b"{}").unwrap();
        clear_registry_cache(h.path()).unwrap();
        assert!(!registry_cache_path(h.path()).exists());
    }

    #[test]
    fn clear_all_cache_when_missing_ok() {
        let h = tempdir().unwrap();
        clear_all_cache(h.path()).unwrap();
    }
}
