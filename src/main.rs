//! P2P AI Agents: Node initialization and lifecycle management
//!
//! This binary provides the main entry point for running a P2P AI agent node.
//! It handles initialization, configuration, identity management, and the main event loop.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use p2p_ai_agents::agent::identity::AgentIdentity;
use p2p_ai_agents::core::{
    config::Config,
    identity::{load_or_create_identity, NodeIdentityData},
    logging::{init_logging, LogFormat, LoggingConfig},
    metadata::version_display,
};
use p2p_ai_agents::project_manager::ProjectManager;
use semaphore::Field;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(name = "p2p-ai-agents")]
#[command(about = "A distributed peer-to-peer network of AI agents")]
#[command(disable_version_flag = true)]
struct Cli {
    /// Display version information and exit
    #[arg(short = 'V', long)]
    version: bool,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,

    /// Port to listen on (overrides config)
    #[arg(short, long, global = true)]
    port: Option<u16>,

    /// Bootstrap nodes (comma-separated, overrides config)
    #[arg(short, long, global = true)]
    bootstrap: Option<String>,

    /// Maximum number of peers (overrides config)
    #[arg(short, long, global = true)]
    max_peers: Option<usize>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    /// Log format (json or text)
    #[arg(long, global = true, default_value = "text")]
    log_format: String,

    /// Enable verbose startup diagnostics
    #[arg(long, global = true)]
    startup_diagnostics: bool,

    /// Run as daemon (Unix only)
    #[arg(long, global = true)]
    daemon: bool,

    /// PID file path (for daemon mode)
    #[arg(long, global = true)]
    pid_file: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Initialize a new node (generate identity)
    Init {
        /// Force regeneration of identity (destroys existing identity!)
        #[arg(long)]
        force: bool,
    },

    /// Display node identity and status
    Status,

    /// Start the node daemon
    Start,

    /// Stop the running node daemon
    Stop,

    /// List connected peers
    Peers,

    /// Monitor node status continuously
    Monitor {
        /// Refresh interval in seconds
        #[arg(long, default_value = "2")]
        interval: u64,
    },

    /// Run a project from a master workflow file
    RunProject {
        /// Path to the master workflow YAML file
        #[arg(long)]
        workflow: PathBuf,

        /// High-level goal for the project
        #[arg(long)]
        goal: String,
    },

    /// Display node ID
    NodeId,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --version flag early
    if cli.version {
        print!("{}", version_display());
        return Ok(());
    }

    // Initialize logging ONLY if NOT daemon mode (or if we are not about to daemonize)
    // Actually, we want to log the "Starting..." message.
    // But we can't re-init logging in the child if it's already init in parent.
    // So we delay logging init until we know if we are daemonizing.

    // Check command early
    let is_daemon_start = match &cli.command {
        Some(Commands::Start) | None => cli.daemon,
        _ => false,
    };

    if !is_daemon_start {
        let log_format = match cli.log_format.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            "pretty" => LogFormat::Pretty,
            _ => LogFormat::Compact,
        };

        let logging_config = LoggingConfig {
            level: cli.log_level.to_uppercase(),
            format: log_format,
            ..Default::default()
        };

        init_logging(&logging_config).context("Failed to initialize logging")?;
        info!("Starting P2P AI Agent Node");
    }

    // Create runtime for non-daemon commands or foreground start
    // We only need this if NOT daemonizing
    let rt = if !is_daemon_start {
        Some(tokio::runtime::Runtime::new().context("Failed to create Tokio runtime")?)
    } else {
        None
    };

    // Handle commands
    match cli.command {
        Some(Commands::Init { force }) => {
            rt.as_ref().unwrap().block_on(cmd_init(force))?;
        }
        Some(Commands::Status) => {
            rt.as_ref().unwrap().block_on(cmd_status(&cli))?;
        }
        Some(Commands::NodeId) => {
            rt.as_ref().unwrap().block_on(cmd_node_id())?;
        }
        Some(Commands::Start) | None => {
            // Check for daemon mode
            if cli.daemon {
                #[cfg(unix)]
                {
                    cmd_start_daemon(&cli)?;
                }
                #[cfg(not(unix))]
                {
                    anyhow::bail!("Daemon mode is not supported on Windows");
                }
            } else {
                rt.as_ref().unwrap().block_on(cmd_start(&cli))?;
            }
        }
        Some(Commands::Stop) => {
            rt.as_ref().unwrap().block_on(cmd_stop(&cli))?;
        }
        Some(Commands::Peers) => {
            rt.as_ref().unwrap().block_on(cmd_peers())?;
        }
        Some(Commands::Monitor { interval }) => {
            rt.as_ref().unwrap().block_on(cmd_monitor(interval))?;
        }
        Some(Commands::RunProject { workflow, goal }) => {
            rt.as_ref()
                .unwrap()
                .block_on(cmd_run_project(&workflow, &goal))?;
        }
    }

    Ok(())
}

/// Start the node in daemon mode (Unix only)
#[cfg(unix)]
fn cmd_start_daemon(cli: &Cli) -> Result<()> {
    use p2p_ai_agents::daemon::{daemonize, default_log_path, PidFileManager};

    info!("🔧 Configuring daemon mode...");

    // Setup PID file manager
    let pid_file_path = if let Some(path) = &cli.pid_file {
        PathBuf::from(path)
    } else {
        PidFileManager::default_path().context("Failed to get default PID file path")?
    };

    let pid_manager = PidFileManager::new(pid_file_path);

    // Check if already running
    if let Some(existing_pid) = pid_manager.check_running()? {
        anyhow::bail!(
            "Another instance is already running with PID: {}. \
             PID file: {}",
            existing_pid,
            pid_manager.path().display()
        );
    }

    // Get log file path
    let log_path = default_log_path().context("Failed to get default log path")?;

    info!("📝 Log file: {}", log_path.display());
    info!("📋 PID file: {}", pid_manager.path().display());
    info!("🔀 Forking to background...");

    // Daemonize the process (this will fork and exit the parent)
    daemonize(log_path.clone()).context("Failed to daemonize process")?;

    // --- WE ARE NOW IN THE DAEMON PROCESS ---

    // After daemonization, re-initialize logging
    let log_format = match cli.log_format.to_lowercase().as_str() {
        "json" => LogFormat::Json,
        "pretty" => LogFormat::Pretty,
        _ => LogFormat::Compact,
    };

    let logging_config = LoggingConfig {
        level: cli.log_level.to_uppercase(),
        format: log_format,
        ..Default::default()
    };

    // Re-init logging for the new process (redirected to file)
    init_logging(&logging_config).context("Failed to re-initialize logging after daemonization")?;

    info!("✅ Daemon process started");
    info!("📝 Logs: {}", log_path.display());

    // Write PID file
    pid_manager.write().context("Failed to write PID file")?;

    // Create a NEW runtime for the daemon process
    let runtime = tokio::runtime::Runtime::new().context("Failed to create daemon runtime")?;

    // Run the node
    let result = runtime.block_on(async { run_node(cli).await });

    // Clean up PID file on exit

    if let Err(e) = pid_manager.remove() {
        tracing::warn!("Failed to remove PID file: {}", e);
    }

    result
}

/// Initialize node identity
async fn cmd_init(force: bool) -> Result<()> {
    use p2p_ai_agents::core::identity::storage::default_identity_path;

    let identity_path = default_identity_path();

    if identity_path.exists() && !force {
        info!("✅ Identity already exists at: {}", identity_path.display());
        info!("   Use --force to regenerate (WARNING: destroys existing identity!)");

        // Load and display existing identity
        let identity = load_or_create_identity()
            .await
            .context("Failed to load identity")?;
        display_identity(&identity)?;
        return Ok(());
    }

    if force && identity_path.exists() {
        info!("⚠️  Force flag set - backing up existing identity");
        let backup_path = identity_path.with_extension("json.bak");
        tokio::fs::rename(&identity_path, &backup_path)
            .await
            .context("Failed to backup existing identity")?;
        info!("   Existing identity moved to: {}", backup_path.display());
    }

    info!("🔑 Generating new node identity...");
    let identity = load_or_create_identity()
        .await
        .context("Failed to create identity")?;

    info!("✅ Identity created successfully!");
    info!("   Location: {}", identity_path.display());
    display_identity(&identity)?;

    Ok(())
}

use p2p_ai_agents::application::status::NodeStatus;

/// Display node status
async fn cmd_status(cli: &Cli) -> Result<()> {
    info!("📊 Node Status");
    info!("════════════════");

    // Try to load dynamic status
    // Use Config to resolve the correct path
    let status_path = match Config::load().await {
        Ok(config) => config.storage_path.join("node_status.json"),
        Err(_) => std::path::PathBuf::from("data/node_status.json"),
    };

    if status_path.exists() {
        match tokio::fs::read_to_string(&status_path).await {
            Ok(content) => match serde_json::from_str::<NodeStatus>(&content) {
                Ok(status) => {
                    info!("Node ID:         {}", status.node_id);
                    info!("State:           {}", status.state);
                    info!("Version:         {}", status.version);
                    info!("Uptime:          {}s", status.uptime_seconds);
                    info!("Peers:           {}", status.connected_peers);
                    info!("Agents:          {}", status.active_agents);
                    info!("Tasks Processed: {}", status.tasks_processed);
                    info!(
                        "Memory:          {:.2} MB / {:.2} MB",
                        status.memory_usage_bytes as f64 / 1024.0 / 1024.0,
                        status.total_memory_bytes as f64 / 1024.0 / 1024.0
                    );
                    info!("CPU Usage:       {:.1}%", status.cpu_usage_percent);
                    info!("Last Update:     {}", status.timestamp.to_rfc3339());
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Failed to parse status file: {}", e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read status file: {}", e);
            }
        }
    } else {
        info!("⚠️  Node is not running or status not available.");
        info!("   (Status file not found at {})", status_path.display());
    }

    // Fallback to static info if dynamic status is not available
    info!("════════════════");
    info!("Static Configuration:");

    // Load identity
    match load_or_create_identity().await {
        Ok(identity) => {
            display_identity(&identity)?;
        }
        Err(e) => {
            info!("  Identity: Not found or error loading ({})", e);
        }
    }

    // Load configuration
    match Config::load().await {
        Ok(_config) => {
            info!("");
            info!("Configuration:");
            info!("  Listen Port: {}", cli.port.unwrap_or(9000));
            info!("  Max Peers: {}", cli.max_peers.unwrap_or(32));

            if let Some(bootstrap) = &cli.bootstrap {
                info!("  Bootstrap Nodes: {}", bootstrap);
            }
        }
        Err(e) => {
            info!("  Configuration: Error loading ({})", e);
        }
    }

    Ok(())
}

/// List connected peers
async fn cmd_peers() -> Result<()> {
    // Try to load dynamic status
    // Use Config to resolve the correct path
    let status_path = match Config::load().await {
        Ok(config) => config.storage_path.join("node_status.json"),
        Err(_) => std::path::PathBuf::from("data/node_status.json"),
    };

    if status_path.exists() {
        match tokio::fs::read_to_string(&status_path).await {
            Ok(content) => match serde_json::from_str::<NodeStatus>(&content) {
                Ok(status) => {
                    info!("Connected Peers ({})", status.connected_peers);
                    info!("════════════════");
                    if status.peers.is_empty() {
                        info!("No peers connected.");
                    } else {
                        for peer in status.peers {
                            info!("- {}", peer);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse status file: {}", e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read status file: {}", e);
            }
        }
    } else {
        info!("⚠️  Node is not running or status not available.");
        info!("   (Status file not found at {})", status_path.display());
    }
    Ok(())
}

/// Monitor node status
async fn cmd_monitor(interval: u64) -> Result<()> {
    info!("Starting monitor (Ctrl+C to stop)...");

    loop {
        // Clear screen (ANSI escape code)
        print!("\x1B[2J\x1B[1;1H");

        let cli = Cli::parse(); // Re-parse just to pass to cmd_status, inefficient but simple
        cmd_status(&cli).await?;

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

/// Display just the node ID
async fn cmd_node_id() -> Result<()> {
    let identity = load_or_create_identity()
        .await
        .context("Failed to load identity")?;

    let node_id = identity
        .derive_node_id()
        .context("Failed to derive node ID")?;

    println!("{}", node_id);
    Ok(())
}

/// Start the node daemon
async fn cmd_start(cli: &Cli) -> Result<()> {
    info!("🚀 Starting P2P AI Agent Node");

    // Daemon mode is now handled in main() before runtime creation
    // This function is only for foreground execution

    // Run in foreground
    run_node(cli).await?;
    Ok(())
}

/// Stop the node daemon
async fn cmd_stop(cli: &Cli) -> Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        use p2p_ai_agents::daemon::PidFileManager;
        use std::time::Duration;

        let pid_file_path = if let Some(path) = &cli.pid_file {
            PathBuf::from(path)
        } else {
            PidFileManager::default_path().context("Failed to get default PID file path")?
        };

        let pid_manager = PidFileManager::new(pid_file_path.clone());

        match pid_manager.check_running()? {
            Some(pid) => {
                info!("Stopping node with PID: {}", pid);

                // Send SIGTERM
                kill(Pid::from_raw(pid), Signal::SIGTERM)
                    .context("Failed to send SIGTERM to process")?;

                // Wait a bit for it to exit (simple check)
                let mut retries = 0;
                while retries < 20 {
                    // Wait up to 2 seconds
                    if pid_manager.check_running()?.is_none() {
                        info!("✅ Node stopped successfully.");
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    retries += 1;
                }

                // If still running, warn user
                if pid_manager.check_running()?.is_some() {
                    tracing::warn!("Node did not stop gracefully within 2 seconds. You may need to use 'kill -9 {}'.", pid);
                }
            }
            None => {
                info!("⚠️  No running node found (PID file missing or stale).");
                info!("   Checked path: {}", pid_file_path.display());
            }
        }
        Ok(())
    }

    #[cfg(not(unix))]
    {
        anyhow::bail!("Daemon commands are not supported on Windows.");
    }
}

/// Run a project from a master workflow
async fn cmd_run_project(workflow_path: &Path, goal: &str) -> Result<()> {
    info!("🚀 Starting Autonomous Project Manager");
    info!("   Workflow: {}", workflow_path.display());
    info!("   Goal: {}", goal);

    let project_manager = ProjectManager::new(workflow_path.to_path_buf(), goal.to_string())
        .await
        .context("Failed to initialize Project Manager")?;

    project_manager
        .run()
        .await
        .context("Project execution failed")?;

    info!("✅ Project completed successfully!");
    Ok(())
}

/// Run the node (common logic for daemon and foreground modes)
async fn run_node(cli: &Cli) -> Result<()> {
    // Ensure default config file exists
    match Config::save_default_if_missing().await {
        Ok(config_path) => {
            info!("📄 Config file: {}", config_path.display());
        }
        Err(e) => {
            info!("⚠️  Could not create default config file: {}", e);
        }
    }

    // Load configuration
    let mut config = Config::load()
        .await
        .context("Failed to load configuration")?;

    // Apply CLI overrides
    if let Some(port) = cli.port {
        info!("Overriding port from CLI: {}", port);
        config.listen_port = port;
    }

    if let Some(bootstrap) = &cli.bootstrap {
        info!("Overriding bootstrap nodes from CLI: {}", bootstrap);
        config.bootstrap_nodes = bootstrap.split(',').map(|s| s.to_string()).collect();
    }

    if let Some(max_peers) = cli.max_peers {
        info!("Overriding max_peers from CLI: {}", max_peers);
        config.max_peers = max_peers;
    }

    // Validate configuration after all overrides
    config
        .validate()
        .context("Configuration validation failed")?;

    info!("✅ Configuration validated successfully");

    // Initialize or load node identity
    info!("🔑 Loading node identity...");
    let identity = load_or_create_identity()
        .await
        .context("Failed to load or create identity")?;

    let node_id = identity
        .derive_node_id()
        .context("Failed to derive node ID")?;

    info!("✅ Node ID: {}", node_id);
    info!(
        "📡 Public Key: {}...{}",
        &identity.public_key_hex[..8],
        &identity.public_key_hex[identity.public_key_hex.len() - 8..]
    );

    // Initialize the AgentIdentity (Trust Registry + Manager)
    info!("🛡️  Initializing Agent Identity & Trust Registry...");
    // 20 is the standard Semaphore depth we confirmed in testing
    let agent_identity = AgentIdentity::new(20, Field::from(0))
        .await
        .context("Failed to initialize AgentIdentity")?;

    // Create local DID if needed (mock for now)
    let my_did = agent_identity.create_my_did().await?;
    info!("🆔 Agent DID: {}", my_did);

    // Run event loop
    info!("🌐 Node ready - starting event loop");
    info!("   Press Ctrl+C to stop");

    run_event_loop(&config, &identity).await?;

    Ok(())
}

/// Display identity information
fn display_identity(identity: &NodeIdentityData) -> Result<()> {
    let node_id = identity
        .derive_node_id()
        .context("Failed to derive node ID")?;

    info!("");
    info!("Identity:");
    info!("  Node ID: {}", node_id);
    info!(
        "  Public Key: {}...{}",
        &identity.public_key_hex[..8],
        &identity.public_key_hex[identity.public_key_hex.len() - 8..]
    );
    info!("  Generated: {}", identity.generated_at);
    info!("  Version: {}", identity.version);

    Ok(())
}

/// Main event loop with proper lifecycle management
async fn run_event_loop(config: &Config, _identity: &NodeIdentityData) -> Result<()> {
    use p2p_ai_agents::application::{lifecycle::LifecycleManager, Application};
    use std::sync::Arc;

    info!("Event loop running...");

    // Create application
    let app = Application::new();

    // Check for startup diagnostics flag
    let cli = Cli::parse();
    let enable_diagnostics = cli.startup_diagnostics;

    // Create lifecycle manager with diagnostics if enabled
    let lifecycle_manager = if enable_diagnostics {
        Arc::new(LifecycleManager::new(app.clone()).with_diagnostics(true))
    } else {
        Arc::new(LifecycleManager::new(app.clone()))
    };

    // Initialize, register, and start the application
    info!("Initializing application...");
    lifecycle_manager
        .startup()
        .await
        .context("Failed to start application")?;

    info!("✅ Application started successfully");

    // Check readiness indicator
    if config.readiness_file_enabled {
        let ready_path = config.storage_path.join(".ready");
        if ready_path.exists() {
            info!("📄 Readiness file: {}", ready_path.display());
        }
    }

    // Setup signal handling
    let shutdown_manager = lifecycle_manager.clone();

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).context("Failed to register SIGTERM handler")?;
        let mut sigint =
            signal(SignalKind::interrupt()).context("Failed to register SIGINT handler")?;

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, initiating graceful shutdown");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT (Ctrl+C), initiating graceful shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .context("Failed to listen for Ctrl+C")?;

        info!("Received Ctrl+C, initiating graceful shutdown");
    }

    // Gracefully shutdown
    info!("Shutting down application...");
    shutdown_manager
        .shutdown()
        .await
        .context("Failed to shutdown application gracefully")?;

    info!("✅ Application shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test basic parsing
        let cli = Cli::parse_from(["p2p-ai-agents", "start"]);
        assert!(matches!(cli.command, Some(Commands::Start)));
    }

    #[test]
    fn test_cli_with_options() {
        let cli = Cli::parse_from([
            "p2p-ai-agents",
            "--port",
            "9001",
            "--max-peers",
            "50",
            "start",
        ]);
        assert_eq!(cli.port, Some(9001));
        assert_eq!(cli.max_peers, Some(50));
    }

    #[test]
    fn test_daemon_flag() {
        let cli = Cli::parse_from(["p2p-ai-agents", "--daemon", "start"]);
        assert!(cli.daemon);
    }

    #[test]
    fn test_daemon_with_custom_pid_file() {
        let cli = Cli::parse_from([
            "p2p-ai-agents",
            "--daemon",
            "--pid-file",
            "/tmp/custom.pid",
            "start",
        ]);
        assert!(cli.daemon);
        assert_eq!(cli.pid_file, Some("/tmp/custom.pid".to_string()));
    }
}
