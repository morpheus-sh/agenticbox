//! Container runtime detection and connection abstraction.
//!
//! AgenticBox supports any OCI-compatible runtime that exposes a Docker Engine
//! API-compatible endpoint. Today that means Docker and Podman; the `bollard`
//! client speaks both because Podman implements the same REST API.
//!
//! The abstraction here is intentionally thin: a [`ContainerRuntime`] enum,
//! socket detection logic, and a single connect entry point. There is no trait
//! hierarchy — both runtimes use the same bollard client, just pointing at
//! different sockets. A trait becomes warranted only when a *fundamentally
//! different* runtime (containerd gRPC, WASM) is added.

#[cfg(unix)]
use std::path::PathBuf;

/// The OCI-compatible container runtime backing this AgenticBox instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerRuntime {
    /// Docker Engine / Docker Desktop / Rancher Desktop (Docker API).
    Docker,
    /// Podman — daemonless, Red Hat backed, Docker API-compatible.
    Podman,
}

impl ContainerRuntime {
    /// Human-readable name for error messages and logs.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Docker => "Docker",
            Self::Podman => "Podman",
        }
    }

    /// Standard socket locations for this runtime on Unix-like systems.
    ///
    /// Ordered by preference — the first existing socket wins during detection.
    #[cfg(unix)]
    fn unix_socket_candidates(&self) -> Vec<PathBuf> {
        match self {
            Self::Docker => {
                let mut sockets = vec![
                    // System-wide socket (rootful, most common on Linux/WSL).
                    PathBuf::from("/var/run/docker.sock"),
                ];
                // Docker Desktop on Linux without root stores the socket here.
                if let Some(home) = std::env::var_os("HOME") {
                    sockets.push(PathBuf::from(&home).join(".docker/run/docker.sock"));
                }
                sockets
            }
            Self::Podman => {
                let mut sockets = vec![
                    // Rootful Podman system socket.
                    PathBuf::from("/run/podman/podman.sock"),
                ];
                // Rootless Podman: $XDG_RUNTIME_DIR is set by systemd to
                // /run/user/$UID, so this covers the common case.
                if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
                    sockets.push(PathBuf::from(xdg).join("podman/podman.sock"));
                }
                sockets
            }
        }
    }

    /// All sockets this runtime *might* use, for error reporting.
    #[cfg(unix)]
    fn describe_candidate_sockets(&self) -> Vec<String> {
        self.unix_socket_candidates()
            .iter()
            .map(|p| p.display().to_string())
            .collect()
    }

    /// Probe the system for an available container runtime (Unix).
    ///
    /// Detection order:
    /// 1. `AGENTICBOX_CONTAINER_SOCKET` env var (explicit override)
    /// 2. `DOCKER_HOST` env var (standard Docker convention, bollard handles it)
    /// 3. Probe known Docker socket paths
    /// 4. Probe known Podman socket paths
    ///
    /// Returns the runtime variant and the socket/endpoint path to connect to.
    #[cfg(unix)]
    pub fn detect() -> anyhow::Result<(Self, String)> {
        use std::path::Path;

        // 1. Explicit override — highest priority.
        if let Ok(socket) = std::env::var("AGENTICBOX_CONTAINER_SOCKET") {
            let runtime = if socket.contains("podman") {
                Self::Podman
            } else {
                Self::Docker
            };
            tracing::info!(
                runtime = runtime.display_name(),
                socket = %socket,
                "Runtime via AGENTICBOX_CONTAINER_SOCKET override"
            );
            return Ok((runtime, socket));
        }

        // 2. DOCKER_HOST — let bollard handle TCP/unix/named-pipe URLs.
        //    Podman Machine often sets this, so we can't be sure which runtime
        //    it is. Default to Docker; the user can override above.
        if let Ok(host) = std::env::var("DOCKER_HOST") {
            tracing::info!("Runtime via DOCKER_HOST env var (assuming Docker)");
            return Ok((Self::Docker, host));
        }

        // 3-4. Probe sockets: Docker first (most common), then Podman.
        for runtime in [Self::Docker, Self::Podman] {
            for socket in runtime.unix_socket_candidates() {
                if Path::new(&socket).exists() {
                    let path = socket.to_string_lossy().to_string();
                    tracing::info!(
                        runtime = runtime.display_name(),
                        socket = %path,
                        "Runtime auto-detected"
                    );
                    return Ok((runtime, path));
                }
            }
        }

        // Nothing found — build a helpful error.
        let all_sockets: Vec<String> = [Self::Docker, Self::Podman]
            .iter()
            .flat_map(|r| r.describe_candidate_sockets())
            .collect();

        anyhow::bail!(
            "No container runtime found.\n\
             Checked sockets: {}\n\
             \n\
             To fix:\n  \
             • Start Docker Desktop / Rancher Desktop / `systemctl start docker`\n  \
             • Or start Podman: `systemctl --user start podman.socket`\n  \
             • Or set AGENTICBOX_CONTAINER_SOCKET=/path/to/socket",
            all_sockets.join(", ")
        )
    }

    /// Probe the system for an available container runtime (non-Unix).
    ///
    /// On Windows/macOS, bollard's `connect_with_local_defaults()` handles
    /// Docker Desktop's named pipe / socket automatically. Podman Machine on
    /// Windows exposes a Docker-compatible API that the user points at via
    /// `DOCKER_HOST` or `AGENTICBOX_CONTAINER_SOCKET`.
    #[cfg(not(unix))]
    pub fn detect() -> anyhow::Result<(Self, String)> {
        // Explicit override.
        if let Ok(socket) = std::env::var("AGENTICBOX_CONTAINER_SOCKET") {
            let runtime = if socket.contains("podman") {
                Self::Podman
            } else {
                Self::Docker
            };
            return Ok((runtime, socket));
        }

        // DOCKER_HOST (e.g. Podman Machine on Windows sets this).
        if let Ok(host) = std::env::var("DOCKER_HOST") {
            return Ok((Self::Docker, host));
        }

        // Assume Docker Desktop is available — bollard handles the connection.
        Ok((Self::Docker, String::from("default")))
    }
}
