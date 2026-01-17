# Agent Benchmark: Autonomous Story Creation

This document outlines a process for benchmarking the autonomous capabilities of a multi-agent system within the `p2p-ai-agents` repository.

## 1. Goal

The primary goal of this benchmark is to assess whether a cluster of AI agents, each with a specific role, can autonomously collaborate to transform a high-level user request into a well-defined, actionable development story.

This test measures the system's ability to:
- Orchestrate a complex, multi-step workflow.
- Manage dialogue and collaboration between different agent personas.
- Utilize its knowledge base to produce a relevant, high-quality outcome.
- Complete the entire process without human intervention.

## 2. Benchmark Scenario

The benchmark uses a pre-defined workflow and agent configuration:

- **Entry-Point Workflow:** `_bmad/core/workflows/party-mode/workflow.md`
  - This workflow is designed to load a team of agents and orchestrate their discussion.
- **Agent Team Configuration:** `_bmad/bmm/teams/default-party.csv`
  - This file defines the roles and initial instructions for the agent team (e.g., Product Manager, Engineer, QA).
- **Core Task:** The `party-mode` session will be directed to execute the `_bmad/bmm/workflows/4-implementation/create-story/workflow.yaml` workflow.
  - This simulates a team receiving a request and collaboratively creating a development story with a checklist, instructions, and template.

## 3. Setup and Execution

### Prerequisites

1.  A running cluster of `p2p-ai-agents` nodes.
2.  The cluster must be correctly networked, with at least one bootstrap node that other nodes can connect to.
3.  The repository must be checked out to the `v1.0.0-benchmark-start` tag to ensure a consistent starting state.

### Running the Benchmark

The benchmark is initiated by running a shell script that simulates the initial user request.

```bash
./scripts/run-benchmark.sh
```

This script will:
1.  Identify a "master" or "orchestrator" agent.
2.  Instruct this agent to begin the `party-mode` workflow.
3.  Pass the `create-story` workflow as the target task for the agent team.

## 4. Measuring Success

The benchmark is considered **successful** if the agent cluster autonomously produces the following artifacts in the designated output directory:

1.  **A completed story file** based on `_bmad/bmm/workflows/4-implementation/create-story/template.md`.
2.  The story file should be populated with a clear title, description, acceptance criteria, and technical tasks.
3.  The process completes without error and without requiring any human input after the initial script is run.

The quality of the output story should be evaluated separately, but the primary benchmark is the successful, autonomous completion of the workflow.
