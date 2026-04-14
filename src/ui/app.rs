//! Main application struct and event loop for the TUI.

use std::io::{self, stdout, IsTerminal, Write};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};

use crate::agents::{default_agents_for_install, detect_installed_agents};
use crate::catalog::files_and_hash;
use crate::cli::{install_with_progress_bar, load_merged_catalog_skills, ResolvedCatalog};
use crate::core::installer::remove_skills;
use crate::core::lockfile;
use crate::project_root::find_project_root;
use crate::types::{InstallMethod, InstallOptions, RemoveOptions, SkillInfo, SkillLockFile};

use super::state::{build_rows, AppState, PendingAction};
use super::widgets;

const TICK_RATE_MS: u64 = 250;

/// Result of handling a single key in the TUI (Enter is deferred to the event loop so we can redraw first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiControl {
    Continue,
    Quit,
    /// Run pending installs/updates/removals (may be a no-op if nothing pending).
    ApplyPending,
}

/// Main entry point for the TUI application.
pub fn run(
    home: &std::path::Path,
    cwd: &std::path::Path,
    catalogs: &[ResolvedCatalog],
) -> Result<()> {
    // Check if stdout is a terminal
    if !stdout().is_terminal() || !std::io::stdin().is_terminal() {
        anyhow::bail!("TTY mode requires a terminal");
    }

    // Load catalog and lockfiles
    let catalog_skills = load_merged_catalog_skills(catalogs)?;

    if catalog_skills.is_empty() {
        println!("Catalog is empty.");
        return Ok(());
    }

    let project_root = find_project_root(Some(cwd));
    let local_lock = lockfile::read_skill_lock(&project_root, home, false);
    let global_lock = lockfile::read_skill_lock(&project_root, home, true);

    // Initialize app state
    let mut app = AppState::new(catalog_skills, local_lock, global_lock)?;

    // Load catalog hashes
    app.load_catalog_hashes()?;

    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Run event loop
    let result = run_tui(&mut terminal, &mut app, home, cwd);

    // Cleanup terminal
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

/// Runs the TUI event loop.
fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    home: &std::path::Path,
    cwd: &std::path::Path,
) -> Result<()> {
    loop {
        // Draw
        terminal.draw(|frame| app.ui(frame))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(TICK_RATE_MS))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.handle_key_event(key.code, home, cwd)? {
                        TuiControl::Quit => break,
                        TuiControl::Continue => {}
                        TuiControl::ApplyPending => {
                            if app.has_any_pending() {
                                app.flash = "A trabalhar…".into();
                                terminal.draw(|f| app.ui(f))?;
                                let _ = stdout().flush();
                            }
                            match app.apply_pending(home, cwd) {
                                Ok(()) => {}
                                Err(e) => {
                                    app.flash = format!("Erro: {e:#}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

impl AppState {
    /// Creates a new AppState from catalog skills and lockfiles.
    pub fn new(
        catalog_skills: Vec<SkillInfo>,
        local_lock: SkillLockFile,
        global_lock: SkillLockFile,
    ) -> Result<Self> {
        let rows = build_rows(&catalog_skills, &local_lock, &global_lock);
        Ok(Self {
            catalog_skills,
            rows,
            local_lock,
            global_lock,
            cursor: 0,
            scroll: 0,
            flash: String::new(),
        })
    }

    /// Loads catalog hashes for all catalog skills.
    pub fn load_catalog_hashes(&mut self) -> Result<()> {
        for row in &mut self.rows {
            if let Some(ref skill) = row.catalog_skill {
                let (_, hash) = files_and_hash(&skill.path)?;
                row.catalog_hash = Some(hash);
            }
        }
        Ok(())
    }

    /// Renders the UI.
    pub fn ui(&self, frame: &mut Frame) {
        widgets::render(frame, &self.rows, self.cursor, self.scroll, &self.flash);
    }

    /// Handles a key event. Enter is returned as [`TuiControl::ApplyPending`] so the event loop can redraw before work runs.
    pub fn handle_key_event(
        &mut self,
        key: KeyCode,
        _home: &std::path::Path,
        _cwd: &std::path::Path,
    ) -> Result<TuiControl> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => Ok(TuiControl::Quit),
            KeyCode::Up => {
                self.move_cursor_up();
                Ok(TuiControl::Continue)
            }
            KeyCode::Down => {
                self.move_cursor_down();
                Ok(TuiControl::Continue)
            }
            KeyCode::Char('c') => {
                self.clear_pending();
                Ok(TuiControl::Continue)
            }
            KeyCode::Char('p') => {
                self.set_pending_install_project();
                Ok(TuiControl::Continue)
            }
            KeyCode::Char('g') => {
                self.set_pending_install_global();
                Ok(TuiControl::Continue)
            }
            KeyCode::Char('u') => {
                self.set_pending_update();
                Ok(TuiControl::Continue)
            }
            KeyCode::Char('r') => {
                self.set_pending_remove();
                Ok(TuiControl::Continue)
            }
            KeyCode::Enter => Ok(TuiControl::ApplyPending),
            _ => Ok(TuiControl::Continue),
        }
    }

    fn has_any_pending(&self) -> bool {
        self.rows.iter().any(|r| r.pending != PendingAction::None)
    }

    fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor + 1 < self.rows.len() {
            self.cursor += 1;
        }
    }

    fn clear_pending(&mut self) {
        if !self.rows.is_empty() {
            self.rows[self.cursor].pending = PendingAction::None;
        }
    }

    fn set_pending_install_project(&mut self) {
        if let Some(row) = self.rows.get(self.cursor) {
            if row.catalog_skill.is_some() {
                self.rows[self.cursor].pending = PendingAction::InstallProject;
            } else {
                self.flash = "fora do catálogo: só remover (r)".into();
            }
        }
    }

    fn set_pending_install_global(&mut self) {
        if let Some(row) = self.rows.get(self.cursor) {
            if row.catalog_skill.is_some() {
                self.rows[self.cursor].pending = PendingAction::InstallGlobal;
            } else {
                self.flash = "fora do catálogo: só remover (r)".into();
            }
        }
    }

    fn set_pending_update(&mut self) {
        let row = &self.rows[self.cursor];
        let local = row.has_local_update();
        let global = row.has_global_update();
        if local || global {
            self.rows[self.cursor].pending = PendingAction::Update;
        } else if row.local.is_some() || row.global.is_some() {
            self.flash = "já sincronizado".into();
        } else {
            self.flash = "não instalado".into();
        }
    }

    fn set_pending_remove(&mut self) {
        let row = &self.rows[self.cursor];
        if row.local.is_some() || row.global.is_some() {
            self.rows[self.cursor].pending = PendingAction::Remove;
        } else {
            self.flash = "nada para remover".into();
        }
    }

    fn apply_pending(&mut self, home: &std::path::Path, cwd: &std::path::Path) -> Result<()> {
        let work: Vec<(usize, PendingAction)> = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                if r.pending != PendingAction::None {
                    Some((i, r.pending))
                } else {
                    None
                }
            })
            .collect();

        if work.is_empty() {
            self.flash = "nada pendente".into();
            return Ok(());
        }

        let project_root = find_project_root(Some(cwd));

        // Do not redirect stdout here: `dup2` on FD 1 breaks the Ratatui/Crossterm backend after
        // the first install (symptom: freeze + only one skill). Core install/audit uses stderr.

        let mut replacement_msgs: Vec<String> = Vec::new();
        for (idx, pending) in &work {
            match pending {
                PendingAction::Update => {
                    let Some(skill) = self.rows[*idx].catalog_skill.as_ref() else {
                        continue;
                    };
                    let name = skill.name.clone();
                    let opts = InstallOptions {
                        global: false,
                        method: InstallMethod::Copy,
                        agents: vec!["cursor".to_string()],
                        skills: vec![name.clone()],
                        force_update: true,
                        audit_forced: false,
                    };
                    crate::core::installer::install_skills(
                        &project_root,
                        home,
                        std::slice::from_ref(skill),
                        &opts,
                        || (),
                    )?;
                }
                PendingAction::InstallProject | PendingAction::InstallGlobal => {
                    if let Some(msg) = self.apply_install(
                        *idx,
                        home,
                        cwd,
                        *pending == PendingAction::InstallGlobal,
                    )? {
                        replacement_msgs.push(msg);
                    }
                }
                PendingAction::Remove => {
                    self.apply_remove(*idx, home, &project_root)?;
                }
                PendingAction::None => {}
            }
        }

        for (idx, _) in &work {
            self.rows[*idx].pending = PendingAction::None;
        }

        self.flash = if !replacement_msgs.is_empty() {
            replacement_msgs.join(" | ")
        } else if work.len() == 1 {
            "Concluído.".into()
        } else {
            format!("Concluído ({} operações).", work.len())
        };

        self.refresh_locks(home, &project_root)?;

        Ok(())
    }

    /// Remove a instalação existente no âmbito (projeto ou global) antes de reinstalar a mesma skill.
    fn remove_skill_scope_before_install(
        &self,
        row_idx: usize,
        home: &std::path::Path,
        cwd: &std::path::Path,
        global: bool,
    ) -> std::io::Result<()> {
        let name = &self.rows[row_idx].name;
        let project_root = find_project_root(Some(cwd));

        let local_lock = lockfile::read_skill_lock(&project_root, home, false);
        let global_lock = lockfile::read_skill_lock(&project_root, home, true);

        let mut agents: std::collections::HashSet<String> = std::collections::HashSet::new();
        if global {
            if let Some(e) = global_lock.skills.get(name) {
                for ag in e.agents.clone().unwrap_or_default() {
                    agents.insert(ag);
                }
            }
        } else if let Some(e) = local_lock.skills.get(name) {
            for ag in e.agents.clone().unwrap_or_default() {
                agents.insert(ag);
            }
        }

        let agents: Vec<String> = if agents.is_empty() {
            vec!["cursor".into()]
        } else {
            agents.into_iter().collect()
        };

        let opts = RemoveOptions {
            global: Some(global),
            force: false,
        };

        let results = remove_skills(&project_root, home, name, &agents, &opts);
        let errs: Vec<String> = results
            .iter()
            .filter(|r| !r.success)
            .filter_map(|r| r.error.clone())
            .collect();
        if !errs.is_empty() {
            return Err(std::io::Error::other(format!(
                "falha ao remover skill antes de substituir: {}",
                errs.join("; ")
            )));
        }
        Ok(())
    }

    fn apply_remove(
        &self,
        row_idx: usize,
        home: &std::path::Path,
        project_root: &std::path::Path,
    ) -> std::io::Result<()> {
        let name = &self.rows[row_idx].name;

        // Collect agents from both locks
        let local_lock = lockfile::read_skill_lock(project_root, home, false);
        let global_lock = lockfile::read_skill_lock(project_root, home, true);

        let mut agents: std::collections::HashSet<String> = std::collections::HashSet::new();

        if let Some(e) = local_lock.skills.get(name) {
            for ag in e.agents.clone().unwrap_or_default() {
                agents.insert(ag);
            }
        }
        if let Some(e) = global_lock.skills.get(name) {
            for ag in e.agents.clone().unwrap_or_default() {
                agents.insert(ag);
            }
        }

        let agents: Vec<String> = if agents.is_empty() {
            vec!["cursor".into()]
        } else {
            agents.into_iter().collect()
        };

        let opts = RemoveOptions {
            global: None,
            force: false,
        };

        let results = remove_skills(project_root, home, name, &agents, &opts);
        if results.iter().any(|r| !r.success) {
            return Err(std::io::Error::other("falha ao remover skill"));
        }

        Ok(())
    }

    /// Installs the row's catalog skill. Returns `Some(msg)` if another catalog's entry
    /// for the same name was replaced, so the caller can surface that to the user.
    fn apply_install(
        &self,
        row_idx: usize,
        home: &std::path::Path,
        cwd: &std::path::Path,
        global: bool,
    ) -> std::io::Result<Option<String>> {
        let row = &self.rows[row_idx];
        // Mirror CLI `install` logic but do **not** use `dispatch_command`: it calls
        // `std::process::exit(1)` on partial failure, which skips TUI teardown (raw mode + alternate screen).
        let Some(skill) = row.catalog_skill.as_ref() else {
            return Err(std::io::Error::other("skill fora do catálogo"));
        };

        // Detect cross-catalog replacement by peeking at the raw lockfile, since
        // `row.local`/`row.global` are None when the owning row is from another catalog.
        let project_root = find_project_root(Some(cwd));
        let replacing_from = {
            let lock = if global {
                lockfile::read_skill_lock(&project_root, home, true)
            } else {
                lockfile::read_skill_lock(&project_root, home, false)
            };
            lock.skills
                .get(&skill.name)
                .and_then(|e| e.catalog_label.clone())
                .filter(|prev| prev.as_str() != skill.catalog_label.as_str())
        };

        // Cleanup before install is needed in two cases:
        // 1. Same-catalog reinstall (row.local/global already present).
        // 2. Cross-catalog replacement (row.local/global is None because join by
        //    catalog_label failed, but `replacing_from` detected a lock entry from
        //    another catalog). Without this, stale agent-dirs from the previous
        //    catalog can outlive the provenance flip when the new install targets
        //    fewer agents than the old one.
        let needs_cleanup = if global {
            row.global.is_some() || replacing_from.is_some()
        } else {
            row.local.is_some() || replacing_from.is_some()
        };
        if needs_cleanup {
            self.remove_skill_scope_before_install(row_idx, home, cwd, global)?;
        }

        let agents = {
            let detected = detect_installed_agents(home, &project_root);
            if detected.is_empty() {
                default_agents_for_install()
            } else {
                detected
            }
        };

        let opts = InstallOptions {
            global,
            method: InstallMethod::Copy,
            agents,
            skills: vec![skill.name.clone()],
            force_update: false,
            audit_forced: false,
        };

        let results =
            install_with_progress_bar(&project_root, home, std::slice::from_ref(skill), &opts)?;

        let errs: Vec<String> = results
            .iter()
            .filter(|r| !r.success)
            .filter_map(|r| r.error.clone())
            .collect();
        if !errs.is_empty() {
            return Err(std::io::Error::other(errs.join("; ")));
        }

        Ok(replacing_from.map(|prev| {
            format!(
                "substituído: {name} (catálogo {prev} → {novo})",
                name = skill.name,
                prev = prev,
                novo = skill.catalog_label
            )
        }))
    }

    fn refresh_locks(
        &mut self,
        home: &std::path::Path,
        project_root: &std::path::Path,
    ) -> Result<()> {
        self.local_lock = lockfile::read_skill_lock(project_root, home, false);
        self.global_lock = lockfile::read_skill_lock(project_root, home, true);

        // Update rows with new locks
        for row in &mut self.rows {
            row.local = self.local_lock.skills.get(&row.name).cloned();
            row.global = self.global_lock.skills.get(&row.name).cloned();
        }

        Ok(())
    }
}
