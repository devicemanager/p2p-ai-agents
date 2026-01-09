# Documentation Index Update

## New Directory Structure (January 2026)

All root-level documentation has been moved to organized subdirectories within `docs/`.

### 📁 Planning Documents (`docs/planning/`)
- [EPICS_AND_STORIES.md](./planning/EPICS_AND_STORIES.md) - User stories and acceptance criteria
- [EPICS_INDEX.md](./planning/EPICS_INDEX.md) - Epic navigation and index
- [STEP-2-EPIC-DESIGN.md](./planning/STEP-2-EPIC-DESIGN.md) - Epic design documentation
- [project-context.md](./planning/project-context.md) - Project context and background

### 📊 Progress Tracking (`docs/progress/`)
- [IMPLEMENTATION_PROGRESS.md](./progress/IMPLEMENTATION_PROGRESS.md) - Overall implementation status
- [STORY_1-1_IMPLEMENTATION_SUMMARY.md](./progress/STORY_1-1_IMPLEMENTATION_SUMMARY.md) - Story 1.1 summary
- [STORY_1-2_IMPLEMENTATION_SUMMARY.md](./progress/STORY_1-2_IMPLEMENTATION_SUMMARY.md) - Story 1.2 summary
- [STORY_1-3_IMPLEMENTATION_SUMMARY.md](./progress/STORY_1-3_IMPLEMENTATION_SUMMARY.md) - Story 1.3 summary
- [STORY_1-3_COMPLETION.md](./progress/STORY_1-3_COMPLETION.md) - Story 1.3 completion report
- [STORY_1-3_REVIEW_REPORT.md](./progress/STORY_1-3_REVIEW_REPORT.md) - Story 1.3 review
- [STORY_1-4_IMPLEMENTATION_SUMMARY.md](./progress/STORY_1-4_IMPLEMENTATION_SUMMARY.md) - Story 1.4 summary
- [STORY_1-4_REVIEW_REPORT.md](./progress/STORY_1-4_REVIEW_REPORT.md) - Story 1.4 review
- [STORY_1-5_IMPLEMENTATION_SUMMARY.md](./progress/STORY_1-5_IMPLEMENTATION_SUMMARY.md) - Story 1.5 summary

### 💻 Development Sessions (`docs/development/sessions/`)
- [DEV_SESSION_2026-01-09.md](./development/sessions/DEV_SESSION_2026-01-09.md) - Development session Jan 9, 2026
- [RECREATION_SUMMARY.md](./development/sessions/RECREATION_SUMMARY.md) - Project recreation summary

### 🏗️ Architecture & Guidelines (`docs/architecture/`)
- [ARCHITECTURE_IMPROVEMENTS.md](./architecture/ARCHITECTURE_IMPROVEMENTS.md) - Architecture improvement proposals
- [DEVELOPER_GUARDRAILS.md](./architecture/DEVELOPER_GUARDRAILS.md) - Developer guidelines and guardrails
- [GUARDRAILS_SUMMARY.md](./architecture/GUARDRAILS_SUMMARY.md) - Summary of guardrails
- [README_GUARDRAILS.md](./architecture/README_GUARDRAILS.md) - Guardrails documentation

### ✅ Validation Reports (`docs/validation/`)
- [FINAL_VALIDATION_REPORT.md](./validation/FINAL_VALIDATION_REPORT.md) - Final validation report
- [START_HERE_VALIDATION.md](./validation/START_HERE_VALIDATION.md) - Validation starting point
- [VALIDATION_CHECKLIST.md](./validation/VALIDATION_CHECKLIST.md) - Validation checklist
- [VALIDATION_COMPLETE.md](./validation/VALIDATION_COMPLETE.md) - Validation completion report
- [VALIDATION_INDEX.md](./validation/VALIDATION_INDEX.md) - Validation documentation index
- [VALIDATION_SUMMARY.md](./validation/VALIDATION_SUMMARY.md) - Validation summary

### 📖 User Guides (`docs/user-guides/`)
- [QUICK_START.md](./user-guides/QUICK_START.md) - Quick start guide for new users
- [AGENTS.md](./user-guides/AGENTS.md) - Agent configuration and usage

### 🔧 Infrastructure (`docs/infrastructure/`)
- [SUPABASE_SETUP_COMPLETE.md](./infrastructure/SUPABASE_SETUP_COMPLETE.md) - Supabase setup documentation

---

## Documentation Standards

### File Placement Rules

1. **Root Level**: Only `README.md` and `LICENSE` remain at repository root
2. **All Documentation**: Must be placed in appropriate `docs/` subdirectory
3. **Organized Categories**: Use the directory structure above for proper categorization

### Linking to Documentation

When referencing documentation from root directory:
```markdown
[Epics](docs/planning/EPICS_AND_STORIES.md)
[Quick Start](docs/user-guides/QUICK_START.md)
```

When referencing from within `docs/`:
```markdown
[Epics](./planning/EPICS_AND_STORIES.md)
[Quick Start](./user-guides/QUICK_START.md)
```

### Naming Conventions

- Use lowercase with hyphens: `my-document.md`
- Session notes: `session-YYYY-MM-DD.md`
- Be descriptive: `storage-consistency-model.md` not `storage.md`
- Use prefixes for related docs: `story-1-1-*.md`

---

*Documentation restructured: January 9, 2026*
*All references updated across markdown, workflows, and scripts*
