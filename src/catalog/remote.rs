//! Catálogo remoto: URL Git → clone/pull em cache sob `~/.cache/.../catalog-git/`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};

use crate::core::cache::cache_dir;

/// Variável opcional: token com `read_repository` / `read_api` para HTTPS privado (mesmo padrão que install.sh).
pub const ENV_AI_WORKFLOW_SKILLS_GIT_TOKEN: &str = "AI_WORKFLOW_SKILLS_GIT_TOKEN";

/// Detecta URL de repositório Git (não caminho local).
pub fn is_remote_catalog(raw: &str) -> bool {
    let s = raw.trim();
    s.starts_with("https://")
        || s.starts_with("http://")
        || s.starts_with("git@")
        || s.starts_with("ssh://")
}

fn cache_dir_for_url(home: &Path, url: &str) -> PathBuf {
    let mut h = Sha256::new();
    h.update(url.trim().as_bytes());
    let hex = format!("{:x}", h.finalize());
    let short = &hex[..16.min(hex.len())];
    cache_dir(home).join("catalog-git").join(short)
}

/// Injeta `oauth2:TOKEN@` em URLs HTTPS quando `AI_WORKFLOW_SKILLS_GIT_TOKEN` está definido.
fn https_url_with_optional_token(url: &str) -> String {
    let t = url.trim();
    if !t.starts_with("https://") {
        return t.to_string();
    }
    if let Ok(tok) = std::env::var(ENV_AI_WORKFLOW_SKILLS_GIT_TOKEN) {
        if !tok.is_empty() && !t.contains("@") {
            return format!("https://oauth2:{}@{}", tok, &t["https://".len()..]);
        }
    }
    t.to_string()
}

fn run_git(workdir: Option<&Path>, args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("git");
    if let Some(d) = workdir {
        cmd.current_dir(d);
    }
    cmd.args(args);
    let out = cmd
        .output()
        .with_context(|| format!("falha ao executar git {}", args.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        anyhow::bail!(
            "git {} falhou ({})\nstdout:\n{stdout}\nstderr:\n{stderr}",
            args.join(" "),
            out.status
        );
    }
    Ok(())
}

/// Garante clone atualizado em `dest` a partir de `url` (branch default, clone raso).
pub fn sync_git_catalog(home: &Path, url: &str) -> Result<PathBuf> {
    if !is_remote_catalog(url) {
        anyhow::bail!(
            "URL de catálogo inválida (esperado https://, http://, git@ ou ssh://): {url}"
        );
    }
    let effective = https_url_with_optional_token(url);
    let dest = cache_dir_for_url(home, url.trim());
    std::fs::create_dir_all(dest.parent().expect("catalog-git parent"))?;

    eprintln!(
        "A sincronizar catálogo remoto para cache: {}",
        dest.display()
    );

    if dest.join(".git").is_dir() {
        run_git(Some(&dest), &["remote", "set-url", "origin", &effective])?;
        run_git(Some(&dest), &["pull", "--ff-only"])?;
    } else {
        if dest.exists() {
            std::fs::remove_dir_all(&dest).with_context(|| {
                format!(
                    "não foi possível limpar cache de catálogo em {}",
                    dest.display()
                )
            })?;
        }
        let dest_str = dest
            .to_str()
            .ok_or_else(|| anyhow!("caminho de cache do catálogo não é UTF-8 válido"))?;
        run_git(None, &["clone", "--depth", "1", &effective, dest_str])?;
    }

    Ok(dest)
}

/// Resolve catálogo: diretório local (desenvolvimento) ou URL Git → cache.
pub fn resolve_remote_or_local(home: &Path, raw: &str) -> Result<(PathBuf, Option<String>)> {
    let t = raw.trim();
    if t.is_empty() {
        anyhow::bail!("caminho ou URL de catálogo vazio");
    }
    if is_remote_catalog(t) {
        let p = sync_git_catalog(home, t)?;
        return Ok((p, Some(t.to_string())));
    }
    Ok((PathBuf::from(t), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_remote_urls() {
        assert!(is_remote_catalog("https://github.com/g/p.git"));
        assert!(is_remote_catalog("git@github.com:a/b.git"));
        assert!(is_remote_catalog("  ssh://host/repo "));
        assert!(!is_remote_catalog("/tmp/catalog"));
        assert!(!is_remote_catalog("./skills"));
    }
}
