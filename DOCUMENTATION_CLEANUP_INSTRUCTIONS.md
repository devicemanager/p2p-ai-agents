# Documentation Cleanup and Restructuring Instructions

**Date**: 2026-01-09  
**Purpose**: Move all root-level `.md` files to the `docs/` folder and update all references

## Objective

Consolidate all documentation into the `docs/` directory for better organization. Only `README.md` and `LICENSE` should remain at the repository root.

## Files to Move

The following root-level markdown files need to be moved to appropriate locations in `docs/`:

### Project Planning & Epics (→ `docs/planning/`)
- `EPICS_AND_STORIES.md`
- `EPICS_INDEX.md`
- `STEP-2-EPIC-DESIGN.md`
- `project-context.md`

### Implementation Progress (→ `docs/progress/`)
- `IMPLEMENTATION_PROGRESS.md`
- `STORY_1-1_IMPLEMENTATION_SUMMARY.md`
- `STORY_1-2_IMPLEMENTATION_SUMMARY.md`
- `STORY_1-3_IMPLEMENTATION_SUMMARY.md`
- `STORY_1-3_COMPLETION.md`
- `STORY_1-3_REVIEW_REPORT.md`
- `STORY_1-4_IMPLEMENTATION_SUMMARY.md`
- `STORY_1-4_REVIEW_REPORT.md`
- `STORY_1-5_IMPLEMENTATION_SUMMARY.md`

### Development Session Notes (→ `docs/development/sessions/`)
- `DEV_SESSION_2026-01-09.md`
- `RECREATION_SUMMARY.md`

### Architecture & Guidelines (→ `docs/architecture/`)
- `ARCHITECTURE_IMPROVEMENTS.md`
- `DEVELOPER_GUARDRAILS.md`
- `GUARDRAILS_SUMMARY.md`
- `README_GUARDRAILS.md`

### Validation Reports (→ `docs/validation/`)
- `FINAL_VALIDATION_REPORT.md`
- `START_HERE_VALIDATION.md`
- `VALIDATION_CHECKLIST.md`
- `VALIDATION_COMPLETE.md`
- `VALIDATION_INDEX.md`
- `VALIDATION_SUMMARY.md`

### User Guides (→ `docs/user-guides/`)
- `QUICK_START.md`
- `AGENTS.md`

### Infrastructure Setup (→ `docs/infrastructure/`)
- `SUPABASE_SETUP_COMPLETE.md`

## Tasks

### 1. Create Directory Structure

```bash
mkdir -p docs/planning
mkdir -p docs/progress
mkdir -p docs/development/sessions
mkdir -p docs/validation
mkdir -p docs/infrastructure
```

### 2. Move Files

For each file listed above:
1. Move the file to its designated directory
2. Track the old path and new path for reference updates

### 3. Update All References

Search for references to moved files in:
- All remaining `.md` files in root and `docs/`
- All `.md` files in subdirectories
- GitHub workflow files (`.github/workflows/*.yml`)
- Configuration files (`*.yaml`, `*.toml`, `*.json`)
- Script files (`scripts/*.sh`, `scripts/*.py`)
- Source code comments that reference documentation

Update patterns to look for:
- Markdown links: `[text](FILE.md)` → `[text](docs/category/FILE.md)`
- Relative links from `docs/`: `../FILE.md` → `category/FILE.md`
- HTML links: `<a href="FILE.md">` → `<a href="docs/category/FILE.md">`
- Script references: `cat FILE.md` → `cat docs/category/FILE.md`

### 4. Update GitHub Copilot Instructions

Update `.github/copilot-instructions.md` to specify:
- All documentation files must be placed in `docs/`
- Use appropriate subdirectories for organization
- Update the "Documentation References" section

### 5. Update Documentation Index

Update `docs/INDEX.md` with the new structure and file locations.

### 6. Verify Links

After moving and updating:
1. Run link checker: `./scripts/validate_docs.py` (if exists)
2. Manually verify key documentation links work
3. Check GitHub renders links correctly

## Reference Update Script Pattern

```bash
# Example: Update references to QUICK_START.md
find . -type f \( -name "*.md" -o -name "*.yml" -o -name "*.yaml" \) \
  -exec sed -i '' 's|QUICK_START\.md|docs/user-guides/QUICK_START.md|g' {} +

# For references from within docs/ directory
find docs -type f -name "*.md" \
  -exec sed -i '' 's|\.\./QUICK_START\.md|user-guides/QUICK_START.md|g' {} +
```

## Files to Keep at Root

These files should **NOT** be moved:
- `README.md` - Repository entry point
- `LICENSE` - Legal requirement
- `CHANGELOG.md` - Optional, but commonly at root
- `.gitignore`, `.dockerignore`, etc. - Configuration files
- `Cargo.toml`, `package.json` - Project manifests
- `Makefile`, `Dockerfile` - Build files

## Validation Checklist

After completion:
- [ ] All root-level `.md` files (except README.md) are moved
- [ ] Directory structure created in `docs/`
- [ ] All file references updated
- [ ] GitHub Copilot instructions updated
- [ ] Documentation index updated
- [ ] Links verified (no broken links)
- [ ] GitHub renders all links correctly
- [ ] CI/CD workflows still work
- [ ] Scripts that reference docs still work

## Future Documentation Guidelines

**All new documentation must be created in the `docs/` directory.**

### Directory Purpose

- `docs/architecture/` - Architecture decisions, patterns, design docs
- `docs/core/` - Core system documentation
- `docs/development/` - Developer guides, setup, workflows
- `docs/implementation/` - Implementation-specific docs
- `docs/planning/` - Epics, stories, project planning
- `docs/progress/` - Implementation progress tracking
- `docs/user-guides/` - End-user documentation
- `docs/validation/` - Validation reports and checklists
- `docs/infrastructure/` - Infrastructure setup and configuration
- `docs/development/sessions/` - Development session notes

### Naming Conventions

- Use lowercase with hyphens: `my-document.md`
- Be descriptive: `storage-consistency-model.md` not `storage.md`
- Include dates for session notes: `session-2026-01-09.md`
- Use prefixes for related docs: `prd-section-7-*.md`

## Implementation Notes

This is a **one-time restructuring task**. Once complete:
1. Document the new structure in `docs/README.md`
2. Add a note in root `README.md` pointing to `docs/`
3. Update contributing guidelines with documentation location rules
4. Consider adding a git pre-commit hook to prevent root-level `.md` files

---

**Assigned to**: AI Developer Agent  
**Priority**: Medium  
**Estimated effort**: 2-3 hours (automated with validation)
