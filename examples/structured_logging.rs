//! Example demonstrating structured JSON logging with correlation IDs
//!
//! This example shows how to use the structured logging system with correlation IDs
//! for distributed tracing.
//!
//! Run with: cargo run --example structured_logging

use p2p_ai_agents::core::{
    correlation::CorrelationId,
    logging::{init_logging, LogFormat, LoggingConfig},
};
use tracing::{debug, error, info, instrument, warn};

#[instrument(fields(correlation_id = %CorrelationId::new()))]
async fn process_user_request(user_id: u32) -> Result<(), String> {
    info!(user_id = user_id, "Processing user request");

    // Simulate some processing steps
    load_user_data(user_id).await?;
    validate_user(user_id).await?;
    perform_operation(user_id).await?;

    info!(user_id = user_id, "Request completed successfully");
    Ok(())
}

#[instrument]
async fn load_user_data(user_id: u32) -> Result<(), String> {
    debug!(user_id = user_id, "Loading user data from database");
    
    // Simulate database delay
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    info!(user_id = user_id, "User data loaded successfully");
    Ok(())
}

#[instrument]
async fn validate_user(user_id: u32) -> Result<(), String> {
    debug!(user_id = user_id, "Validating user permissions");
    
    // Simulate validation
    if user_id == 999 {
        let err = "User not found";
        error!(
            user_id = user_id,
            error_type = "ValidationError",
            error_message = err,
            "User validation failed"
        );
        return Err(err.to_string());
    }
    
    info!(user_id = user_id, "User validated successfully");
    Ok(())
}

#[instrument]
async fn perform_operation(user_id: u32) -> Result<(), String> {
    info!(user_id = user_id, "Performing operation");
    
    // Simulate operation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    if user_id % 10 == 0 {
        warn!(user_id = user_id, "Operation took longer than expected");
    }
    
    info!(user_id = user_id, "Operation completed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize JSON logging
    let config = LoggingConfig {
        format: LogFormat::Json,
        level: "DEBUG".to_string(),
        target_filters: vec!["structured_logging=TRACE".to_string()],
        output: "stdout".to_string(),
    };
    
    init_logging(&config)?;
    
    info!("Starting structured logging example");
    
    // Process several requests, each with its own correlation ID
    for user_id in [1, 2, 999, 10, 5] {
        match process_user_request(user_id).await {
            Ok(()) => info!(user_id = user_id, "Request succeeded"),
            Err(e) => error!(
                user_id = user_id,
                error_message = e,
                "Request failed"
            ),
        }
    }
    
    info!("Example completed");
    
    Ok(())
}
