use serde::{Deserialize, Serialize};
use shared_types::NetworkPolicy;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkGuard {
    pub policy: NetworkPolicy,
}

impl NetworkGuard {
    pub fn new(policy: NetworkPolicy) -> Self {
        Self { policy }
    }

    pub fn check(&self, destination: &str) -> Result<(), NetworkError> {
        info!("Checking network policy for {}", destination);
        match &self.policy {
            NetworkPolicy::Full => Ok(()),
            NetworkPolicy::LocalhostOnly => {
                if destination.contains("localhost") || destination.contains("127.0.0.1") {
                    Ok(())
                } else {
                    Err(NetworkError::Blocked(format!("{} not localhost", destination)))
                }
            }
            NetworkPolicy::Allowlist(domains) => {
                if domains.iter().any(|d| destination.contains(d)) {
                    Ok(())
                } else {
                    Err(NetworkError::Blocked(format!("{} not in allowlist", destination)))
                }
            }
            NetworkPolicy::Offline => Err(NetworkError::Blocked("Offline mode".into())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Network blocked: {0}")]
    Blocked(String),
}
