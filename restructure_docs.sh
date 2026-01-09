#!/bin/bash
set -e

echo "🚀 Starting documentation restructuring..."

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p docs/planning
mkdir -p docs/progress
mkdir -p docs/development/sessions
mkdir -p docs/validation
mkdir -p docs/infrastructure

# Move files to docs/planning/
echo "📦 Moving planning files..."
git mv EPICS_AND_STORIES.md docs/planning/ 2>/dev/null || mv EPICS_AND_STORIES.md docs/planning/
git mv EPICS_INDEX.md docs/planning/ 2>/dev/null || mv EPICS_INDEX.md docs/planning/
git mv STEP-2-EPIC-DESIGN.md docs/planning/ 2>/dev/null || mv STEP-2-EPIC-DESIGN.md docs/planning/
git mv project-context.md docs/planning/ 2>/dev/null || mv project-context.md docs/planning/

# Move files to docs/progress/
echo "📊 Moving progress files..."
git mv IMPLEMENTATION_PROGRESS.md docs/progress/ 2>/dev/null || mv IMPLEMENTATION_PROGRESS.md docs/progress/
git mv STORY_1-1_IMPLEMENTATION_SUMMARY.md docs/progress/ 2>/dev/null || mv STORY_1-1_IMPLEMENTATION_SUMMARY.md docs/progress/
git mv STORY_1-2_IMPLEMENTATION_SUMMARY.md docs/progress/ 2>/dev/null || mv STORY_1-2_IMPLEMENTATION_SUMMARY.md docs/progress/
git mv STORY_1-3_IMPLEMENTATION_SUMMARY.md docs/progress/ 2>/dev/null || mv STORY_1-3_IMPLEMENTATION_SUMMARY.md docs/progress/
git mv STORY_1-3_COMPLETION.md docs/progress/ 2>/dev/null || mv STORY_1-3_COMPLETION.md docs/progress/
git mv STORY_1-3_REVIEW_REPORT.md docs/progress/ 2>/dev/null || mv STORY_1-3_REVIEW_REPORT.md docs/progress/
git mv STORY_1-4_IMPLEMENTATION_SUMMARY.md docs/progress/ 2>/dev/null || mv STORY_1-4_IMPLEMENTATION_SUMMARY.md docs/progress/
git mv STORY_1-4_REVIEW_REPORT.md docs/progress/ 2>/dev/null || mv STORY_1-4_REVIEW_REPORT.md docs/progress/
git mv STORY_1-5_IMPLEMENTATION_SUMMARY.md docs/progress/ 2>/dev/null || mv STORY_1-5_IMPLEMENTATION_SUMMARY.md docs/progress/

# Move files to docs/development/sessions/
echo "💻 Moving development session files..."
git mv DEV_SESSION_2026-01-09.md docs/development/sessions/ 2>/dev/null || mv DEV_SESSION_2026-01-09.md docs/development/sessions/
git mv RECREATION_SUMMARY.md docs/development/sessions/ 2>/dev/null || mv RECREATION_SUMMARY.md docs/development/sessions/

# Move files to docs/architecture/ (if not exists, use existing architecture dir)
echo "🏗️  Moving architecture files..."
git mv ARCHITECTURE_IMPROVEMENTS.md docs/architecture/ 2>/dev/null || mv ARCHITECTURE_IMPROVEMENTS.md docs/architecture/
git mv DEVELOPER_GUARDRAILS.md docs/architecture/ 2>/dev/null || mv DEVELOPER_GUARDRAILS.md docs/architecture/
git mv GUARDRAILS_SUMMARY.md docs/architecture/ 2>/dev/null || mv GUARDRAILS_SUMMARY.md docs/architecture/
git mv README_GUARDRAILS.md docs/architecture/ 2>/dev/null || mv README_GUARDRAILS.md docs/architecture/

# Move files to docs/validation/
echo "✅ Moving validation files..."
git mv FINAL_VALIDATION_REPORT.md docs/validation/ 2>/dev/null || mv FINAL_VALIDATION_REPORT.md docs/validation/
git mv START_HERE_VALIDATION.md docs/validation/ 2>/dev/null || mv START_HERE_VALIDATION.md docs/validation/
git mv VALIDATION_CHECKLIST.md docs/validation/ 2>/dev/null || mv VALIDATION_CHECKLIST.md docs/validation/
git mv VALIDATION_COMPLETE.md docs/validation/ 2>/dev/null || mv VALIDATION_COMPLETE.md docs/validation/
git mv VALIDATION_INDEX.md docs/validation/ 2>/dev/null || mv VALIDATION_INDEX.md docs/validation/
git mv VALIDATION_SUMMARY.md docs/validation/ 2>/dev/null || mv VALIDATION_SUMMARY.md docs/validation/

# Move files to docs/user-guides/ (if not exists, create it)
echo "📖 Moving user guide files..."
mkdir -p docs/user-guides
git mv QUICK_START.md docs/user-guides/ 2>/dev/null || mv QUICK_START.md docs/user-guides/
git mv AGENTS.md docs/user-guides/ 2>/dev/null || mv AGENTS.md docs/user-guides/

# Move files to docs/infrastructure/
echo "🔧 Moving infrastructure files..."
git mv SUPABASE_SETUP_COMPLETE.md docs/infrastructure/ 2>/dev/null || mv SUPABASE_SETUP_COMPLETE.md docs/infrastructure/

echo "✅ All files moved successfully!"

# Now update references
echo "🔄 Updating references in markdown files..."

# Update references from root to moved files
find . -type f -name "*.md" -not -path "./node_modules/*" -not -path "./target/*" -not -path "./.git/*" | while read file; do
    # Skip the files we just moved
    if [[ ! -f "$file" ]]; then
        continue
    fi
    
    # Planning files
    sed -i '' 's|EPICS_AND_STORIES\.md|docs/planning/EPICS_AND_STORIES.md|g' "$file" 2>/dev/null || sed -i 's|EPICS_AND_STORIES\.md|docs/planning/EPICS_AND_STORIES.md|g' "$file"
    sed -i '' 's|EPICS_INDEX\.md|docs/planning/EPICS_INDEX.md|g' "$file" 2>/dev/null || sed -i 's|EPICS_INDEX\.md|docs/planning/EPICS_INDEX.md|g' "$file"
    sed -i '' 's|STEP-2-EPIC-DESIGN\.md|docs/planning/STEP-2-EPIC-DESIGN.md|g' "$file" 2>/dev/null || sed -i 's|STEP-2-EPIC-DESIGN\.md|docs/planning/STEP-2-EPIC-DESIGN.md|g' "$file"
    sed -i '' 's|project-context\.md|docs/planning/project-context.md|g' "$file" 2>/dev/null || sed -i 's|project-context\.md|docs/planning/project-context.md|g' "$file"
    
    # Progress files
    sed -i '' 's|IMPLEMENTATION_PROGRESS\.md|docs/progress/IMPLEMENTATION_PROGRESS.md|g' "$file" 2>/dev/null || sed -i 's|IMPLEMENTATION_PROGRESS\.md|docs/progress/IMPLEMENTATION_PROGRESS.md|g' "$file"
    sed -i '' 's|STORY_1-1_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-1_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-1_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-1_IMPLEMENTATION_SUMMARY.md|g' "$file"
    sed -i '' 's|STORY_1-2_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-2_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-2_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-2_IMPLEMENTATION_SUMMARY.md|g' "$file"
    sed -i '' 's|STORY_1-3_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-3_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-3_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-3_IMPLEMENTATION_SUMMARY.md|g' "$file"
    sed -i '' 's|STORY_1-3_COMPLETION\.md|docs/progress/STORY_1-3_COMPLETION.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-3_COMPLETION\.md|docs/progress/STORY_1-3_COMPLETION.md|g' "$file"
    sed -i '' 's|STORY_1-3_REVIEW_REPORT\.md|docs/progress/STORY_1-3_REVIEW_REPORT.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-3_REVIEW_REPORT\.md|docs/progress/STORY_1-3_REVIEW_REPORT.md|g' "$file"
    sed -i '' 's|STORY_1-4_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-4_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-4_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-4_IMPLEMENTATION_SUMMARY.md|g' "$file"
    sed -i '' 's|STORY_1-4_REVIEW_REPORT\.md|docs/progress/STORY_1-4_REVIEW_REPORT.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-4_REVIEW_REPORT\.md|docs/progress/STORY_1-4_REVIEW_REPORT.md|g' "$file"
    sed -i '' 's|STORY_1-5_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-5_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|STORY_1-5_IMPLEMENTATION_SUMMARY\.md|docs/progress/STORY_1-5_IMPLEMENTATION_SUMMARY.md|g' "$file"
    
    # Development session files
    sed -i '' 's|DEV_SESSION_2026-01-09\.md|docs/development/sessions/DEV_SESSION_2026-01-09.md|g' "$file" 2>/dev/null || sed -i 's|DEV_SESSION_2026-01-09\.md|docs/development/sessions/DEV_SESSION_2026-01-09.md|g' "$file"
    sed -i '' 's|RECREATION_SUMMARY\.md|docs/development/sessions/RECREATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|RECREATION_SUMMARY\.md|docs/development/sessions/RECREATION_SUMMARY.md|g' "$file"
    
    # Architecture files
    sed -i '' 's|ARCHITECTURE_IMPROVEMENTS\.md|docs/architecture/ARCHITECTURE_IMPROVEMENTS.md|g' "$file" 2>/dev/null || sed -i 's|ARCHITECTURE_IMPROVEMENTS\.md|docs/architecture/ARCHITECTURE_IMPROVEMENTS.md|g' "$file"
    sed -i '' 's|DEVELOPER_GUARDRAILS\.md|docs/architecture/DEVELOPER_GUARDRAILS.md|g' "$file" 2>/dev/null || sed -i 's|DEVELOPER_GUARDRAILS\.md|docs/architecture/DEVELOPER_GUARDRAILS.md|g' "$file"
    sed -i '' 's|GUARDRAILS_SUMMARY\.md|docs/architecture/GUARDRAILS_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|GUARDRAILS_SUMMARY\.md|docs/architecture/GUARDRAILS_SUMMARY.md|g' "$file"
    sed -i '' 's|README_GUARDRAILS\.md|docs/architecture/README_GUARDRAILS.md|g' "$file" 2>/dev/null || sed -i 's|README_GUARDRAILS\.md|docs/architecture/README_GUARDRAILS.md|g' "$file"
    
    # Validation files
    sed -i '' 's|FINAL_VALIDATION_REPORT\.md|docs/validation/FINAL_VALIDATION_REPORT.md|g' "$file" 2>/dev/null || sed -i 's|FINAL_VALIDATION_REPORT\.md|docs/validation/FINAL_VALIDATION_REPORT.md|g' "$file"
    sed -i '' 's|START_HERE_VALIDATION\.md|docs/validation/START_HERE_VALIDATION.md|g' "$file" 2>/dev/null || sed -i 's|START_HERE_VALIDATION\.md|docs/validation/START_HERE_VALIDATION.md|g' "$file"
    sed -i '' 's|VALIDATION_CHECKLIST\.md|docs/validation/VALIDATION_CHECKLIST.md|g' "$file" 2>/dev/null || sed -i 's|VALIDATION_CHECKLIST\.md|docs/validation/VALIDATION_CHECKLIST.md|g' "$file"
    sed -i '' 's|VALIDATION_COMPLETE\.md|docs/validation/VALIDATION_COMPLETE.md|g' "$file" 2>/dev/null || sed -i 's|VALIDATION_COMPLETE\.md|docs/validation/VALIDATION_COMPLETE.md|g' "$file"
    sed -i '' 's|VALIDATION_INDEX\.md|docs/validation/VALIDATION_INDEX.md|g' "$file" 2>/dev/null || sed -i 's|VALIDATION_INDEX\.md|docs/validation/VALIDATION_INDEX.md|g' "$file"
    sed -i '' 's|VALIDATION_SUMMARY\.md|docs/validation/VALIDATION_SUMMARY.md|g' "$file" 2>/dev/null || sed -i 's|VALIDATION_SUMMARY\.md|docs/validation/VALIDATION_SUMMARY.md|g' "$file"
    
    # User guide files
    sed -i '' 's|QUICK_START\.md|docs/user-guides/QUICK_START.md|g' "$file" 2>/dev/null || sed -i 's|QUICK_START\.md|docs/user-guides/QUICK_START.md|g' "$file"
    sed -i '' 's|AGENTS\.md|docs/user-guides/AGENTS.md|g' "$file" 2>/dev/null || sed -i 's|AGENTS\.md|docs/user-guides/AGENTS.md|g' "$file"
    
    # Infrastructure files
    sed -i '' 's|SUPABASE_SETUP_COMPLETE\.md|docs/infrastructure/SUPABASE_SETUP_COMPLETE.md|g' "$file" 2>/dev/null || sed -i 's|SUPABASE_SETUP_COMPLETE\.md|docs/infrastructure/SUPABASE_SETUP_COMPLETE.md|g' "$file"
done

# Update references in workflow files
echo "🔄 Updating references in workflow files..."
find .github/workflows -type f -name "*.yml" | while read file; do
    sed -i '' 's|QUICK_START\.md|docs/user-guides/QUICK_START.md|g' "$file" 2>/dev/null || sed -i 's|QUICK_START\.md|docs/user-guides/QUICK_START.md|g' "$file"
done

# Update references in scripts
echo "🔄 Updating references in script files..."
find scripts -type f \( -name "*.sh" -o -name "*.py" \) | while read file; do
    # Update common references
    sed -i '' 's|VALIDATION_CHECKLIST\.md|docs/validation/VALIDATION_CHECKLIST.md|g' "$file" 2>/dev/null || sed -i 's|VALIDATION_CHECKLIST\.md|docs/validation/VALIDATION_CHECKLIST.md|g' "$file"
done

echo "✅ Documentation restructuring complete!"
echo ""
echo "📊 Summary:"
echo "  - Created 5 new directory structures"
echo "  - Moved 27 markdown files"
echo "  - Updated references in all markdown files"
echo "  - Updated references in workflow files"
echo "  - Updated references in script files"
echo ""
echo "🔍 Next steps:"
echo "  1. Review moved files: git status"
echo "  2. Update .github/copilot-instructions.md"
echo "  3. Update docs/INDEX.md"
echo "  4. Verify links work"
echo "  5. Commit changes"
