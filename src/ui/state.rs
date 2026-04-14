//! Application state types for the TUI.

use crate::types::{SkillInfo, SkillLockEntry, SkillLockFile};

/// Pending action for a skill row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    /// No pending action.
    None,
    /// Install to project level.
    InstallProject,
    /// Install globally.
    InstallGlobal,
    /// Update out-of-sync installations (project and/or global).
    Update,
    /// Remove installed skill.
    Remove,
}

impl PendingAction {
    /// Returns a short label for display.
    pub fn label(&self) -> &'static str {
        match self {
            PendingAction::None => "—",
            PendingAction::InstallProject => "[p] install (P)",
            PendingAction::InstallGlobal => "[g] install (G)",
            PendingAction::Update => "[u] update",
            PendingAction::Remove => "[r] remove",
        }
    }
}

/// A single row in the skill list, mirroring unified_skills_tui.rs Row.
#[derive(Debug, Clone)]
pub struct SkillRow {
    /// Catalog skill info, if present (None = orphaned).
    pub catalog_skill: Option<SkillInfo>,
    /// Skill name.
    pub name: String,
    /// Content hash from catalog.
    pub catalog_hash: Option<String>,
    /// Local lock entry, if installed locally.
    pub local: Option<SkillLockEntry>,
    /// Global lock entry, if installed globally.
    pub global: Option<SkillLockEntry>,
    /// Pending action.
    pub pending: PendingAction,
}

impl SkillRow {
    /// Returns true if the local installation is outdated.
    pub fn local_outdated(&self) -> bool {
        let Some(h) = &self.catalog_hash else {
            return false;
        };
        let Some(e) = &self.local else {
            return false;
        };
        e.content_hash.as_deref() != Some(h.as_str())
    }

    /// Returns true if the global installation is outdated.
    pub fn global_outdated(&self) -> bool {
        let Some(h) = &self.catalog_hash else {
            return false;
        };
        let Some(e) = &self.global else {
            return false;
        };
        e.content_hash.as_deref() != Some(h.as_str())
    }

    /// Returns true if there is a local update available.
    pub fn has_local_update(&self) -> bool {
        self.local.is_some() && self.local_outdated() && self.catalog_skill.is_some()
    }

    /// Returns true if there is a global update available.
    pub fn has_global_update(&self) -> bool {
        self.global.is_some() && self.global_outdated() && self.catalog_skill.is_some()
    }

    /// Returns true if there is any update available (project or global).
    pub fn has_any_update(&self) -> bool {
        self.has_local_update() || self.has_global_update()
    }

    /// Returns a human-readable description of what will be updated.
    pub fn update_scope_label(&self) -> &'static str {
        let local = self.has_local_update();
        let global = self.has_global_update();
        match (local, global) {
            (true, true) => "P + G",
            (true, false) => "P",
            (false, true) => "G",
            (false, false) => "—",
        }
    }

    /// Returns the label shown in the Pending column for the current pending action.
    pub fn pending_display_label(&self) -> String {
        match self.pending {
            PendingAction::None => "—".into(),
            PendingAction::InstallProject => {
                if self.local.is_some() {
                    "[p] substitui (P)".into()
                } else {
                    "[p] P".into()
                }
            }
            PendingAction::InstallGlobal => {
                if self.global.is_some() {
                    "[g] substitui (G)".into()
                } else {
                    "[g] G".into()
                }
            }
            PendingAction::Update => format!("[u] {}", self.update_scope_label()),
            PendingAction::Remove => "[r] remove".into(),
        }
    }

    /// Rótulo curto do catálogo para a tabela (skills órfãs: em branco).
    pub fn catalog_list_label(&self) -> &str {
        self.catalog_skill
            .as_ref()
            .map(|s| s.catalog_label.as_str())
            .unwrap_or("—")
    }

    /// Returns a short status string.
    pub fn status_compact(&self) -> String {
        let lp: &str = if self.local.is_some() {
            if self.local_outdated() {
                "P:!"
            } else {
                "P:OK"
            }
        } else {
            "P:—"
        };

        let gp: &str = if self.global.is_some() {
            if self.global_outdated() {
                "G:!"
            } else {
                "G:OK"
            }
        } else {
            "G:—"
        };

        format!("{lp} {gp}")
    }

    /// Returns true if this is an orphaned skill (not in catalog).
    pub fn is_orphan(&self) -> bool {
        self.catalog_skill.is_none()
    }
}

/// Builds rows joining catalog skills to lockfile entries by `(name, catalog_label)`.
///
/// Only one physical installation per skill name can exist on disk (see ARCHITECTURE.md),
/// so at most one catalog row "owns" a given lock entry. The matching rule is:
///
/// 1. A lock entry whose `catalog_label` equals the row's `catalog_skill.catalog_label`
///    attaches to that row.
/// 2. A legacy lock entry (`catalog_label = None`, pre-v3) attaches to the first row
///    with that name — provenance is unknown, so we grant a best-effort match and let
///    the next install record real provenance.
/// 3. Rows for the same name on other catalogs show as not installed; installing them
///    replaces the physical skill and flips provenance.
/// 4. Lock entries that no row claimed become orphan rows.
pub fn build_rows(
    catalog_skills: &[SkillInfo],
    local_lock: &SkillLockFile,
    global_lock: &SkillLockFile,
) -> Vec<SkillRow> {
    let mut rows: Vec<SkillRow> = Vec::new();
    let mut claimed_local: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut claimed_global: std::collections::HashSet<String> = std::collections::HashSet::new();

    for s in catalog_skills {
        let name = s.name.clone();
        let label = s.catalog_label.as_str();
        let local = claim_matching_entry(local_lock, &name, label, &mut claimed_local);
        let global = claim_matching_entry(global_lock, &name, label, &mut claimed_global);
        rows.push(SkillRow {
            catalog_skill: Some(s.clone()),
            name,
            catalog_hash: None,
            local,
            global,
            pending: PendingAction::None,
        });
    }

    let mut orphans: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for n in local_lock.skills.keys() {
        if !claimed_local.contains(n) {
            orphans.insert(n.clone());
        }
    }
    for n in global_lock.skills.keys() {
        if !claimed_global.contains(n) {
            orphans.insert(n.clone());
        }
    }

    for n in orphans {
        rows.push(SkillRow {
            catalog_skill: None,
            name: n.clone(),
            catalog_hash: None,
            local: local_lock.skills.get(&n).cloned(),
            global: global_lock.skills.get(&n).cloned(),
            pending: PendingAction::None,
        });
    }

    rows
}

/// Returns the lock entry for `name` that matches `row_label`:
/// - if an entry with `catalog_label == row_label` exists and is still unclaimed, claim it;
/// - else if a legacy entry (`catalog_label = None`) exists and is still unclaimed, claim it;
/// - else `None`.
fn claim_matching_entry(
    lock: &SkillLockFile,
    name: &str,
    row_label: &str,
    claimed: &mut std::collections::HashSet<String>,
) -> Option<SkillLockEntry> {
    let entry = lock.skills.get(name)?;
    if claimed.contains(name) {
        return None;
    }
    let matches = match entry.catalog_label.as_deref() {
        Some(label) => label == row_label,
        None => true, // legacy entry — best-effort match to first encountered row
    };
    if matches {
        claimed.insert(name.to_string());
        Some(entry.clone())
    } else {
        None
    }
}

/// Main application state for the TUI.
pub struct AppState {
    /// Catalog skills loaded at startup.
    pub catalog_skills: Vec<SkillInfo>,
    /// Skill rows to display.
    pub rows: Vec<SkillRow>,
    /// Local lockfile.
    pub local_lock: SkillLockFile,
    /// Global lockfile.
    pub global_lock: SkillLockFile,
    /// Current cursor position.
    pub cursor: usize,
    /// Scroll offset.
    pub scroll: usize,
    /// Flash message to display.
    pub flash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillInfo, SkillLockEntry, SkillLockFile, LOCKFILE_VERSION};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn skill(name: &str, label: &str) -> SkillInfo {
        SkillInfo {
            name: name.into(),
            catalog_label: label.into(),
            description: "d".into(),
            catalog_version: "1.0.0".into(),
            tags: vec![],
            path: PathBuf::from("/tmp").join(name),
            category: None,
        }
    }

    fn lock_entry(name: &str, catalog_label: Option<&str>) -> SkillLockEntry {
        SkillLockEntry {
            name: name.into(),
            source: "local".into(),
            content_hash: Some("h".into()),
            installed_at: "t0".into(),
            updated_at: "t0".into(),
            agents: Some(vec!["cursor".into()]),
            method: Some("copy".into()),
            global: Some(false),
            version: Some("1.0.0".into()),
            catalog_label: catalog_label.map(str::to_string),
        }
    }

    fn lockfile(entries: Vec<SkillLockEntry>) -> SkillLockFile {
        let mut m = HashMap::new();
        for e in entries {
            m.insert(e.name.clone(), e);
        }
        SkillLockFile {
            version: LOCKFILE_VERSION,
            skills: m,
        }
    }

    #[test]
    fn build_rows_unique_names_across_catalogs_each_gets_its_own_row() {
        let skills = vec![skill("a", "cat1"), skill("b", "cat2")];
        let local = lockfile(vec![lock_entry("a", Some("cat1"))]);
        let global = lockfile(vec![]);
        let rows = build_rows(&skills, &local, &global);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].local.is_some());
        assert!(rows[1].local.is_none());
    }

    #[test]
    fn build_rows_collision_attaches_lock_only_to_matching_catalog() {
        let skills = vec![skill("dup", "cat1"), skill("dup", "cat2")];
        let local = lockfile(vec![lock_entry("dup", Some("cat2"))]);
        let global = lockfile(vec![]);
        let rows = build_rows(&skills, &local, &global);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].local.is_none(), "cat1 row should not own lock");
        assert!(rows[1].local.is_some(), "cat2 row should own lock");
        assert_eq!(
            rows[1].local.as_ref().unwrap().catalog_label.as_deref(),
            Some("cat2")
        );
    }

    #[test]
    fn build_rows_legacy_none_label_claims_first_row() {
        let skills = vec![skill("dup", "cat1"), skill("dup", "cat2")];
        let local = lockfile(vec![lock_entry("dup", None)]);
        let global = lockfile(vec![]);
        let rows = build_rows(&skills, &local, &global);
        assert!(rows[0].local.is_some(), "first row claims legacy entry");
        assert!(rows[1].local.is_none(), "second row gets nothing");
    }

    #[test]
    fn build_rows_orphan_when_no_row_matches_label() {
        let skills = vec![skill("dup", "cat1")];
        let local = lockfile(vec![lock_entry("dup", Some("cat-gone"))]);
        let global = lockfile(vec![]);
        let rows = build_rows(&skills, &local, &global);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].local.is_none());
        assert!(rows[1].is_orphan());
        assert_eq!(rows[1].name, "dup");
    }
}
