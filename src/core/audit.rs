use std::fs;
use std::path::PathBuf;

use crate::constants::{AUDIT_LOG_FILE, GLOBAL_CONFIG_DIR};
use crate::types::AuditEntry;

pub fn audit_log_path(home: &std::path::Path) -> PathBuf {
    home.join(GLOBAL_CONFIG_DIR).join(AUDIT_LOG_FILE)
}

pub fn log_audit(home: &std::path::Path, mut entry: AuditEntry) -> std::io::Result<()> {
    let dir = home.join(GLOBAL_CONFIG_DIR);
    fs::create_dir_all(&dir)?;
    let path = audit_log_path(home);
    entry.timestamp = Some(chrono::Utc::now().to_rfc3339());
    let line = format!("{}\n", serde_json::to_string(&entry)?);
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    f.write_all(line.as_bytes())?;
    Ok(())
}

pub fn read_audit_log(home: &std::path::Path, limit: Option<usize>) -> Vec<AuditEntry> {
    let path = audit_log_path(home);
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut skipped = 0usize;
    let mut entries: Vec<AuditEntry> = Vec::new();
    for line in content.lines().filter(|l| !l.is_empty()) {
        // First check: is this valid JSON at all?
        if serde_json::from_str::<serde_json::Value>(line).is_err() {
            skipped += 1;
            continue;
        }
        // Second check: does it match our schema?
        match serde_json::from_str::<AuditEntry>(line) {
            Ok(e) => entries.push(e),
            Err(e) => {
                skipped += 1;
                eprintln!(
                    "aviso: entrada de auditoria com esquema incompatível em {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }
    if skipped > 0 {
        eprintln!(
            "aviso: {skipped} linha(s) em {} com JSON inválido foram ignoradas.",
            path.display()
        );
    }
    entries.reverse();
    match limit {
        None => entries,
        Some(0) => Vec::new(),
        Some(n) => entries.into_iter().take(n).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AuditEntry;
    use tempfile::tempdir;

    #[test]
    fn log_and_read_latest() {
        let home = tempdir().unwrap();
        log_audit(
            home.path(),
            AuditEntry {
                action: "install".into(),
                skill_name: "demo".into(),
                agents: vec![],
                success: 1,
                failed: 0,
                forced: None,
                details: None,
                timestamp: None,
            },
        )
        .unwrap();
        let entries = read_audit_log(home.path(), Some(1));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "install");
        assert_eq!(entries[0].skill_name, "demo");
        assert!(entries[0].timestamp.is_some());
    }

    #[test]
    fn read_empty_limit_zero() {
        let home = tempdir().unwrap();
        assert!(read_audit_log(home.path(), Some(0)).is_empty());
    }
}
