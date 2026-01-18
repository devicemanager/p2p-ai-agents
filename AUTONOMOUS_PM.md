# Autonomous Project Manager

This document describes the Autonomous Project Manager, a system designed to orchestrate a team of specialized AI agents to complete a project from a high-level goal, without human intervention.

## Overview

The Project Manager operates by executing a master workflow file that defines the phases of a project. Each phase is delegated to a specialized sub-workflow, which in turn is executed by a team of AI agents.

## Usage

To run the Autonomous Project Manager, use the `run-project` command:

```bash
cargo run -- run-project --workflow /path/to/your/master_workflow.yaml --goal "Your high-level project goal"
```

### Arguments

- `--workflow <PATH>`: The path to the master workflow YAML file. This file defines the stages of your project.
- `--goal <STRING>`: A high-level description of the project goal. This goal is passed to the agents at the start of the project.

## Master Workflow

The master workflow is a YAML file that defines the project lifecycle. It is a list of phases, each with a `name` and a `workflow` file.

Example `master_workflow.yaml`:
```yaml
phases:
  - name: "Analysis"
    workflow: "_bmad/sub_workflows/analysis.yaml"
  - name: "Planning"
    workflow: "_bmad/sub_workflows/planning.yaml"
  - name: "Solutioning"
    workflow: "_bmad/sub_workflows/solutioning.yaml"
  - name: "Implementation"
    workflow: "_bmad/sub_workflows/implementation.yaml"
  - name: "Verification"
    workflow: "_bmad/sub_workflows/verification.yaml"
```
