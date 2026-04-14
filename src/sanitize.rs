use std::path::{Path, PathBuf};

/// Sanitiza nome de skill para uso em paths (paridade com agent-skills).
pub fn sanitize_name(name: &str) -> String {
    let mut sanitized = name
        .replace(['/', '\\'], "")
        .chars()
        .filter(|c| !matches!(c, '\0' | '*' | ':' | '?' | '"' | '<' | '>' | '|'))
        .collect::<String>();
    sanitized = sanitized
        .trim_matches(|c: char| c == '.' || c.is_whitespace())
        .to_string();
    while sanitized.contains("..") {
        sanitized = sanitized.replace("..", "");
    }
    sanitized = sanitized.trim_start_matches('.').to_string();
    if sanitized.is_empty() {
        "unnamed-skill".to_string()
    } else {
        sanitized.chars().take(255).collect()
    }
}

/// Resolve o caminho lógico: canonicaliza o maior prefixo existente em disco e reaplica o sufixo.
/// Assim caminhos cujo último segmento ainda não existe (ex.: destino de instalação novo) podem
/// ser comparados com segurança, sem symlinks no trecho já existente.
fn resolve_logical_path(path: &Path) -> Option<PathBuf> {
    let mut cur = path;
    loop {
        match dunce::canonicalize(cur) {
            Ok(canonical) => {
                let suffix = path.strip_prefix(cur).unwrap_or_else(|_| Path::new(""));
                return Some(join_normalized(canonical, suffix));
            }
            Err(_) => cur = cur.parent()?,
        }
    }
}

fn join_normalized(mut base: PathBuf, ext: &Path) -> PathBuf {
    for c in ext.components() {
        match c {
            std::path::Component::Normal(s) => base.push(s),
            std::path::Component::ParentDir => {
                base.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
        }
    }
    base
}

/// Verifica se `target` permanece sob `base` após resolução.
pub fn is_path_safe(base: &Path, target: &Path) -> bool {
    match (resolve_logical_path(base), resolve_logical_path(target)) {
        (Some(b), Some(t)) => t.starts_with(&b) || t == b,
        _ => {
            // Caminhos inválidos ou FS sem prefixo canonicalizável: prefixo literal (sem symlinks)
            eprintln!("aviso: caminho não pode ser canonicalizado — verificação por prefixo usada");
            target.starts_with(base) || target == base
        }
    }
}

pub fn skill_install_path(base: &Path, skill_name: &str) -> PathBuf {
    base.join(sanitize_name(skill_name))
}

/// Slug estilo agent-skills (`toSlug`).
pub fn to_slug(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut out = String::new();
    let mut prev_hyphen = true;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_hyphen = false;
        } else if !prev_hyphen {
            out.push('-');
            prev_hyphen = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sanitize_empty_becomes_unnamed() {
        assert_eq!(sanitize_name("..."), "unnamed-skill");
    }

    /// Destino ainda não criado (paridade com instalação de skill nova): não deve depender do fallback nem avisar.
    #[test]
    fn path_safe_allows_nonexistent_leaf_under_base() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("proj").join(".cursor").join("skills");
        fs::create_dir_all(&base).unwrap();
        let target = base.join("nova-skill");
        assert!(target.parent().is_some());
        assert!(!target.exists());
        assert!(is_path_safe(&base, &target));
    }
}
