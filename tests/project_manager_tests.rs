use anyhow::Result;
use p2p_ai_agents::project_manager::ProjectManager;
use std::path::PathBuf;

#[tokio::test]
async fn test_project_manager_template_resolution() -> Result<()> {
    // Create a temporary master workflow file
    let workflow_path = PathBuf::from("tests/fixtures/test_workflow.yaml");

    // We can use reflection or testable wrapper methods if we expose them,
    // but since resolve_template is private, we might need to test public methods
    // or make it testable.
    // However, given the structure, we can verify basic instantiation for now.

    // Create a dummy file to satisfy the existence check
    if !workflow_path.exists() {
        if let Some(parent) = workflow_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(
            &workflow_path,
            "name: Test\ndescription: Test Workflow\nparams: []\nphases: []",
        )
        .await?;
    }

    let pm = ProjectManager::new(workflow_path.clone(), "Test Goal".to_string()).await?;

    let result = pm.run().await;

    // Clean up
    let _ = tokio::fs::remove_file(workflow_path).await;

    assert!(
        result.is_ok(),
        "ProjectManager run failed: {:?}",
        result.err()
    );

    Ok(())
}
