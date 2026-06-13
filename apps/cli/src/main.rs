use anyhow::{Context, Result};
use clap::{Parser, Subcommand, Args};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use shared_types::{CreateSessionRequest, ModelConfig, PermissionSet, Session, SessionStatus};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

const DEFAULT_DAEMON_URL: &str = "http://127.0.0.1:8080";
const CONFIG_FILE_NAME: &str = "agenticbox.toml";

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
    /// Interactive configuration wizard
    Setup {
        /// Skip interactive prompts, use defaults/env
        #[arg(long)]
        non_interactive: bool,

        /// Reset configuration to defaults
        #[arg(long)]
        reset: bool,
    },

    /// Deploy a new agent session
    Deploy {
        /// Agent name
        #[arg(long, short)]
        name: String,

        /// Model provider (openai, anthropic, ollama, etc.)
        #[arg(long, default_value = "openai")]
        provider: String,

        /// Model name (gpt-4o, claude-3-5-sonnet, llama3, etc.)
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
        #[arg(long, default_value = "api.openai.com,api.anthropic.com,github.com")]
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

    /// Show current configuration
    Config {
        /// Show config file path only
        #[arg(long)]
        path: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    daemon_url: Option<String>,
    default_provider: Option<String>,
    default_model: Option<String>,
    providers: HashMap<String, ProviderConfig>,
    aliases: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProviderConfig {
    base_url: Option<String>,
    api_key_env: Option<String>,
    models: Vec<String>,
    default_model: Option<String>,
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

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("agenticbox")
        .join(CONFIG_FILE_NAME)
}

fn load_config() -> Result<Config> {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(Config::default())
    }
}

fn save_config(config: &Config) -> Result<()> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap())?;
    let content = toml::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

fn get_daemon_url(config: &Config, cli_url: &str) -> String {
    config.daemon_url.clone().unwrap_or_else(|| cli_url.to_string())
}

fn cmd_setup(non_interactive: bool, reset: bool) -> Result<()> {
    println!("{}", console::style("AgenticBox Setup").bold().green());
    println!("{}", console::style("─────────────────").dim());

    let mut config = if reset {
        Config::default()
    } else {
        load_config()?
    };

    if non_interactive {
        println!("Running in non-interactive mode. Using environment variables and defaults.");
        // Just ensure config file exists
        save_config(&config)?;
        println!("{} Config saved to {}", console::style("✓").green(), console::style(config_path().display()).cyan());
        return Ok(());
    }

    // Daemon URL
    println!("\n{} {}", console::style("1.").bold(), console::style("Daemon URL").bold());
    let current_url = config.daemon_url.as_deref().unwrap_or(DEFAULT_DAEMON_URL);
    let url = prompt_with_default("Daemon URL", current_url)?;
    config.daemon_url = Some(url.trim_end_matches('/').to_string());

    // Default provider
    println!("\n{} {}", console::style("2.").bold(), console::style("Default Provider").bold());
    let providers = vec!["openai", "anthropic", "ollama", "openrouter", "custom"];
    let current_provider = config.default_provider.as_deref().unwrap_or("openai");
    println!("Available: {}", providers.join(", "));
    let provider = prompt_with_default("Provider", current_provider)?;
    config.default_provider = Some(provider.clone());

    // Provider-specific config
    let provider_config = config.providers.entry(provider.clone()).or_insert(ProviderConfig {
        base_url: None,
        api_key_env: None,
        models: vec![],
        default_model: None,
    });

    // API key env var
    println!("\n{} {}", console::style("3.").bold(), console::style("API Key").bold());
    let default_key_env = match provider.as_str() {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        _ => "API_KEY",
    };
    let key_env = prompt_with_default("API key environment variable", default_key_env)?;
    provider_config.api_key_env = Some(key_env);

    // Base URL (for custom/ollama/openrouter)
    if ["ollama", "openrouter", "custom"].contains(&provider.as_str()) {
        println!("\n{} {}", console::style("4.").bold(), console::style("Base URL").bold());
        let default_base = match provider.as_str() {
            "ollama" => "http://localhost:11434/v1",
            "openrouter" => "https://openrouter.ai/api/v1",
            _ => "",
        };
        let base = prompt_with_default("Base URL (empty for default)", default_base)?;
        if !base.is_empty() {
            provider_config.base_url = Some(base);
        }
    }

    // Default model
    println!("\n{} {}", console::style("5.").bold(), console::style("Default Model").bold());
    let default_model = match provider.as_str() {
        "openai" => "gpt-4o",
        "anthropic" => "claude-3-5-sonnet-20241022",
        "ollama" => "llama3.1",
        "openrouter" => "anthropic/claude-3.5-sonnet",
        _ => "gpt-4o",
    };
    let current_model = config.default_model.as_deref().unwrap_or(default_model);
    let model = prompt_with_default("Model", current_model)?;
    config.default_model = Some(model.clone());
    provider_config.default_model = Some(model);
    provider_config.models = vec!["gpt-4o".into(), "gpt-4o-mini".into(), "gpt-4-turbo".into()]; // TODO: fetch dynamically

    // Save
    save_config(&config)?;
    println!("\n{} Configuration saved to {}", console::style("✓").green(), console::style(config_path().display()).cyan());

    // Validate
    println!("\n{} Validating configuration...", console::style("▶").cyan());
    validate_config(&config)?;

    println!("\n{} Setup complete!", console::style("✓").green().bold());
    println!("Next steps:");
    println!("  {} Start daemon:  {}", console::style("→").dim(), console::style("agenticbox daemon").cyan());
    println!("  {} Deploy agent:  {}", console::style("→").dim(), console::style("agenticbox deploy --name my-agent").cyan());

    Ok(())
}

fn prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    use std::io::{self, Write};
    print!("{} [{default}]: ", console::style(prompt).bold());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.is_empty() { Ok(default.to_string()) } else { Ok(input.to_string()) }
}

fn validate_config(config: &Config) -> Result<()> {
    let url = get_daemon_url(config, DEFAULT_DAEMON_URL);
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    match client.get(format!("{}/health", url)).send() {
        Ok(resp) if resp.status().is_success() => {
            println!("{} Daemon reachable at {}", console::style("✓").green(), console::style(&url).cyan());
        }
        Ok(_) => {
            println!("{} Daemon responded with error (is it running?)", console::style("⚠").yellow());
        }
        Err(e) => {
            println!("{} Could not reach daemon at {}: {}", console::style("⚠").yellow(), console::style(&url).cyan(), e);
            println!("  Start it with: {}", console::style("agenticbox daemon").cyan());
        }
    }

    // Check API key
    if let Some(provider) = &config.default_provider {
        if let Some(pconfig) = config.providers.get(provider) {
            if let Some(key_env) = &pconfig.api_key_env {
                match std::env::var(key_env) {
                    Ok(v) if !v.is_empty() => println!("{} {} is set", console::style("✓").green(), console::style(key_env).cyan()),
                    _ => println!("{} {} not set (set it before deploying)", console::style("⚠").yellow(), console::style(key_env).cyan()),
                }
            }
        }
    }

    Ok(())
}

fn cmd_config_show(path_only: bool) -> Result<()> {
    let path = config_path();
    if path_only {
        println!("{}", path.display());
        return Ok(());
    }

    if !path.exists() {
        println!("{} No config file found. Run {}", console::style("⚠").yellow(), console::style("agenticbox setup").cyan());
        return Ok(());
    }

    let config = load_config()?;
    println!("{} {}", console::style("Config:").bold(), console::style(path.display()).cyan());
    println!("{}", console::style("─────────────────").dim());

    println!("{} {}", console::style("Daemon URL:").bold(), config.daemon_url.as_deref().unwrap_or(DEFAULT_DAEMON_URL));
    println!("{} {}", console::style("Default Provider:").bold(), config.default_provider.as_deref().unwrap_or("openai"));
    println!("{} {}", console::style("Default Model:").bold(), config.default_model.as_deref().unwrap_or("gpt-4o"));

    if !config.providers.is_empty() {
        println!("\n{}", console::style("Providers:").bold());
        for (name, p) in &config.providers {
            println!("  {}:", console::style(name).cyan());
            if let Some(base) = &p.base_url { println!("    base_url: {}", base); }
            if let Some(key) = &p.api_key_env { println!("    api_key_env: {}", key); }
            if let Some(model) = &p.default_model { println!("    default_model: {}", model); }
        }
    }

    Ok(())
}

fn cmd_deploy(
    client: &Client,
    base: &str,
    name: String,
    provider: String,
    model: String,
    api_key_env: String,
    terminal: bool,
    fs: String,
    browser: bool,
    network: String,
    domains: String,
    watch: bool,
) -> Result<()> {
    let api_key = std::env::var(&api_key_env).unwrap_or_default();
    if api_key.is_empty() {
        anyhow::bail!("API key not found in environment variable '{}'. Run `agenticbox setup` first.", api_key_env);
    }

    let model_config = ModelConfig {
        provider,
        model,
        api_key: Some(api_key),
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

    println!("{} Deploying agent '{}'...", console::style("▶").cyan(), name);
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
    println!("{} Agent deployed!", console::style("✓").green());
    println!("   ID:     {}", session.id);
    println!("   Status: {:?}", session.status);

    if watch {
        println!("\n{} Streaming logs (Ctrl+C to stop)...", console::style("▶").cyan());
        stream_logs(client, base, session.id, true)?;
    } else {
        println!("\n{} Run `agenticbox logs {} -f` to stream logs", console::style("→").dim(), session.id);
    }

    Ok(())
}

fn cmd_list(client: &Client, base: &str, json: bool) -> Result<()> {
    let resp = client.get(format!("{}/sessions", base)).send().context("Failed to list sessions")?;

    if !resp.status().is_success() {
        anyhow::bail!("List failed: {}", resp.text().unwrap_or_default());
    }

    let sessions: Vec<SessionResponse> = resp.json().context("Failed to parse response")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
    } else if sessions.is_empty() {
        println!("{} No sessions found. Deploy one with `agenticbox deploy --name my-agent`", console::style("→").dim());
    } else {
        println!("{:<36} {:<20} {:<15} {}", "ID", "NAME", "STATUS", "CREATED");
        println!("{}", "─".repeat(90));
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
    Ok(())
}

fn cmd_get(client: &Client, base: &str, id: Uuid, json: bool) -> Result<()> {
    let resp = client.get(format!("{}/sessions/{}", base, id)).send().context("Failed to get session")?;

    if !resp.status().is_success() {
        anyhow::bail!("Get failed: {}", resp.text().unwrap_or_default());
    }

    let session: SessionResponse = resp.json().context("Failed to parse response")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&session)?);
    } else {
        println!("{}", console::style("Session Details").bold());
        println!("{}", console::style("───────────────").dim());
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
    Ok(())
}

fn cmd_logs(client: &Client, base: &str, id: Uuid, follow: bool) -> Result<()> {
    stream_logs(client, base, id, follow)
}

fn cmd_stop(client: &Client, base: &str, id: Uuid) -> Result<()> {
    println!("{} Stopping session {}...", console::style("▶").cyan(), id);
    let resp = client
        .post(format!("{}/sessions/{}/status", base, id))
        .json(&serde_json::json!({ "status": "Stopped" }))
        .send()
        .context("Failed to stop session")?;

    if resp.status().is_success() {
        println!("{} Session stopped", console::style("✓").green());
    } else {
        anyhow::bail!("Stop failed: {}", resp.text().unwrap_or_default());
    }
    Ok(())
}

fn cmd_health(client: &Client, base: &str) -> Result<()> {
    let resp = client.get(format!("{}/health", base)).send().context("Health check failed")?;
    if resp.status().is_success() {
        println!("{} Daemon healthy at {}", console::style("✓").green(), base);
    } else {
        anyhow::bail!("Daemon unhealthy: {}", resp.text().unwrap_or_default());
    }
    Ok(())
}

fn stream_logs(client: &Client, base: &str, id: Uuid, follow: bool) -> Result<()> {
    println!("{} Log streaming not yet implemented (needs Phase 2 log streaming)", console::style("⚠").yellow());
    println!("{} For now, check daemon stdout/stderr or run with `RUST_LOG=debug`", console::style("→").dim());
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let config = load_config().unwrap_or_default();

    match cli.command {
        Commands::Setup { non_interactive, reset } => {
            cmd_setup(non_interactive, reset)?
        }
        Commands::Config { path } => {
            cmd_config_show(path)?
        }
        Commands::Deploy { name, provider, model, api_key_env, terminal, fs, browser, network, domains, watch } => {
            let base = get_daemon_url(&config, &cli.url).trim_end_matches('/').to_string();
            cmd_deploy(&client, &base, name, provider, model, api_key_env, terminal, fs, browser, network, domains, watch)?
        }
        Commands::List { json } => {
            let base = get_daemon_url(&config, &cli.url).trim_end_matches('/').to_string();
            cmd_list(&client, &base, json)?
        }
        Commands::Get { id, json } => {
            let base = get_daemon_url(&config, &cli.url).trim_end_matches('/').to_string();
            cmd_get(&client, &base, id, json)?
        }
        Commands::Logs { id, follow } => {
            let base = get_daemon_url(&config, &cli.url).trim_end_matches('/').to_string();
            cmd_logs(&client, &base, id, follow)?
        }
        Commands::Stop { id } => {
            let base = get_daemon_url(&config, &cli.url).trim_end_matches('/').to_string();
            cmd_stop(&client, &base, id)?
        }
        Commands::Rm { id } => {
            println!("{} Not yet implemented (needs daemon DELETE endpoint)", console::style("⚠").yellow());
        }
        Commands::Health => {
            let base = get_daemon_url(&config, &cli.url).trim_end_matches('/').to_string();
            cmd_health(&client, &base)?
        }
    }
    Ok(())
}