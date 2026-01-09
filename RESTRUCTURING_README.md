# Documentation Restructuring - Execution Guide

This directory contains all the tools needed to complete the documentation restructuring task.

## 📋 What This Does

Moves 27 root-level markdown files into organized subdirectories within `docs/` and updates all references across the codebase.

## 🚀 Quick Start

### Option 1: Automated Execution (Recommended)

```bash
# 1. Make scripts executable
chmod +x preflight_check.sh execute_restructure.sh

# 2. Run pre-flight check
./preflight_check.sh

# 3. Execute restructuring
./execute_restructure.sh

# 4. Validate links
python3 scripts/validate_links.py

# 5. Review and commit
git status
git add .
git commit -m "docs: restructure documentation into organized subdirectories"
```

### Option 2: Manual Review First

```bash
# 1. Review what will be moved
./preflight_check.sh

# 2. Review the implementation summary
cat RESTRUCTURE_IMPLEMENTATION_COMPLETE.md

# 3. Proceed with execution
./execute_restructure.sh
```

## 📁 Files Created

| File | Purpose |
|------|---------|
| `execute_restructure.sh` | Main execution script - moves files and updates references |
| `preflight_check.sh` | Pre-flight verification - checks everything is ready |
| `scripts/validate_links.py` | Link validator - checks for broken references |
| `docs/NEW_STRUCTURE_SECTION.md` | New content for docs/INDEX.md |
| `RESTRUCTURE_IMPLEMENTATION_COMPLETE.md` | Complete implementation summary and guide |
| `RESTRUCTURING_README.md` | This file |

## 📊 What Gets Changed

### Files Moved (27 total)

```
Root Level                               → New Location
────────────────────────────────────────────────────────────────
EPICS_AND_STORIES.md                    → docs/planning/
EPICS_INDEX.md                          → docs/planning/
STEP-2-EPIC-DESIGN.md                   → docs/planning/
project-context.md                      → docs/planning/

IMPLEMENTATION_PROGRESS.md              → docs/progress/
STORY_1-1_IMPLEMENTATION_SUMMARY.md     → docs/progress/
STORY_1-2_IMPLEMENTATION_SUMMARY.md     → docs/progress/
STORY_1-3_IMPLEMENTATION_SUMMARY.md     → docs/progress/
STORY_1-3_COMPLETION.md                 → docs/progress/
STORY_1-3_REVIEW_REPORT.md              → docs/progress/
STORY_1-4_IMPLEMENTATION_SUMMARY.md     → docs/progress/
STORY_1-4_REVIEW_REPORT.md              → docs/progress/
STORY_1-5_IMPLEMENTATION_SUMMARY.md     → docs/progress/

DEV_SESSION_2026-01-09.md               → docs/development/sessions/
RECREATION_SUMMARY.md                   → docs/development/sessions/

ARCHITECTURE_IMPROVEMENTS.md            → docs/architecture/
DEVELOPER_GUARDRAILS.md                 → docs/architecture/
GUARDRAILS_SUMMARY.md                   → docs/architecture/
README_GUARDRAILS.md                    → docs/architecture/

FINAL_VALIDATION_REPORT.md              → docs/validation/
START_HERE_VALIDATION.md                → docs/validation/
VALIDATION_CHECKLIST.md                 → docs/validation/
VALIDATION_COMPLETE.md                  → docs/validation/
VALIDATION_INDEX.md                     → docs/validation/
VALIDATION_SUMMARY.md                   → docs/validation/

QUICK_START.md                          → docs/user-guides/
AGENTS.md                               → docs/user-guides/

SUPABASE_SETUP_COMPLETE.md              → docs/infrastructure/
```

### References Updated

- All `.md` files in repository
- `.github/workflows/*.yml` files
- `scripts/*.sh` and `scripts/*.py` files
- Updated `.github/copilot-instructions.md`

## ✅ Verification Steps

After running the scripts:

1. **Check file moves**:
   ```bash
   ls docs/planning/
   ls docs/progress/
   ls docs/development/sessions/
   ls docs/architecture/
   ls docs/validation/
   ls docs/user-guides/
   ls docs/infrastructure/
   ```

2. **Verify no broken links**:
   ```bash
   python3 scripts/validate_links.py
   ```

3. **Check git status**:
   ```bash
   git status
   # Should show 27 renames and various modifications
   ```

4. **Review changes**:
   ```bash
   git diff README.md
   git diff .github/copilot-instructions.md
   ```

## 🔄 If Something Goes Wrong

### Rollback Everything

```bash
git reset --hard HEAD
```

### Rollback Specific File

```bash
git checkout HEAD -- <file>
```

### Re-run After Fixing Issues

The scripts are idempotent - you can run them multiple times safely.

## 📝 After Execution

1. **Update docs/INDEX.md**:
   - Merge content from `docs/NEW_STRUCTURE_SECTION.md` into `docs/INDEX.md`

2. **Clean up**:
   ```bash
   # Remove temporary files if desired
   rm RESTRUCTURING_README.md
   rm RESTRUCTURE_IMPLEMENTATION_COMPLETE.md
   rm docs/NEW_STRUCTURE_SECTION.md
   rm execute_restructure.sh
   rm preflight_check.sh
   rm restructure_docs.sh  # Old version if exists
   ```

3. **Commit**:
   ```bash
   git add .
   git commit -m "docs: restructure documentation into organized subdirectories

   - Move 27 root-level markdown files to docs/ subdirectories
   - Organize into planning, progress, development, architecture, validation, user-guides, infrastructure
   - Update all references in markdown files, workflows, and scripts
   - Update GitHub Copilot instructions with new documentation structure
   - Add comprehensive documentation index
   - Only README.md and LICENSE remain at root level

   Refs: DOCUMENTATION_CLEANUP_INSTRUCTIONS.md"
   ```

## 🎯 Success Criteria

- [ ] All 27 files moved to correct locations
- [ ] All references updated correctly
- [ ] No broken links (validate_links.py passes)
- [ ] GitHub workflows still work
- [ ] Scripts reference correct paths
- [ ] Copilot instructions updated
- [ ] Documentation index updated
- [ ] Git history preserved (files moved with git mv)

## 📚 Additional Documentation

- See `RESTRUCTURE_IMPLEMENTATION_COMPLETE.md` for complete implementation details
- See `DOCUMENTATION_CLEANUP_INSTRUCTIONS.md` for original requirements
- See `docs/NEW_STRUCTURE_SECTION.md` for documentation index content

## 🆘 Need Help?

- Check `RESTRUCTURE_IMPLEMENTATION_COMPLETE.md` for detailed information
- Review the scripts - they have extensive comments
- Run `./preflight_check.sh` to verify your environment
- All scripts provide detailed progress output

---

**Ready to begin!** Start with: `./preflight_check.sh`
