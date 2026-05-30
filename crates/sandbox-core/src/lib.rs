use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions};
use bollard::models::{HostConfig, MountTypeEnum};
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

pub use bollard::errors::Error as ContainerError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub image: String,
    pub cmd: Vec<String>,
    pub env: HashMap<String, String>,
    pub mounts: Vec<SandboxMount>,
    pub resources: SandboxResources,
    pub network_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxMount {
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxResources {
    pub memory_mb: Option<i64>,
    pub cpu_count: Option<i64>,
}

pub struct SandboxHandle {
    pub id: String,
    client: Docker,
}

impl SandboxHandle {
    pub fn new(id: String, client: Docker) -> Self {
        Self { id, client }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.client.start_container(&self.id, None::<StartContainerOptions<String>>).await?;
        Ok(())
    }

    pub async fn stop(&self, timeout: Option<i64>) -> anyhow::Result<()> {
        let opts = timeout.map(|t| StopContainerOptions { t });
        self.client.stop_container(&self.id, opts).await?;
        Ok(())
    }

    pub async fn remove(&self, force: bool) -> anyhow::Result<()> {
        let opts = RemoveContainerOptions { force, ..Default::default() };
        self.client.remove_container(&self.id, Some(opts)).await?;
        Ok(())
    }

    pub async fn logs(&self) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
}

pub struct SandboxManager {
    client: Docker,
}

impl SandboxManager {
    pub fn new() -> anyhow::Result<Self> {
        let client = Docker::connect_with_local_defaults()?;
        Ok(Self { client })
    }

    pub async fn create(&self, config: SandboxConfig) -> anyhow::Result<SandboxHandle> {
        let id = format!("sandbox-{}", Uuid::new_v4());
        let host_config = HostConfig {
            mounts: Some(config.mounts.iter().map(|m| bollard::models::Mount {
                target: Some(m.target.clone()),
                source: Some(m.source.clone()),
                read_only: Some(m.read_only),
                typ: Some(MountTypeEnum::BIND),
                ..Default::default()
            }).collect()),
            memory: config.resources.memory_mb.map(|m| m * 1024 * 1024),
            cpu_count: config.resources.cpu_count,
            network_mode: Some(config.network_mode.clone()),
            ..Default::default()
        };

        let container_config = Config {
            image: Some(config.image.clone()),
            cmd: Some(config.cmd.clone()),
            env: Some(config.env.iter().map(|(k, v)| format!("{}={}", k, v)).collect()),
            host_config: Some(host_config),
            ..Default::default()
        };

        self.client.create_container(
            Some(CreateContainerOptions { name: id.clone(), ..Default::default() }),
            container_config,
        ).await?;

        Ok(SandboxHandle::new(id, self.client.clone()))
    }
}
