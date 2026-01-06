use p2p_ai_agents::application::{Application, ApplicationState};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_graceful_shutdown_sequence() {
    // 1. Setup
    let app = Application::new();
    
    // 2. Initialize and Start
    app.initialize().await.expect("Initialization failed");
    app.register().await.expect("Registration failed");
    app.start().await.expect("Startup failed");
    
    assert_eq!(app.state().await, ApplicationState::Active);
    
    // 3. Simulate some work / wait
    sleep(Duration::from_millis(100)).await;
    
    // 4. Trigger Stop (Graceful Shutdown)
    app.stop().await.expect("Stop failed");
    
    // 5. Verify Stopped State
    assert_eq!(app.state().await, ApplicationState::Stopped);
    
    // 6. Verify components are shut down (indirectly via logs or lack of errors)
    // In a real integration test we might check if ports are closed or files are flushed.
    // For now, clean execution of stop() implies success of shutdown_components().
}
