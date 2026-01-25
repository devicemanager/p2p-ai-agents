use anyhow::Result;
use p2p_ai_agents::project_manager::ProjectManager;
use std::path::PathBuf;

#[tokio::test]
async fn test_project_manager_template_resolution() -> Result<()> {
    // Create a temporary master workflow file
    let workflow_path = PathBuf::from("tests/fixtures/test_workflow.yaml");

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

#[tokio::test]
async fn test_project_manager_loop_execution() -> Result<()> {
    let output_root = PathBuf::from("_bmad-output/test_loop_execution");
    if output_root.exists() {
        tokio::fs::remove_dir_all(&output_root).await?;
    }

    // 1. Create sub-workflow for the loop
    let sub_wf_path = PathBuf::from("tests/fixtures/sub_workflow.yaml");
    let sub_wf_content = "name: Sub Workflow
description: Process single item
steps:
  - name: Write Item
    kind: shell
    command: 'echo \"Processing {{item_name}}\" > {{output_root}}/{{item_id}}.txt'
";
    if let Some(parent) = sub_wf_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&sub_wf_path, sub_wf_content).await?;

    // 2. Create master workflow
    let master_wf_path = PathBuf::from("tests/fixtures/master_loop_workflow.yaml");
    let master_wf_content = "name: Master Loop
description: Test looping
params:
  - name: output_root
    type: string
    description: Output root
    default: \"_bmad-output/test_loop_execution\"
phases:
  - name: Generate List
    description: Generate JSON list
    params: []
    sub_phases: []
    workflow: tests/fixtures/generator.yaml
";

    tokio::fs::write(&master_wf_path, master_wf_content).await?;

    // 3. Create generator workflow (the "Planning" phase)
    let gen_wf_path = PathBuf::from("tests/fixtures/generator.yaml");
    // Note: We use `shell` to simulate `ai` outputting JSON for stability
    let gen_wf_content = "name: Generator
description: Generate list
steps:
  - name: Create JSON
    kind: shell
    command: 'echo ''[{\"item_id\": \"1\", \"item_name\": \"First\"}, {\"item_id\": \"2\", \"item_name\": \"Second\"}]'''
    output_var: items_json
  
  - name: Process Loop
    kind: loop
    loop_var: items_json
    loop_workflow: tests/fixtures/sub_workflow.yaml
";
    tokio::fs::write(&gen_wf_path, gen_wf_content).await?;

    // 4. Run ProjectManager
    let pm = ProjectManager::new(master_wf_path.clone(), "Test Loop".to_string()).await?;
    pm.run().await?;

    // 5. Verify outputs
    let file1 = output_root.join("1.txt");
    let file2 = output_root.join("2.txt");

    assert!(file1.exists(), "File 1.txt should exist");
    assert!(file2.exists(), "File 2.txt should exist");

    let content1 = tokio::fs::read_to_string(file1).await?;
    assert!(content1.contains("Processing First"));

    let content2 = tokio::fs::read_to_string(file2).await?;
    assert!(content2.contains("Processing Second"));

    // Cleanup
    let _ = tokio::fs::remove_file(sub_wf_path).await;
    let _ = tokio::fs::remove_file(master_wf_path).await;
    let _ = tokio::fs::remove_file(gen_wf_path).await;
    let _ = tokio::fs::remove_dir_all(output_root).await;

    Ok(())
}
