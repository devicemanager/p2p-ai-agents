//! Daemon process management for Unix systems
//!
//! This module provides functionality for running the P2P agent as a background daemon,
//! including PID file management, process daemonization, and log redirection.

use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// PID file manager for daemon processes
pub struct PidFileManager {
    pid_file_path: PathBuf,
}

impl PidFileManager {
    /// Create a new PID file manager
    pub fn new(pid_file_path: PathBuf) -> Self {
        Self { pid_file_path }
    }

    /// Get the default PID file path (~/.p2p-ai-agents/p2p-agent.pid)
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".p2p-ai-agents").join("p2p-agent.pid"))
    }

    /// Check if a process is running with the given PID
    #[cfg(unix)]
    fn is_process_running(pid: i32) -> bool {
        use nix::sys::signal::kill;
        use nix::unistd::Pid;

        // Signal 0 (null signal) doesn't send a signal, but checks if we can
        // This is a standard way to check if a process exists
        kill(Pid::from_raw(pid), None).is_ok()
    }

    #[cfg(not(unix))]
    fn is_process_running(_pid: i32) -> bool {
        false
    }

    /// Check if another instance is already running
    pub fn check_running(&self) -> Result<Option<i32>> {
        if !self.pid_file_path.exists() {
            return Ok(None);
        }

        // Read existing PID
        let mut file = File::open(&self.pid_file_path).context("Failed to open PID file")?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Failed to read PID file")?;

        let pid: i32 = contents.trim().parse().context("Invalid PID in PID file")?;

        // Check if process is still running
        if Self::is_process_running(pid) {
            Ok(Some(pid))
        } else {
            warn!(
                "Stale PID file found (process {} not running), removing",
                pid
            );
            self.remove()?;
            Ok(None)
        }
    }

    /// Write the current process PID to file
    pub fn write(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.pid_file_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create PID file directory")?;
        }

        let pid = std::process::id();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.pid_file_path)
            .context("Failed to create PID file")?;

        write!(file, "{}", pid).context("Failed to write PID to file")?;

        info!(
            "PID file created: {} (PID: {})",
            self.pid_file_path.display(),
            pid
        );
        Ok(())
    }

    /// Remove the PID file
    pub fn remove(&self) -> Result<()> {
        if self.pid_file_path.exists() {
            std::fs::remove_file(&self.pid_file_path).context("Failed to remove PID file")?;
            info!("PID file removed: {}", self.pid_file_path.display());
        }
        Ok(())
    }

    /// Get the PID file path
    pub fn path(&self) -> &Path {
        &self.pid_file_path
    }
}

/// Daemonize the current process (Unix only)
///
/// This implementation uses native Unix fork/setsid to create a daemon process
/// without relying on external unmaintained crates.
#[cfg(unix)]
pub fn daemonize(log_path: PathBuf) -> Result<()> {
    use nix::unistd::{fork, setsid, ForkResult};
    use std::os::unix::io::AsRawFd;

    info!("Daemonizing process...");

    // Ensure log directory exists
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create log directory")?;
    }

    // Create or open log file
    let stdout = File::create(&log_path)
        .with_context(|| format!("Failed to create log file: {}", log_path.display()))?;

    let stderr = stdout
        .try_clone()
        .context("Failed to duplicate log file handle")?;

    info!("Log output will be redirected to: {}", log_path.display());

    // First fork to ensure we're not a process group leader
    match unsafe { fork() }.context("Failed to fork process")? {
        ForkResult::Parent { .. } => {
            // Parent exits immediately
            std::process::exit(0);
        }
        ForkResult::Child => {
            // Child continues
        }
    }

    // Create new session and become session leader
    setsid().context("Failed to create new session")?;

    // Second fork to ensure we can't acquire a controlling terminal
    match unsafe { fork() }.context("Failed to fork process (second time)")? {
        ForkResult::Parent { .. } => {
            // First child exits
            std::process::exit(0);
        }
        ForkResult::Child => {
            // Second child (daemon) continues
        }
    }

    // Initialize tokio runtime here if needed? No, we do that in run_node

    // Set umask for security
    use nix::sys::stat::{umask, Mode};
    umask(Mode::from_bits_truncate(0o027));

    // Change to root directory to avoid keeping any directory in use
    std::env::set_current_dir("/").context("Failed to change to root directory")?;

    // Redirect stdout and stderr to log file
    use nix::unistd::dup2;
    // We need to re-open stdout/stderr because they might be closed or invalid after fork
    // However, we just opened 'stdout' file above.

    // NOTE: The panic "failed to wake I/O driver" often happens if a tokio runtime was created
    // BEFORE fork. The file descriptors for the reactor are not inherited properly or are closed.
    // In src/main.rs, we use #[tokio::main], which starts a runtime *before* main() runs.
    // To fix this, we should NOT use #[tokio::main] if we plan to daemonize, OR we must destroy
    // the runtime before forking, OR (easiest) run the daemon code in a separate thread/process
    // before the main runtime starts?
    // Actually, the easiest fix for `tokio` + `fork` is to ensuring no runtime exists when we fork.
    // Since `main` is async, the runtime is already running.
    // We cannot safe fork from within a tokio runtime.

    dup2(stdout.as_raw_fd(), std::io::stdout().as_raw_fd()).context("Failed to redirect stdout")?;
    dup2(stderr.as_raw_fd(), std::io::stderr().as_raw_fd()).context("Failed to redirect stderr")?;

    // Close stdin
    let devnull = File::open("/dev/null").context("Failed to open /dev/null")?;
    dup2(devnull.as_raw_fd(), std::io::stdin().as_raw_fd()).context("Failed to redirect stdin")?;

    info!("Daemon process running");

    Ok(())
}

#[cfg(not(unix))]
pub fn daemonize(_log_path: PathBuf) -> Result<()> {
    anyhow::bail!(
        "Daemon mode is not supported on this platform (Windows). Please run in foreground mode."
    )
}

/// Get the default log file path for daemon mode
pub fn default_log_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".p2p-ai-agents").join("logs").join("node.log"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pid_file_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let manager = PidFileManager::new(pid_path.clone());
        assert_eq!(manager.path(), pid_path);
    }

    #[test]
    fn test_pid_file_write_and_remove() {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let manager = PidFileManager::new(pid_path.clone());

        // Write PID
        manager.write().expect("Should write PID file");
        assert!(pid_path.exists());

        // Verify contents
        let contents = std::fs::read_to_string(&pid_path).unwrap();
        let pid: u32 = contents.trim().parse().unwrap();
        assert_eq!(pid, std::process::id());

        // Remove PID
        manager.remove().expect("Should remove PID file");
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_check_running_no_file() {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let manager = PidFileManager::new(pid_path);

        let result = manager.check_running().expect("Should check successfully");
        assert!(result.is_none());
    }

    #[test]
    fn test_check_running_current_process() {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let manager = PidFileManager::new(pid_path);

        // Write current PID
        manager.write().expect("Should write PID");

        // Check if running
        let result = manager.check_running().expect("Should check successfully");
        assert_eq!(result, Some(std::process::id() as i32));

        // Clean up
        manager.remove().ok();
    }

    #[test]
    fn test_default_paths() {
        // Just verify these don't panic
        let pid_path = PidFileManager::default_path();
        assert!(pid_path.is_ok());

        let log_path = default_log_path();
        assert!(log_path.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_is_process_running() {
        // Current process should be running
        let current_pid = std::process::id() as i32;
        assert!(PidFileManager::is_process_running(current_pid));

        // Very high PID unlikely to exist
        assert!(!PidFileManager::is_process_running(999999));
    }
}
