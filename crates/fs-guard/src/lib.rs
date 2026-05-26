use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct FsGuard {
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
        Self { allowed_roots }
    }

    pub fn resolve(&self, path: &str) -> Result<PathBuf, FsGuardError> {
        let path = Path::new(path);
        if path.is_absolute() {
            for root in &self.allowed_roots {
                if path.starts_with(root) {
                    return Ok(path.to_path_buf());
                }
            }
        } else {
            if let Some(root) = self.allowed_roots.first() {
                let resolved = root.join(path);
                return Ok(resolved.canonicalize().unwrap_or(resolved));
            }
        }
        warn!("Filesystem access denied for path: {}", path.display());
        Err(FsGuardError::Escaped(path.display().to_string()))
    }
}
