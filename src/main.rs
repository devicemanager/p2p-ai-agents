//! P2P AI Agents: Node initialization and lifecycle management
//!
//! This binary provides the main entry point for running a P2P AI agent node.
//! It handles initialization, configuration, identity management, and the main event loop.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use p2p_ai_agents::core::{
    config::Config,
    identity::{load_or_create_identity, NodeIdentityData},
    logging::{init_logging, LogFormat, LoggingConfig},
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "p2p-ai-agents")]
#[command(about = "A distributed peer-to-peer network of AI agents")]
struct Cli {
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

    /// Run as daemon (Unix only)
    #[arg(long, global = true)]
    daemon: bool,

    /// PID file path (for daemon mode)
    #[arg(long, global = true)]
    pid_file: Option<String>,
}

#[derive(Subcommand, Debug)]
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

    /// List connected peers
    Peers,

    /// Monitor node status continuously
    Monitor {
        /// Refresh interval in seconds
        #[arg(long, default_value = "2")]
        interval: u64,
    },

    /// Display node ID
    NodeId,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
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

    // Handle commands
    match cli.command {
        Some(Commands::Init { force }) => {
            cmd_init(force).await?;
        }
        Some(Commands::Status) => {
            cmd_status(&cli).await?;
        }
        Some(Commands::NodeId) => {
            cmd_node_id().await?;
        }
        Some(Commands::Start) | None => {
            cmd_start(&cli).await?;
        }
        Some(Commands::Peers) => {
            cmd_peers().await?;
        }
        Some(Commands::Monitor { interval }) => {
            cmd_monitor(interval).await?;
        }
    }

    Ok(())
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
    let status_path = std::path::Path::new("data/node_status.json");
    
    if status_path.exists() {
        match tokio::fs::read_to_string(status_path).await {
            Ok(content) => {
                match serde_json::from_str::<NodeStatus>(&content) {
                    Ok(status) => {
                        info!("Node ID:         {}", status.node_id);
                        info!("State:           {}", status.state);
                        info!("Version:         {}", status.version);
                        info!("Uptime:          {}s", status.uptime_seconds);
                        info!("Peers:           {}", status.connected_peers);
                        info!("Agents:          {}", status.active_agents);
                        info!("Tasks Processed: {}", status.tasks_processed);
                        info!("Memory:          {:.2} MB / {:.2} MB", 
                             status.memory_usage_bytes as f64 / 1024.0 / 1024.0,
                             status.total_memory_bytes as f64 / 1024.0 / 1024.0);
                        info!("CPU Usage:       {:.1}%", status.cpu_usage_percent);
                        info!("Last Update:     {}", status.timestamp.to_rfc3339());
                        return Ok(());
                    },
                    Err(e) => {
                        tracing::warn!("Failed to parse status file: {}", e);
                    }
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
        },
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
        },
        Err(e) => {
             info!("  Configuration: Error loading ({})", e);
        }
    }

    Ok(())
}

/// List connected peers
async fn cmd_peers() -> Result<()> {
    // Try to load dynamic status
    let status_path = std::path::Path::new("data/node_status.json");
    
    if status_path.exists() {
        match tokio::fs::read_to_string(status_path).await {
            Ok(content) => {
                match serde_json::from_str::<NodeStatus>(&content) {
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
                    },
                    Err(e) => {
                        tracing::warn!("Failed to parse status file: {}", e);
                    }
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

    // Handle daemon mode
    if cli.daemon {
        #[cfg(not(unix))]
        {
            anyhow::bail!("Daemon mode is not supported on Windows. Please run in foreground mode.");
        }

        #[cfg(unix)]
        {
            use p2p_ai_agents::daemon::{PidFileManager, daemonize, default_log_path};

            info!("🔧 Configuring daemon mode...");

            // Setup PID file manager
            let pid_file_path = if let Some(path) = &cli.pid_file {
                PathBuf::from(path)
            } else {
                PidFileManager::default_path()
                    .context("Failed to get default PID file path")?
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
            let log_path = default_log_path()
                .context("Failed to get default log path")?;

            info!("📝 Log file: {}", log_path.display());
            info!("📋 PID file: {}", pid_manager.path().display());
            info!("🔀 Forking to background...");

            // Daemonize the process (this will fork and exit the parent)
            daemonize(log_path.clone())
                .context("Failed to daemonize process")?;

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

            init_logging(&logging_config)
                .context("Failed to re-initialize logging after daemonization")?;

            info!("✅ Daemon process started");
            info!("📝 Logs: {}", log_path.display());

            // Write PID file
            pid_manager.write()
                .context("Failed to write PID file")?;

            // Setup cleanup on exit
            let pid_manager_clone = PidFileManager::new(pid_manager.path().to_path_buf());
            let cleanup_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Run the node
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    run_node(cli).await
                })
            }));

            // Clean up PID file
            if let Err(e) = pid_manager_clone.remove() {
                tracing::warn!("Failed to remove PID file: {}", e);
            }

            // Re-throw panic if there was one
            match cleanup_result {
                Ok(result) => result?,
                Err(e) => std::panic::resume_unwind(e),
            }

            return Ok(());
        }
    }

    // Normal foreground mode
    run_node(cli).await
}

use std::path::PathBuf;

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
    config.validate()
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
    info!("📡 Public Key: {}...{}", 
          &identity.public_key_hex[..8], 
          &identity.public_key_hex[identity.public_key_hex.len()-8..]);

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
    info!("  Public Key: {}...{}", 
          &identity.public_key_hex[..8], 
          &identity.public_key_hex[identity.public_key_hex.len()-8..]);
    info!("  Generated: {}", identity.generated_at);
    info!("  Version: {}", identity.version);

    Ok(())
}

/// Main event loop with proper lifecycle management
async fn run_event_loop(_config: &Config, _identity: &NodeIdentityData) -> Result<()> {
    use p2p_ai_agents::application::{Application, lifecycle::LifecycleManager};
    use std::sync::Arc;

    info!("Event loop running...");
    
    // Create application
    let app = Application::new();
    
    // Create lifecycle manager
    let lifecycle_manager = Arc::new(LifecycleManager::new(app.clone()));
    
    // Initialize, register, and start the application
    info!("Initializing application...");
    lifecycle_manager.startup().await
        .context("Failed to start application")?;

    info!("✅ Application started successfully");
    
    // Setup signal handling
    let shutdown_manager = lifecycle_manager.clone();
    
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        
        let mut sigterm = signal(SignalKind::terminate())
            .context("Failed to register SIGTERM handler")?;
        let mut sigint = signal(SignalKind::interrupt())
            .context("Failed to register SIGINT handler")?;
        
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
    shutdown_manager.shutdown().await
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
        let cli = Cli::parse_from(&["p2p-ai-agents", "start"]);
        assert!(matches!(cli.command, Some(Commands::Start)));
    }

    #[test]
    fn test_cli_with_options() {
        let cli = Cli::parse_from(&[
            "p2p-ai-agents",
            "--port", "9001",
            "--max-peers", "50",
            "start",
        ]);
        assert_eq!(cli.port, Some(9001));
        assert_eq!(cli.max_peers, Some(50));
    }

    #[test]
    fn test_daemon_flag() {
        let cli = Cli::parse_from(&[
            "p2p-ai-agents",
            "--daemon",
            "start",
        ]);
        assert!(cli.daemon);
    }

    #[test]
    fn test_daemon_with_custom_pid_file() {
        let cli = Cli::parse_from(&[
            "p2p-ai-agents",
            "--daemon",
            "--pid-file", "/tmp/custom.pid",
            "start",
        ]);
        assert!(cli.daemon);
        assert_eq!(cli.pid_file, Some("/tmp/custom.pid".to_string()));
    }
}
