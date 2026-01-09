#!/bin/bash
# Pre-flight check for documentation restructuring

echo "════════════════════════════════════════════════════════════════"
echo "  🔍 Documentation Restructuring Pre-flight Check"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must be run from repository root"
    exit 1
fi

echo "✅ Running from repository root"
echo ""

# Count files to be moved
echo "📊 Files to be moved:"
echo ""

count=0

# Planning files
planning_files=(
    "EPICS_AND_STORIES.md"
    "EPICS_INDEX.md"
    "STEP-2-EPIC-DESIGN.md"
    "project-context.md"
)
planning_count=0
for file in "${planning_files[@]}"; do
    if [ -f "$file" ]; then
        ((planning_count++))
        ((count++))
    fi
done
echo "  Planning:      $planning_count files"

# Progress files
progress_files=(
    "IMPLEMENTATION_PROGRESS.md"
    "STORY_1-1_IMPLEMENTATION_SUMMARY.md"
    "STORY_1-2_IMPLEMENTATION_SUMMARY.md"
    "STORY_1-3_IMPLEMENTATION_SUMMARY.md"
    "STORY_1-3_COMPLETION.md"
    "STORY_1-3_REVIEW_REPORT.md"
    "STORY_1-4_IMPLEMENTATION_SUMMARY.md"
    "STORY_1-4_REVIEW_REPORT.md"
    "STORY_1-5_IMPLEMENTATION_SUMMARY.md"
)
progress_count=0
for file in "${progress_files[@]}"; do
    if [ -f "$file" ]; then
        ((progress_count++))
        ((count++))
    fi
done
echo "  Progress:      $progress_count files"

# Development files
dev_files=(
    "DEV_SESSION_2026-01-09.md"
    "RECREATION_SUMMARY.md"
)
dev_count=0
for file in "${dev_files[@]}"; do
    if [ -f "$file" ]; then
        ((dev_count++))
        ((count++))
    fi
done
echo "  Development:   $dev_count files"

# Architecture files
arch_files=(
    "ARCHITECTURE_IMPROVEMENTS.md"
    "DEVELOPER_GUARDRAILS.md"
    "GUARDRAILS_SUMMARY.md"
    "README_GUARDRAILS.md"
)
arch_count=0
for file in "${arch_files[@]}"; do
    if [ -f "$file" ]; then
        ((arch_count++))
        ((count++))
    fi
done
echo "  Architecture:  $arch_count files"

# Validation files
val_files=(
    "FINAL_VALIDATION_REPORT.md"
    "START_HERE_VALIDATION.md"
    "VALIDATION_CHECKLIST.md"
    "VALIDATION_COMPLETE.md"
    "VALIDATION_INDEX.md"
    "VALIDATION_SUMMARY.md"
)
val_count=0
for file in "${val_files[@]}"; do
    if [ -f "$file" ]; then
        ((val_count++))
        ((count++))
    fi
done
echo "  Validation:    $val_count files"

# User guide files
guide_files=(
    "QUICK_START.md"
    "AGENTS.md"
)
guide_count=0
for file in "${guide_files[@]}"; do
    if [ -f "$file" ]; then
        ((guide_count++))
        ((count++))
    fi
done
echo "  User Guides:   $guide_count files"

# Infrastructure files
infra_files=(
    "SUPABASE_SETUP_COMPLETE.md"
)
infra_count=0
for file in "${infra_files[@]}"; do
    if [ -f "$file" ]; then
        ((infra_count++))
        ((count++))
    fi
done
echo "  Infrastructure: $infra_count files"

echo ""
echo "  Total:         $count files"
echo ""

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo "   Consider committing or stashing before restructuring"
    echo ""
fi

# Check if restructure script exists
if [ ! -f "execute_restructure.sh" ]; then
    echo "❌ Error: execute_restructure.sh not found"
    exit 1
fi

echo "✅ Restructure script found"

# Check if validation script exists
if [ ! -f "scripts/validate_links.py" ]; then
    echo "⚠️  Warning: Link validation script not found"
else
    echo "✅ Link validation script found"
fi

# Check if docs directory exists
if [ ! -d "docs" ]; then
    echo "❌ Error: docs/ directory not found"
    exit 1
fi

echo "✅ docs/ directory exists"
echo ""

# Check Python availability
if command -v python3 &> /dev/null; then
    echo "✅ Python 3 available for validation"
else
    echo "⚠️  Warning: Python 3 not found - link validation may not work"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "  ✅ Pre-flight Check Complete"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Ready to execute restructuring!"
echo ""
echo "Next steps:"
echo "  1. chmod +x execute_restructure.sh"
echo "  2. ./execute_restructure.sh"
echo "  3. python3 scripts/validate_links.py"
echo "  4. Review changes with: git status && git diff"
echo "  5. Commit: git add . && git commit -m 'docs: restructure documentation'"
echo ""
