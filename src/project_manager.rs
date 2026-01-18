//! src/project_manager.rs

use crate::task::{executor::TaskExecutor, Task};
use anyhow::{Context, Result};
use futures::future::BoxFuture;
use libp2p::identity::Keypair;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{info, instrument};

// Represents the entire master workflow, like project_lifecycle.yaml
#[derive(Debug, Serialize, Deserialize)]
struct MasterWorkflow {
    name: String,
    description: String,
    params: Vec<WorkflowParam>,
    phases: Vec<Phase>,
}

// A parameter for the workflow
#[derive(Debug, Serialize, Deserialize)]
struct WorkflowParam {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
    description: String,
    default: Option<String>,
}

// A major phase in the project lifecycle
#[derive(Debug, Serialize, Deserialize)]
struct Phase {
    name: String,
    description: String,
    workflow: Option<String>, // Path to the sub-workflow
    #[serde(default)]
    params: Vec<PhaseParam>,
    #[serde(default, rename = "sub_phases")]
    sub_phases: Vec<Phase>, // For nested phases
}

// A parameter for a specific phase, linking outputs to inputs
#[derive(Debug, Serialize, Deserialize)]
struct PhaseParam {
    name: String,
    value: String,
}

// Represents a sub-workflow file
#[derive(Debug, Serialize, Deserialize)]
struct SubWorkflow {
    name: String,
    description: String,
    steps: Vec<WorkflowStep>,
}

// A single step in a sub-workflow
#[derive(Debug, Serialize, Deserialize)]
struct WorkflowStep {
    name: String,
    #[serde(default = "default_step_kind")]
    kind: StepKind,
    command: String, // Serves as the shell command OR the AI prompt
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum StepKind {
    Shell,
    Ai,
}

fn default_step_kind() -> StepKind {
    StepKind::Shell
}

/// The ProjectManager is responsible for orchestrating an end-to-end project
/// by executing a master workflow.
pub struct ProjectManager {
    master_workflow_path: PathBuf,
    goal: String,
    task_executor: TaskExecutor,
    identity: PeerId,
}

impl ProjectManager {
    /// Creates a new ProjectManager.
    pub async fn new(master_workflow_path: PathBuf, goal: String) -> Result<Self> {
        if !master_workflow_path.exists() {
            anyhow::bail!(
                "Master workflow file not found at: {}",
                master_workflow_path.display()
            );
        }

        // Generate a temporary identity for this standalone run
        let keypair = Keypair::generate_ed25519();
        let identity = PeerId::from(keypair.public());

        Ok(Self {
            master_workflow_path,
            goal,
            task_executor: TaskExecutor::new(),
            identity,
        })
    }

    /// Runs the entire project lifecycle autonomously.
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        info!("Starting autonomous project: {}", self.goal);

        let workflow_content = tokio::fs::read_to_string(&self.master_workflow_path).await?;
        let master_workflow: MasterWorkflow = serde_yaml::from_str(&workflow_content)?;

        let output_root = master_workflow
            .params
            .iter()
            .find(|p| p.name == "output_root")
            .and_then(|p| p.default.clone())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./_bmad-output/autonomous-project"));

        tokio::fs::create_dir_all(&output_root).await?;
        info!("Output will be generated in: {}", output_root.display());

        let mut context_map = HashMap::new();
        context_map.insert("project_goal".to_string(), self.goal.clone());
        context_map.insert(
            "output_root".to_string(),
            output_root.to_string_lossy().to_string(),
        );

        for phase in &master_workflow.phases {
            self.execute_phase(phase, &mut context_map).await?;
        }

        info!("Autonomous project lifecycle completed successfully.");
        Ok(())
    }

    /// Executes a single phase (or a phase with sub-phases).
    fn execute_phase<'a>(
        &'a self,
        phase: &'a Phase,
        context_map: &'a mut HashMap<String, String>,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            info!("Executing Phase: {}", phase.name);

            if !phase.sub_phases.is_empty() {
                for sub_phase in &phase.sub_phases {
                    self.execute_phase(sub_phase, context_map).await?;
                }
            } else if let Some(workflow_path_template) = &phase.workflow {
                let workflow_path = self.resolve_template(workflow_path_template, context_map);
                info!("  -> Triggering sub-workflow: {}", workflow_path);

                let sub_workflow_content = tokio::fs::read_to_string(&workflow_path)
                    .await
                    .with_context(|| {
                        format!("Failed to read sub-workflow file: {}", workflow_path)
                    })?;
                let sub_workflow: SubWorkflow = serde_yaml::from_str(&sub_workflow_content)?;

                let mut params_for_sub = HashMap::new();
                for param in &phase.params {
                    let resolved_value = self.resolve_template(&param.value, context_map);
                    params_for_sub.insert(param.name.clone(), resolved_value);
                }

                // Add the sub-workflow params to the main context for its steps
                context_map.extend(params_for_sub.clone());

                for step in &sub_workflow.steps {
                    info!("    -> Executing Step: {}", step.name);
                    let content_template = &step.command;
                    let resolved_content = self.resolve_template(content_template, context_map);

                    match step.kind {
                        StepKind::Shell => self.run_command(&resolved_content).await?,
                        StepKind::Ai => self.run_ai_task(&resolved_content).await?,
                    }
                }

                info!("  -> Sub-workflow completed: {}", workflow_path);
            }
            Ok(())
        })
    }

    async fn run_command(&self, command: &str) -> Result<()> {
        info!("      [Shell] $ {}", command);

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        let status = cmd
            .status()
            .await
            .with_context(|| format!("Failed to execute command: {}", command))?;

        if !status.success() {
            anyhow::bail!("Command failed with status: {}", status);
        }

        Ok(())
    }

    async fn run_ai_task(&self, prompt: &str) -> Result<()> {
        info!("      [AI] Prompt: {}", prompt);

        let task = Task::new(prompt.to_string(), self.identity);
        let result = self.task_executor.execute(task).await;

        info!("      [AI] Result: {}", result.result);
        info!("      [AI] Duration: {}ms", result.duration_ms);

        Ok(())
    }

    /// Resolves a template string using the context map.
    fn resolve_template(&self, template: &str, context_map: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in context_map {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}
