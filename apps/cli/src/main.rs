use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use shared_types::{CreateSessionRequest, ModelConfig, PermissionSet, Session, SessionStatus};
use std::time::Duration;
use uuid::Uuid;

const DEFAULT_DAEMON_URL: &str = "http://127.0.0.1:8080";

#[derive(Parser)]
#[command(name = "agenticbox", version, about = "AgenticBox CLI - Deploy and manage AI agents locally")]
struct Cli {
    #[arg(long, short, default_value = DEFAULT_DAEMON_URL, global = true)]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy a new agent session
    Deploy {
        /// Agent name
        #[arg(long, short)]
        name: String,

        /// Model provider (openai, ollama, etc.)
        #[arg(long, default_value = "openai")]
        provider: String,

        /// Model name (gpt-4o, llama3, etc.)
        #[arg(long, default_value = "gpt-4o")]
        model: String,

        /// API key environment variable name (value will be read and sent to daemon)
        #[arg(long, default_value = "OPENAI_API_KEY")]
        api_key_env: String,

        /// Enable terminal access
        #[arg(long, default_value = "true")]
        terminal: bool,

        /// Filesystem permission: readonly, readwrite, none
        #[arg(long, default_value = "readwrite")]
        fs: String,

        /// Enable browser automation
        #[arg(long, default_value = "false")]
        browser: bool,

        /// Network policy: allowlist, localhost, offline, full
        #[arg(long, default_value = "allowlist")]
        network: String,

        /// Allowed domains (comma-separated, for allowlist)
        #[arg(long, default_value = "api.openai.com,github.com")]
        domains: String,

        /// Watch logs after deploy
        #[arg(long, short)]
        watch: bool,
    },

    /// List all sessions
    List {
        /// Output as JSON
        #[arg(long, short)]
        json: bool,
    },

    /// Get session details
    Get {
        /// Session ID
        id: Uuid,

        /// Output as JSON
        #[arg(long, short)]
        json: bool,
    },

    /// Stream logs for a session
    Logs {
        /// Session ID
        id: Uuid,

        /// Follow logs (like tail -f)
        #[arg(long, short)]
        follow: bool,
    },

    /// Stop a running session
    Stop {
        /// Session ID
        id: Uuid,
    },

    /// Remove a session
    Rm {
        /// Session ID
        id: Uuid,
    },

    /// Check daemon health
    Health,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionResponse {
    id: Uuid,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    model_config: ModelConfig,
    permissions: PermissionSet,
    status: SessionStatus,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let base = cli.url.trim_end_matches('/');

    match cli.command {
        Commands::Deploy {
            name,
            provider,
            model,
            api_key_env,
            terminal,
            fs,
            browser,
            network,
            domains,
            watch,
        } => {
            let model_config = ModelConfig {
                provider,
                model,
                api_key: Some(std::env::var(&api_key_env).unwrap_or_default()),
                base_url: None,
            };

            let filesystem = match fs.as_str() {
                "readonly" => shared_types::FsPermission::ReadOnly,
                "readwrite" => shared_types::FsPermission::ReadWrite,
                _ => shared_types::FsPermission::None,
            };

            let network_policy = match network.as_str() {
                "allowlist" => {
                    let domains_vec: Vec<String> = domains.split(',').map(|s| s.trim().to_string()).collect();
                    shared_types::NetworkPolicy::Allowlist(domains_vec)
                }
                "localhost" => shared_types::NetworkPolicy::LocalhostOnly,
                "offline" => shared_types::NetworkPolicy::Offline,
                "full" => shared_types::NetworkPolicy::Full,
                _ => shared_types::NetworkPolicy::Allowlist(vec![]),
            };

            let permissions = PermissionSet {
                terminal,
                filesystem,
                browser,
                network: network_policy,
            };

            let req = CreateSessionRequest {
                name: name.clone(),
                model_config,
                permissions,
            };

            println!("🚀 Deploying agent '{}'...", name);
            let resp = client
                .post(format!("{}/sessions", base))
                .json(&req)
                .send()
                .context("Failed to send deploy request")?;

            if !resp.status().is_success() {
                let err = resp.text().unwrap_or_default();
                anyhow::bail!("Deploy failed: {}", err);
            }

            let session: SessionResponse = resp.json().context("Failed to parse response")?;
            println!("✅ Agent deployed!");
            println!("   ID: {}", session.id);
            println!("   Status: {:?}", session.status);

            if watch {
                println!("\n📜 Streaming logs (Ctrl+C to stop)...");
                stream_logs(&client, base, session.id, true)?;
            } else {
                println!("\n💡 Run `agenticbox logs {} -f` to stream logs", session.id);
            }
        }

        Commands::List { json } => {
            let resp = client
                .get(format!("{}/sessions", base))
                .send()
                .context("Failed to list sessions")?;

            if !resp.status().is_success() {
                anyhow::bail!("List failed: {}", resp.text().unwrap_or_default());
            }

            let sessions: Vec<SessionResponse> = resp.json().context("Failed to parse response")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&sessions)?);
            } else if sessions.is_empty() {
                println!("No sessions found. Deploy one with `agenticbox deploy --name my-agent`");
            } else {
                println!("{:<36} {:<20} {:<15} {}", "ID", "NAME", "STATUS", "CREATED");
                println!("{}", "-".repeat(90));
                for s in sessions {
                    println!(
                        "{:<36} {:<20} {:<15} {}",
                        s.id,
                        truncate(&s.name, 20),
                        format!("{:?}", s.status),
                        s.created_at.format("%Y-%m-%d %H:%M")
                    );
                }
            }
        }

        Commands::Get { id, json } => {
            let resp = client
                .get(format!("{}/sessions/{}", base, id))
                .send()
                .context("Failed to get session")?;

            if !resp.status().is_success() {
                anyhow::bail!("Get failed: {}", resp.text().unwrap_or_default());
            }

            let session: SessionResponse = resp.json().context("Failed to parse response")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&session)?);
            } else {
                println!("Session Details");
                println!("===============");
                println!("ID:          {}", session.id);
                println!("Name:        {}", session.name);
                println!("Status:      {:?}", session.status);
                println!("Created:     {}", session.created_at);
                println!("Updated:     {}", session.updated_at);
                println!("Model:       {} ({})", session.model_config.model, session.model_config.provider);
                println!("Terminal:    {}", session.permissions.terminal);
                println!("Filesystem:  {:?}", session.permissions.filesystem);
                println!("Browser:     {}", session.permissions.browser);
                println!("Network:     {:?}", session.permissions.network);
            }
        }

        Commands::Logs { id, follow } => {
            stream_logs(&client, base, id, follow)?;
        }

        Commands::Stop { id } => {
            println!("🛑 Stopping session {}...", id);
            let resp = client
                .post(format!("{}/sessions/{}/status", base, id))
                .json(&serde_json::json!({ "status": "Stopped" }))
                .send()
                .context("Failed to stop session")?;

            if resp.status().is_success() {
                println!("✅ Session stopped");
            } else {
                anyhow::bail!("Stop failed: {}", resp.text().unwrap_or_default());
            }
        }

        Commands::Rm { id } => {
            println!("🗑️  Removing session {}...", id);
            // Note: This would need a DELETE endpoint on the daemon
            println!("⚠️  Not yet implemented (needs daemon DELETE endpoint)");
        }

        Commands::Health => {
            let resp = client.get(format!("{}/health", base)).send().context("Health check failed")?;
            if resp.status().is_success() {
                println!("✅ Daemon healthy at {}", base);
            } else {
                anyhow::bail!("Daemon unhealthy: {}", resp.text().unwrap_or_default());
            }
        }
    }

    Ok(())
}

fn stream_logs(client: &Client, base: &str, id: Uuid, follow: bool) -> Result<()> {
    // This is a placeholder - the daemon doesn't have a logs endpoint yet
    // In Phase 2, this would connect to a WebSocket or SSE endpoint
    println!("⚠️  Log streaming not yet implemented (needs Phase 2 log streaming)");
    println!("💡 For now, check daemon stdout/stderr or run with `RUST_LOG=debug`");
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}