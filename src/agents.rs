use std::path::{Path, PathBuf};

/// IDs suportados (mesmo conjunto base do agent-skills).
pub const AGENT_IDS: &[&str] = &[
    "cursor",
    "claude-code",
    "github-copilot",
    "windsurf",
    "cline",
    "aider",
    "codex",
    "gemini",
    "antigravity",
    "roo",
    "kilocode",
    "amazon-q",
    "augment",
    "tabnine",
    "opencode",
    "sourcegraph",
    "droid",
    "trae",
    "kiro",
];

pub struct AgentDef {
    pub id: &'static str,
    pub display_name: &'static str,
    pub skills_dir: &'static str,
    pub global_skills_dir: fn(&Path) -> PathBuf,
    pub detect: fn(home: &Path, project_root: &Path) -> bool,
}

pub fn all_defs() -> &'static [AgentDef] {
    &[
        AgentDef {
            id: "cursor",
            display_name: "Cursor",
            skills_dir: ".cursor/skills",
            global_skills_dir: |h| h.join(".cursor/skills"),
            detect: |h, pr| h.join(".cursor").exists() || pr.join(".cursor").exists(),
        },
        AgentDef {
            id: "claude-code",
            display_name: "Claude Code",
            skills_dir: ".claude/skills",
            global_skills_dir: |h| h.join(".claude/skills"),
            detect: |h, pr| h.join(".claude").exists() || pr.join(".claude").exists(),
        },
        AgentDef {
            id: "github-copilot",
            display_name: "GitHub Copilot",
            skills_dir: ".github/skills",
            global_skills_dir: |h| h.join(".copilot/skills"),
            detect: |h, pr| h.join(".copilot").exists() || pr.join(".github").exists(),
        },
        AgentDef {
            id: "windsurf",
            display_name: "Windsurf",
            skills_dir: ".windsurf/skills",
            global_skills_dir: |h| h.join(".codeium/windsurf/skills"),
            detect: |h, pr| h.join(".codeium/windsurf").exists() || pr.join(".windsurf").exists(),
        },
        AgentDef {
            id: "cline",
            display_name: "Cline",
            skills_dir: ".cline/skills",
            global_skills_dir: |h| h.join(".cline/skills"),
            detect: |h, pr| h.join(".cline").exists() || pr.join(".cline").exists(),
        },
        AgentDef {
            id: "aider",
            display_name: "Aider",
            skills_dir: ".aider/skills",
            global_skills_dir: |h| h.join(".aider/skills"),
            detect: |h, pr| h.join(".aider").exists() || pr.join(".aider").exists(),
        },
        AgentDef {
            id: "codex",
            display_name: "OpenAI Codex",
            skills_dir: ".codex/skills",
            global_skills_dir: |h| h.join(".codex/skills"),
            detect: |h, pr| h.join(".codex").exists() || pr.join(".codex").exists(),
        },
        AgentDef {
            id: "gemini",
            display_name: "Gemini CLI",
            skills_dir: ".gemini/skills",
            global_skills_dir: |h| h.join(".gemini/skills"),
            detect: |h, pr| h.join(".gemini").exists() || pr.join(".gemini").exists(),
        },
        AgentDef {
            id: "antigravity",
            display_name: "Antigravity",
            skills_dir: ".agent/skills",
            global_skills_dir: |h| h.join(".gemini/antigravity/skills"),
            detect: |h, pr| h.join(".gemini/antigravity").exists() || pr.join(".agent").exists(),
        },
        AgentDef {
            id: "roo",
            display_name: "Roo Code",
            skills_dir: ".roo/skills",
            global_skills_dir: |h| h.join(".roo/skills"),
            detect: |h, pr| h.join(".roo").exists() || pr.join(".roo").exists(),
        },
        AgentDef {
            id: "kilocode",
            display_name: "Kilo Code",
            skills_dir: ".kilocode/skills",
            global_skills_dir: |h| h.join(".kilocode/skills"),
            detect: |h, pr| h.join(".kilocode").exists() || pr.join(".kilocode").exists(),
        },
        AgentDef {
            id: "trae",
            display_name: "TRAE",
            skills_dir: ".trae/skills",
            global_skills_dir: |h| h.join(".trae/skills"),
            detect: |h, pr| h.join(".trae").exists() || pr.join(".trae").exists(),
        },
        AgentDef {
            id: "kiro",
            display_name: "Kiro",
            skills_dir: ".kiro/skills",
            global_skills_dir: |h| h.join(".kiro/skills"),
            detect: |h, pr| h.join(".kiro").exists() || pr.join(".kiro").exists(),
        },
        AgentDef {
            id: "amazon-q",
            display_name: "Amazon Q",
            skills_dir: ".amazonq/skills",
            global_skills_dir: |h| h.join(".amazonq/skills"),
            detect: |h, pr| h.join(".amazonq").exists() || pr.join(".amazonq").exists(),
        },
        AgentDef {
            id: "augment",
            display_name: "Augment",
            skills_dir: ".augment/skills",
            global_skills_dir: |h| h.join(".augment/skills"),
            detect: |h, pr| h.join(".augment").exists() || pr.join(".augment").exists(),
        },
        AgentDef {
            id: "tabnine",
            display_name: "Tabnine",
            skills_dir: ".tabnine/skills",
            global_skills_dir: |h| h.join(".tabnine/skills"),
            detect: |h, pr| h.join(".tabnine").exists() || pr.join(".tabnine").exists(),
        },
        AgentDef {
            id: "opencode",
            display_name: "OpenCode",
            skills_dir: ".opencode/skills",
            global_skills_dir: |h| h.join(".config/opencode/skills"),
            detect: |h, pr| {
                h.join(".config/opencode").exists()
                    || pr.join(".opencode").exists()
                    || pr.join(".config/opencode").exists()
            },
        },
        AgentDef {
            id: "sourcegraph",
            display_name: "Sourcegraph Cody",
            skills_dir: ".sourcegraph/skills",
            global_skills_dir: |h| h.join(".sourcegraph/skills"),
            detect: |h, pr| h.join(".sourcegraph").exists() || pr.join(".sourcegraph").exists(),
        },
        AgentDef {
            id: "droid",
            display_name: "Droid (Factory.ai)",
            skills_dir: ".factory/skills",
            global_skills_dir: |h| h.join(".factory/skills"),
            detect: |h, pr| h.join(".factory").exists() || pr.join(".factory").exists(),
        },
    ]
}

pub fn get_def(id: &str) -> Option<&'static AgentDef> {
    all_defs().iter().find(|d| d.id == id)
}

pub fn validate_agents(ids: &[String]) -> anyhow::Result<Vec<String>> {
    let mut out = Vec::new();
    for id in ids {
        let trimmed = id.trim();
        if !AGENT_IDS.contains(&trimmed) {
            anyhow::bail!("unknown agent: {trimmed}");
        }
        out.push(trimmed.to_string());
    }
    Ok(out)
}

/// Agentes usados quando nenhum `-a` foi passado e a detecção não achou pastas de agente.
///
/// **Cursor primeiro:** alinha com o [Agent Skills do Cursor](https://cursor.com/docs/skills)
/// (skills em `.cursor/skills/<skill>/` com `SKILL.md`). Outros agentes — ex. `claude-code`,
/// `windsurf` — exigem `-a`.
pub fn default_agents_for_install() -> Vec<String> {
    vec!["cursor".into()]
}

pub fn detect_installed_agents(home: &Path, project_root: &Path) -> Vec<String> {
    all_defs()
        .iter()
        .filter(|d| (d.detect)(home, project_root))
        .map(|d| d.id.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn validate_rejects_unknown_agent() {
        let e = validate_agents(&["not-a-real-agent".to_string()]).unwrap_err();
        assert!(e.to_string().contains("unknown agent"));
    }

    #[test]
    fn validate_accepts_known_ids() {
        let v = validate_agents(&["cursor".to_string(), " claude-code ".to_string()]).unwrap();
        assert_eq!(v, vec!["cursor", "claude-code"]);
    }

    #[test]
    fn get_def_lookup() {
        assert!(get_def("cursor").is_some());
        assert!(get_def("nope").is_none());
    }

    #[test]
    fn default_agents_is_cursor_only() {
        assert_eq!(default_agents_for_install(), vec!["cursor".to_string()]);
    }

    #[test]
    fn detect_cursor_when_project_has_dot_cursor() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().join("home");
        let pr = tmp.path().join("project");
        fs::create_dir_all(pr.join(".cursor")).unwrap();
        let ids = detect_installed_agents(&home, &pr);
        assert!(ids.contains(&"cursor".to_string()));
    }
}
