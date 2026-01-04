//! Structured JSON logging configuration
//!
//! This module provides structured JSON logging with correlation IDs for
//! distributed tracing. Logs are emitted in JSON format suitable for
//! aggregation and analysis.

use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log format (json, pretty, compact)
    #[serde(default = "default_format")]
    pub format: LogFormat,

    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR)
    #[serde(default = "default_level")]
    pub level: String,

    /// Target-specific filters
    #[serde(default)]
    pub target_filters: Vec<String>,

    /// Output destination (stdout or file path)
    #[serde(default = "default_output")]
    pub output: String,
}

fn default_format() -> LogFormat {
    LogFormat::Json
}

fn default_level() -> String {
    "INFO".to_string()
}

fn default_output() -> String {
    "stdout".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Json,
            level: "INFO".to_string(),
            target_filters: vec![],
            output: "stdout".to_string(),
        }
    }
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON format (structured)
    Json,
    /// Pretty format (human-readable)
    Pretty,
    /// Compact format (minimal)
    Compact,
}

/// Error type for logging operations
#[derive(Debug, Error)]
pub enum LoggingError {
    /// Configuration error
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Initialization error
    #[error("Failed to initialize logging: {0}")]
    Initialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Initialize the logging system with the given configuration
///
/// This function sets up structured JSON logging with correlation IDs.
/// It should be called once at application startup.
///
/// # Example
/// ```no_run
/// use p2p_ai_agents::core::logging::{LoggingConfig, init_logging};
///
/// let config = LoggingConfig::default();
/// init_logging(&config).expect("Failed to initialize logging");
/// ```
pub fn init_logging(config: &LoggingConfig) -> Result<(), LoggingError> {
    // Build the env filter with the base level and target filters
    let mut filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Add target-specific filters
    for target_filter in &config.target_filters {
        filter = filter.add_directive(target_filter.parse().map_err(|e| {
            LoggingError::Config(format!("Invalid filter '{}': {}", target_filter, e))
        })?);
    }

    // Create the subscriber based on format
    match config.format {
        LogFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(false)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_writer(io::stdout);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LoggingError::Initialization(e.to_string()))?;
        }
        LogFormat::Pretty => {
            let layer = fmt::layer()
                .pretty()
                .with_target(true)
                .with_level(true)
                .with_writer(io::stdout);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LoggingError::Initialization(e.to_string()))?;
        }
        LogFormat::Compact => {
            let layer = fmt::layer()
                .compact()
                .with_target(true)
                .with_level(true)
                .with_writer(io::stdout);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LoggingError::Initialization(e.to_string()))?;
        }
    }

    Ok(())
}

/// Initialize logging with default configuration
///
/// This is a convenience function that initializes JSON logging at INFO level.
///
/// # Example
/// ```no_run
/// use p2p_ai_agents::core::logging::init_default_logging;
///
/// init_default_logging().expect("Failed to initialize logging");
/// ```
pub fn init_default_logging() -> Result<(), LoggingError> {
    init_logging(&LoggingConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "INFO");
        assert_eq!(config.output, "stdout");
        assert!(config.target_filters.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = LoggingConfig {
            format: LogFormat::Json,
            level: "DEBUG".to_string(),
            target_filters: vec!["p2p_ai_agents=TRACE".to_string()],
            output: "stdout".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LoggingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.format, deserialized.format);
        assert_eq!(config.level, deserialized.level);
    }

    #[test]
    fn test_log_format_variants() {
        assert_eq!(
            serde_json::from_str::<LogFormat>("\"json\"").unwrap(),
            LogFormat::Json
        );
        assert_eq!(
            serde_json::from_str::<LogFormat>("\"pretty\"").unwrap(),
            LogFormat::Pretty
        );
        assert_eq!(
            serde_json::from_str::<LogFormat>("\"compact\"").unwrap(),
            LogFormat::Compact
        );
    }

    // Note: Cannot test actual logging initialization in unit tests
    // as it can only be initialized once per process.
    // Integration tests handle this.
}
