use std::path::{Path, PathBuf};

const MARKERS: &[&str] = &["package.json", ".git"];

/// Sobe diretórios até encontrar marcador de projeto (paridade com agent-skills).
pub fn find_project_root(start: Option<&Path>) -> PathBuf {
    let fallback = start
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut current = dunce::canonicalize(&fallback).unwrap_or(fallback.clone());
    let root_component = current
        .ancestors()
        .last()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/"));

    while current != root_component {
        for m in MARKERS {
            if current.join(m).exists() {
                return current;
            }
        }
        if let Some(p) = current.parent() {
            current = p.to_path_buf();
        } else {
            break;
        }
    }

    fallback
}
