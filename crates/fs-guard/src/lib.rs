use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct FsGuard {
    /// Allowed roots, lexically normalized (and canonicalized if they exist
    /// on disk) at construction time so every `resolve()` compares like forms.
    pub allowed_roots: Vec<PathBuf>,
}

#[derive(Debug, Error)]
pub enum FsGuardError {
    #[error("Path outside allowed roots: {0}")]
    Escaped(String),
    #[error("Permission denied: {0}")]
    Denied(String),
}

impl FsGuard {
    pub fn new(allowed_roots: Vec<PathBuf>) -> Self {
        let allowed_roots = allowed_roots
            .into_iter()
            .map(|r| best_effort_canonical(&normalize(&r)))
            .collect();
        Self { allowed_roots }
    }

    /// Resolve `path` to a canonical form and verify it stays within an
    /// allowed root.
    ///
    /// Two defences are layered:
    ///   1. **Lexical normalization** resolves `.` and `..` *without touching
    ///      the disk*, closing the prefix-matching traversal hole
    ///      (`/workspace/../etc/passwd`) for paths that don't exist yet.
    ///   2. **Canonicalization** of the deepest existing ancestor resolves
    ///      symlinks, so a symlink that points outside a root is rejected.
    pub fn resolve(&self, path: &str) -> Result<PathBuf, FsGuardError> {
        let raw = Path::new(path);
        let candidate = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            // Relative paths resolve against the first allowed root.
            match self.allowed_roots.first() {
                Some(root) => root.join(raw),
                None => {
                    warn!("Filesystem access denied (no allowed roots): {}", path);
                    return Err(FsGuardError::Escaped(path.to_string()));
                }
            }
        };

        let resolved = best_effort_canonical(&normalize(&candidate));

        for root in &self.allowed_roots {
            if resolved.starts_with(root) {
                return Ok(resolved);
            }
        }
        warn!("Filesystem access denied for path: {}", resolved.display());
        Err(FsGuardError::Escaped(resolved.display().to_string()))
    }
}

/// Lexically resolve `.` and `..` components without touching the filesystem.
/// Only `Normal` components are popped by `..` — prefixes and root dirs are
/// never removed, so `..` at the root simply stays at the root.
fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if matches!(result.components().next_back(), Some(Component::Normal(_))) {
                    result.pop();
                }
            }
            Component::CurDir => {} // skip "."
            other => result.push(other.as_os_str()),
        }
    }
    result
}

/// Return the most-canonical form of `path` available. If the full path
/// exists, canonicalize it entirely (resolves symlinks). Otherwise
/// canonicalize the deepest *existing* ancestor and re-append the tail. If
/// nothing exists, fall back to the (already normalized) input.
///
/// On Windows, `canonicalize` returns `\\?\`-prefixed verbatim paths. We
/// strip that prefix so canonical and lexical forms compare consistently —
/// otherwise a root canonicalized through its parent (`\\?\C:\workspace`)
/// would never `starts_with` a non-existent candidate (`C:\workspace\...`).
fn best_effort_canonical(path: &Path) -> PathBuf {
    if path.is_absolute() {
        if let Ok(c) = path.canonicalize() {
            return strip_verbatim(c);
        }
        if let Some(parent) = path.parent() {
            if let Ok(canon_parent) = parent.canonicalize() {
                if let Some(name) = path.file_name() {
                    return strip_verbatim(canon_parent).join(name);
                }
            }
        }
    }
    path.to_path_buf()
}

/// Strip the Windows verbatim (`\\?\`) prefix so canonicalized paths compare
/// equal to their lexical counterparts. No-op off Windows.
#[cfg(windows)]
fn strip_verbatim(path: PathBuf) -> PathBuf {
    if let Some(s) = path.as_os_str().to_str() {
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path
}

#[cfg(not(windows))]
fn strip_verbatim(path: PathBuf) -> PathBuf {
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    // Platform-aware paths: on Windows, forward-slash paths like /etc/passwd
    // are NOT truly absolute (no drive prefix), so they fall through to the
    // relative-path branch. Use real absolute paths for each platform.

    #[cfg(windows)]
    fn workspace() -> PathBuf {
        PathBuf::from("C:\\workspace")
    }
    #[cfg(windows)]
    fn tmp_agent() -> PathBuf {
        PathBuf::from("C:\\tmp\\agent")
    }
    #[cfg(windows)]
    fn blocked_path() -> &'static str {
        "C:\\Windows\\System32\\config\\SAM"
    }
    #[cfg(windows)]
    fn ssh_key() -> &'static str {
        "C:\\Users\\hacker\\.ssh\\id_rsa"
    }
    #[cfg(windows)]
    fn aws_cred() -> &'static str {
        "C:\\Users\\hacker\\.aws\\credentials"
    }
    #[cfg(windows)]
    fn outside_root() -> &'static str {
        "C:\\Users\\hacker\\stuff"
    }

    #[cfg(not(windows))]
    fn workspace() -> PathBuf {
        PathBuf::from("/workspace")
    }
    #[cfg(not(windows))]
    fn tmp_agent() -> PathBuf {
        PathBuf::from("/tmp/agent")
    }
    #[cfg(not(windows))]
    fn blocked_path() -> &'static str {
        "/etc/passwd"
    }
    #[cfg(not(windows))]
    fn ssh_key() -> &'static str {
        "/root/.ssh/id_rsa"
    }
    #[cfg(not(windows))]
    fn aws_cred() -> &'static str {
        "/root/.aws/credentials"
    }
    #[cfg(not(windows))]
    fn outside_root() -> &'static str {
        "/home/hacker/stuff"
    }

    fn guard() -> FsGuard {
        FsGuard::new(vec![workspace(), tmp_agent()])
    }

    // ── Allowed paths ──────────────────────────────────────

    #[test]
    fn resolve_absolute_path_within_root() {
        let g = guard();
        let ws = workspace();
        let test_file = ws.join("src/main.rs");
        let result = g.resolve(test_file.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_absolute_path_within_second_root() {
        let g = guard();
        let tmp = tmp_agent();
        let test_file = tmp.join("output.log");
        assert!(g.resolve(test_file.to_str().unwrap()).is_ok());
    }

    #[test]
    fn resolve_root_itself() {
        let g = guard();
        let ws = workspace();
        assert!(g.resolve(ws.to_str().unwrap()).is_ok());
    }

    // ── Blocked paths (escape attempts) ────────────────────

    #[test]
    fn block_path_outside_all_roots() {
        let g = guard();
        let result = g.resolve(blocked_path());
        assert!(result.is_err());
        match result.unwrap_err() {
            FsGuardError::Escaped(msg) => assert!(!msg.is_empty()),
            other => panic!("expected Escaped, got {:?}", other),
        }
    }

    #[test]
    fn block_ssh_key_access() {
        let g = guard();
        assert!(g.resolve(ssh_key()).is_err());
    }

    #[test]
    fn block_aws_credentials_access() {
        let g = guard();
        assert!(g.resolve(aws_cred()).is_err());
    }

    #[test]
    fn block_path_traversal_attempt() {
        // FIXED: lexical normalization resolves `..` before the prefix check,
        // so `/workspace/../etc/passwd` no longer matches the `/workspace`
        // root and is correctly rejected.
        let g = guard();
        let ws = workspace();
        let traversal = format!("{}/../etc/passwd", ws.display());
        let result = g.resolve(&traversal);
        assert!(result.is_err(), "path traversal must be blocked");
    }

    #[test]
    fn block_multi_level_traversal() {
        let g = guard();
        let ws = workspace();
        // three `..` escape two levels (a, b) plus the workspace root itself
        let traversal = format!("{}/a/b/../../../etc/passwd", ws.display());
        assert!(g.resolve(&traversal).is_err());
    }

    #[test]
    fn block_dotdot_at_root_stays_in_root_check() {
        // `..` past the root cannot escape it (no parent of root to match),
        // so `/workspace/../workspace/file` normalizes back under the root.
        let g = guard();
        let ws = workspace();
        let back = format!("{}/../workspace/file.rs", ws.display());
        assert!(g.resolve(&back).is_ok());
    }

    #[test]
    fn block_system_path() {
        let g = guard();
        let ws = workspace();
        let sys_path = ws.with_file_name("procsys"); // sibling of workspace, outside roots
        assert!(g.resolve(sys_path.to_str().unwrap()).is_err());
    }

    // ── Relative paths ─────────────────────────────────────

    #[test]
    fn resolve_relative_path_against_first_root() {
        let g = guard();
        let result = g.resolve("src/main.rs");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        let ws = workspace();
        assert!(resolved.starts_with(&ws));
    }

    #[test]
    fn block_relative_traversal() {
        let g = guard();
        // relative `..` escapes the first root
        assert!(g.resolve("../../etc/passwd").is_err());
    }

    // ── Edge cases ─────────────────────────────────────────

    #[test]
    fn empty_roots_block_absolute() {
        let g = FsGuard::new(vec![]);
        let ws = workspace();
        let f = ws.join("file.txt");
        assert!(g.resolve(f.to_str().unwrap()).is_err());
    }

    #[test]
    fn empty_roots_block_relative() {
        let g = FsGuard::new(vec![]);
        assert!(g.resolve("file.txt").is_err());
    }

    #[test]
    fn multiple_allowed_roots() {
        let ws = workspace();
        let data = ws.with_file_name("data");
        let app = ws.with_file_name("opt").join("app");
        let g = FsGuard::new(vec![ws.clone(), data.clone(), app.clone()]);

        assert!(g.resolve(ws.join("a").to_str().unwrap()).is_ok());
        assert!(g.resolve(data.join("b").to_str().unwrap()).is_ok());
        assert!(g.resolve(app.join("c").to_str().unwrap()).is_ok());
        assert!(g.resolve(outside_root()).is_err());
    }

    // ── normalize() unit checks ────────────────────────────

    #[test]
    fn normalize_resolves_dotdot() {
        let n = normalize(&PathBuf::from("/workspace/../etc/passwd"));
        assert_eq!(n, PathBuf::from("/etc/passwd"));
    }

    #[test]
    fn normalize_resolves_curdir() {
        let n = normalize(&PathBuf::from("/workspace/./src/./main.rs"));
        assert_eq!(n, PathBuf::from("/workspace/src/main.rs"));
    }
}
