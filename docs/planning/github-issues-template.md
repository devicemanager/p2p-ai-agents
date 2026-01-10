# GitHub Issues Template for MVP Roadmap

This document provides templates for creating GitHub issues for the 4-week MVP roadmap.

---

## Epic 1: Product Definition & User Research

**Title**: `[Epic 1] Product Definition & User Research`

**Description**:
```markdown
## Goal
Validate the problem hypothesis and define clear target users, use cases, and value proposition through user research.

## Success Criteria
- [ ] 5+ user interviews completed
- [ ] Primary persona and use case defined
- [ ] `docs/planning/project-context.md` fully populated
- [ ] Value proposition articulated in one sentence
- [ ] 4/5 users validate problem as real and important

## Timeline
Week 1: Jan 10 - Jan 17, 2026

## Owner
John (Product Manager)

## Related Documents
- `/docs/planning/project-context.md`
- `/docs/planning/mvp-roadmap.md`

## Stories
- #[TBD] Story 1.1: Conduct User Interviews
- #[TBD] Story 1.2: Create Project Context Document
- #[TBD] Story 1.3: Define MVP Scope and Success Criteria
```

**Labels**: `epic`, `week-1`, `product`, `research`

---

### Story 1.1: Conduct User Interviews

**Title**: `[Story 1.1] Conduct User Interviews`

**Description**:
```markdown
## User Story
As a **Product Manager**, I need to **interview 5-10 potential users** so that I can **validate the problem hypothesis and understand real user needs**.

## Acceptance Criteria
- [ ] Identify 5-10 target users (developers, researchers, or enterprises)
- [ ] Prepare interview script with problem validation questions:
  - What distributed computing challenges do they face?
  - What alternatives are they using? Why do those fail?
  - Would they trust P2P network for compute? Under what conditions?
- [ ] Conduct 5+ interviews (30-45 min each)
- [ ] Document findings in structured format
- [ ] Synthesize common themes and pain points
- [ ] Score problem validation: 4/5+ say "this is a real problem"

## Definition of Done
- [ ] Interview notes captured for each session
- [ ] Synthesis document created with key findings
- [ ] Problem validation score documented
- [ ] Primary user persona identified

## Estimated Effort
5 days

## Dependencies
None

## Owner
John (PM)

## Related
- Epic #[TBD]
- `/docs/planning/project-context.md`
```

**Labels**: `story`, `week-1`, `research`, `user-interviews`

---

### Story 1.2: Create Project Context Document

**Title**: `[Story 1.2] Create Project Context Document`

**Description**:
```markdown
## User Story
As a **Product Team**, we need a **comprehensive project context document** so that **everyone understands who we're building for and why**.

## Acceptance Criteria
- [ ] Template at `docs/planning/project-context.md` populated with research findings
- [ ] All *[To be defined]* sections completed based on user interviews
- [ ] Primary and secondary personas documented with:
  - Goals
  - Pain points
  - Current solutions
  - Success criteria
- [ ] Jobs-to-be-Done framework completed
- [ ] Competitive landscape analysis included
- [ ] Use cases clearly defined (MVP + future)
- [ ] Document reviewed and approved by team

## Definition of Done
- [ ] Document is complete with no TBD placeholders
- [ ] Team can articulate user, problem, and solution clearly
- [ ] Document committed to repo and linked from README

## Estimated Effort
2 days (after interviews complete)

## Dependencies
- Story 1.1: User interviews must be completed

## Owner
John (PM), with input from Mary (Analyst)

## Related
- Epic #[TBD]
- Story 1.1
```

**Labels**: `story`, `week-1`, `documentation`, `product`

---

### Story 1.3: Define MVP Scope and Success Criteria

**Title**: `[Story 1.3] Define MVP Scope and Success Criteria`

**Description**:
```markdown
## User Story
As the **Technical Team**, we need **clear MVP scope and success criteria** so that **we know exactly what to build and when we're done**.

## Acceptance Criteria
- [ ] MVP scope clearly defined in project-context.md:
  - In scope: Specific features and capabilities
  - Out of scope: What we're NOT building
- [ ] North Star metric identified (single measure of success)
- [ ] Supporting metrics defined:
  - Technical validation metrics
  - User validation metrics
  - Adoption signals
- [ ] MVP success criteria documented:
  - Technical: Performance/reliability targets
  - Usability: Setup time, ease of use
  - Validation: User acceptance threshold
- [ ] Scope reviewed and approved by full team

## Definition of Done
- [ ] MVP scope section complete in project-context.md
- [ ] Success metrics clearly quantified
- [ ] Team consensus on scope boundaries
- [ ] No ambiguity about MVP vs post-MVP features

## Estimated Effort
1 day

## Dependencies
- Story 1.1: User research findings
- Story 1.2: Project context draft

## Owner
John (PM), Winston (Architect)

## Related
- Epic #[TBD]
- Story 1.1, 1.2
```

**Labels**: `story`, `week-1`, `product`, `planning`

---

## Epic 2: MVP Architecture & Technical Spike

**Title**: `[Epic 2] MVP Architecture & Technical Spike`

**Description**:
```markdown
## Goal
Design simplified MVP architecture and prove technical feasibility through working prototype.

## Success Criteria
- [ ] `docs/architecture/mvp-architecture.md` created with system diagram
- [ ] Technical spike: 2 agents successfully exchange 1 task
- [ ] Architecture Decision Records (ADRs) for key choices
- [ ] Implementation roadmap defined for Week 3

## Timeline
Week 2: Jan 17 - Jan 24, 2026

## Owner
Winston (Architect)

## Related Documents
- `/docs/architecture/mvp-architecture.md`
- `/docs/architecture/decisions/`
- `/docs/planning/mvp-roadmap.md`

## Stories
- #[TBD] Story 2.1: Create Simplified Architecture Diagram
- #[TBD] Story 2.2: Write MVP Architecture Decision Record
- #[TBD] Story 2.3: Technical Spike for P2P Task Execution
```

**Labels**: `epic`, `week-2`, `architecture`, `technical`

---

### Story 2.1: Create Simplified Architecture Diagram

**Title**: `[Story 2.1] Create Simplified Architecture Diagram`

**Description**:
```markdown
## User Story
As a **Developer**, I need **clear architecture diagrams** so that I **understand the system design before implementing**.

## Acceptance Criteria
- [ ] Create `docs/architecture/mvp-architecture.md`
- [ ] Include system architecture diagram showing:
  - Agent components (identity, networking, task execution)
  - P2P network layer (libp2p)
  - Communication flow between agents
  - Data flow for task execution
- [ ] Document component responsibilities
- [ ] Define interfaces between components
- [ ] Specify technology choices (Rust, libp2p, ed25519, etc.)
- [ ] Identify what's stripped out for MVP (Redis, Supabase, Prometheus)

## Definition of Done
- [ ] Architecture document is complete and committed
- [ ] Diagrams are clear and readable
- [ ] Technical team reviews and approves design
- [ ] Document linked from main README

## Estimated Effort
2 days

## Dependencies
- Epic 1 complete: MVP scope defined

## Owner
Winston (Architect), with review from Barry & Amelia

## Related
- Epic #[TBD]
- `/docs/planning/project-context.md`
```

**Labels**: `story`, `week-2`, `architecture`, `documentation`

---

### Story 2.2: Write MVP Architecture Decision Record

**Title**: `[Story 2.2] Write MVP Architecture Decision Record`

**Description**:
```markdown
## User Story
As a **Technical Team**, we need **documented architecture decisions** so that **future contributors understand our choices and trade-offs**.

## Acceptance Criteria
- [ ] Create ADR template at `docs/architecture/decisions/template.md`
- [ ] Write ADRs for key decisions:
  - ADR-001: Why libp2p for P2P networking
  - ADR-002: Local storage only for MVP
  - ADR-003: Task serialization format
  - ADR-004: Agent discovery mechanism (mDNS vs bootstrap)
- [ ] Each ADR includes:
  - Context and problem statement
  - Decision made
  - Alternatives considered
  - Consequences and trade-offs
- [ ] ADRs reviewed by technical team

## Definition of Done
- [ ] 4+ ADRs written and committed
- [ ] ADRs follow consistent template
- [ ] Technical team has reviewed and approved
- [ ] ADRs referenced from architecture document

## Estimated Effort
2 days

## Dependencies
- Story 2.1: Architecture design

## Owner
Winston (Architect)

## Related
- Epic #[TBD]
- Story 2.1
```

**Labels**: `story`, `week-2`, `architecture`, `documentation`

---

### Story 2.3: Technical Spike for P2P Task Execution

**Title**: `[Story 2.3] Technical Spike for P2P Task Execution`

**Description**:
```markdown
## User Story
As an **Architect**, I need to **prove that P2P task exchange works** before we commit to full implementation.

## Acceptance Criteria
- [ ] Create spike branch: `spike/p2p-task-exchange`
- [ ] Implement minimal proof-of-concept:
  - Agent A: Create libp2p identity and start listening
  - Agent B: Create libp2p identity and start listening
  - Agents discover each other (mDNS or manual peer)
  - Agent A sends simple task to Agent B
  - Agent B receives, executes, and returns result
  - Agent A validates result received
- [ ] Document what works and what doesn't
- [ ] Identify technical challenges and solutions
- [ ] Measure performance metrics (discovery time, task latency)
- [ ] Create technical spike report

## Definition of Done
- [ ] Proof-of-concept code runs successfully
- [ ] Spike report documents findings and risks
- [ ] Performance metrics captured
- [ ] Technical team validates feasibility
- [ ] Learnings inform Week 3 implementation

## Estimated Effort
3 days

## Dependencies
- Story 2.1: Architecture design approved

## Owner
Winston (Architect), Barry (Dev support)

## Related
- Epic #[TBD]
- Story 2.1
```

**Labels**: `story`, `week-2`, `spike`, `technical`, `proof-of-concept`

---

## Epic 3: MVP Implementation

**Title**: `[Epic 3] MVP Implementation`

**Description**:
```markdown
## Goal
Build working MVP demo based on approved architecture, with comprehensive testing and documentation.

## Success Criteria
- [ ] `examples/mvp_demo.rs` works end-to-end
- [ ] Fresh clone to running demo in < 5 minutes
- [ ] Integration tests pass with 100% critical path coverage
- [ ] Documentation enables user to run demo independently
- [ ] CI pipeline validates MVP demo

## Timeline
Week 3: Jan 24 - Jan 31, 2026

## Owners
Barry (Lead), Amelia (Support)

## Related Documents
- `/examples/mvp_demo.rs`
- `/docs/user-guides/QUICK_START_MVP.md`
- `/tests/mvp_integration/`

## Stories
- #[TBD] Story 3.1: Build mvp_demo.rs
- #[TBD] Story 3.2: Remove Non-MVP Dependencies
- #[TBD] Story 3.3: Create Integration Tests for MVP
- #[TBD] Story 3.4: Write 5-Minute Quickstart Guide
```

**Labels**: `epic`, `week-3`, `implementation`, `mvp`

---

### Story 3.1: Build mvp_demo.rs

**Title**: `[Story 3.1] Build mvp_demo.rs`

**Description**:
```markdown
## User Story
As a **Developer**, I will **implement the MVP demo** so that **users can see P2P agents working**.

## Acceptance Criteria
- [ ] Create `examples/mvp_demo.rs`
- [ ] Implement Agent A (task requester):
  - Creates identity
  - Starts P2P network
  - Discovers peers
  - Sends task
  - Receives and validates result
- [ ] Implement Agent B (task executor):
  - Creates identity
  - Starts P2P network
  - Listens for tasks
  - Executes received task
  - Returns result
- [ ] Simple task type: Hash computation or similar
- [ ] Clear progress logging at INFO level
- [ ] Error handling for common failures
- [ ] Success/failure exit codes
- [ ] `cargo run --example mvp_demo` executes successfully

## Definition of Done
- [ ] Demo completes successfully on clean checkout
- [ ] Task exchange happens in < 5 seconds
- [ ] Code follows project style guidelines
- [ ] No unwrap() or panic! in production paths
- [ ] Clippy passes with --deny warnings

## Estimated Effort
4 days

## Dependencies
- Epic 2 complete: Architecture and spike validated

## Owner
Barry (Dev), with code review from Amelia

## Related
- Epic #[TBD]
- `/docs/architecture/mvp-architecture.md`
```

**Labels**: `story`, `week-3`, `implementation`, `demo`

---

### Story 3.2: Remove Non-MVP Dependencies

**Title**: `[Story 3.2] Remove Non-MVP Dependencies`

**Description**:
```markdown
## User Story
As a **Technical Team**, we need to **simplify the MVP** by removing unnecessary dependencies.

## Acceptance Criteria
- [ ] Audit current Cargo.toml dependencies
- [ ] Remove or feature-flag non-MVP dependencies:
  - Redis (feature: storage-redis)
  - Supabase (feature: storage-supabase)
  - Prometheus (feature: metrics-prometheus)
- [ ] Update feature flags for MVP:
  - Default: minimal features only
  - Full: all features (for future)
- [ ] Ensure MVP demo works with default features only
- [ ] Update CI to test MVP configuration
- [ ] Document removed dependencies in ADR

## Definition of Done
- [ ] MVP builds with minimal dependencies
- [ ] Build time improved
- [ ] Default feature set documented
- [ ] No broken imports or unused dependencies

## Estimated Effort
1 day

## Dependencies
- Story 3.1: MVP demo implementation

## Owner
Amelia (Dev)

## Related
- Epic #[TBD]
- `/Cargo.toml`
```

**Labels**: `story`, `week-3`, `refactor`, `dependencies`

---

### Story 3.3: Create Integration Tests for MVP

**Title**: `[Story 3.3] Create Integration Tests for MVP`

**Description**:
```markdown
## User Story
As a **Test Architect**, I need **comprehensive integration tests** so that **we know the MVP works reliably**.

## Acceptance Criteria
- [ ] Create `tests/mvp_integration/` directory
- [ ] Write integration tests:
  - `test_agent_discovery.rs` - Agents find each other
  - `test_task_execution.rs` - Full task exchange flow
  - `test_failure_modes.rs` - Network failures, timeouts
  - `test_signature_verification.rs` - Security validation
- [ ] 100% coverage of MVP critical path
- [ ] Tests run in CI pipeline
- [ ] Test documentation explains scenarios
- [ ] Performance benchmarks captured

## Definition of Done
- [ ] All integration tests pass
- [ ] Tests are deterministic (no flakes)
- [ ] Test suite runs in < 60 seconds
- [ ] Coverage report shows 100% critical path
- [ ] Tests documented in testing guide

## Estimated Effort
3 days

## Dependencies
- Story 3.1: MVP demo implementation

## Owner
Murat (Test Architect), Amelia (Dev support)

## Related
- Epic #[TBD]
- `/docs/development/testing-guide.md`
```

**Labels**: `story`, `week-3`, `testing`, `integration`

---

### Story 3.4: Write 5-Minute Quickstart Guide

**Title**: `[Story 3.4] Write 5-Minute Quickstart Guide`

**Description**:
```markdown
## User Story
As a **New User**, I need a **clear quickstart guide** so that I can **run the MVP demo in 5 minutes**.

## Acceptance Criteria
- [ ] Create `docs/user-guides/QUICK_START_MVP.md`
- [ ] Include step-by-step instructions:
  1. Prerequisites (Rust version, OS requirements)
  2. Clone repository
  3. Build project
  4. Run MVP demo
  5. Understand the output
- [ ] Add troubleshooting section for common issues
- [ ] Include expected output examples
- [ ] Link from main README.md
- [ ] Test guide with fresh user (Sally or external)
- [ ] Validate < 5 minute completion time

## Definition of Done
- [ ] Guide is complete and committed
- [ ] Fresh user can complete demo in < 5 minutes
- [ ] No steps missing or unclear
- [ ] Troubleshooting covers common errors
- [ ] README updated with quickstart link

## Estimated Effort
1 day

## Dependencies
- Story 3.1: MVP demo working

## Owner
Paige (Technical Writer), with UX input from Sally

## Related
- Epic #[TBD]
- `/README.md`
```

**Labels**: `story`, `week-3`, `documentation`, `quickstart`

---

## Epic 4: Validation & Iteration

**Title**: `[Epic 4] Validation & Iteration`

**Description**:
```markdown
## Goal
Demo MVP to target users, gather feedback, and plan next iteration based on validated learnings.

## Success Criteria
- [ ] 3+ user demos completed
- [ ] 2/3 users express intent to use MVP
- [ ] Feedback synthesized and documented
- [ ] Post-MVP roadmap created based on learnings
- [ ] Decision made: Pivot, persevere, or iterate

## Timeline
Week 4: Jan 31 - Feb 7, 2026

## Owner
John (PM), Full Team participation

## Related Documents
- `/docs/planning/user-feedback-synthesis.md`
- `/docs/planning/post-mvp-roadmap.md`

## Stories
- #[TBD] Story 4.1: Conduct User Demos
- #[TBD] Story 4.2: Synthesize Feedback and Learnings
- #[TBD] Story 4.3: Create Post-MVP Roadmap
```

**Labels**: `epic`, `week-4`, `validation`, `user-research`

---

## Issue Creation Checklist

When creating GitHub issues from these templates:

- [ ] Replace `#[TBD]` with actual issue numbers
- [ ] Assign appropriate owners from team
- [ ] Add to project board for tracking
- [ ] Link related issues and documents
- [ ] Set milestone (Week 1, 2, 3, or 4)
- [ ] Add time estimates for sprint planning
- [ ] Ensure acceptance criteria are testable

---

## Labels to Create

### Category Labels
- `epic` - Large body of work spanning multiple stories
- `story` - User story or task
- `spike` - Technical investigation/proof-of-concept
- `bug` - Something broken

### Phase Labels
- `week-1` - Discovery & Definition
- `week-2` - Architecture & Spike
- `week-3` - Implementation
- `week-4` - Validation

### Type Labels
- `product` - Product management work
- `architecture` - System design
- `implementation` - Code development
- `testing` - Test creation
- `documentation` - Docs work
- `research` - User research
- `ux` - User experience

### Priority Labels
- `critical` - Blocks progress
- `high` - Important for MVP
- `medium` - Nice to have
- `low` - Future work

---

**Next Step**: Create these issues in GitHub and populate the project board for tracking.
