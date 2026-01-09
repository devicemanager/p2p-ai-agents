#!/bin/bash
# Master execution script - runs all steps in sequence with safety checks

set -e

echo "════════════════════════════════════════════════════════════════"
echo "  📚 Documentation Restructuring - Master Execution"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "This script will:"
echo "  1. Run pre-flight checks"
echo "  2. Execute documentation restructuring"
echo "  3. Validate all links"
echo "  4. Generate summary report"
echo ""
read -p "Press Enter to continue or Ctrl+C to abort..."
echo ""

# Step 1: Pre-flight check
echo "Step 1/4: Running pre-flight checks..."
echo "───────────────────────────────────────────────────────────────"
if [ -f "./preflight_check.sh" ]; then
    chmod +x ./preflight_check.sh
    ./preflight_check.sh
    if [ $? -ne 0 ]; then
        echo ""
        echo "❌ Pre-flight check failed. Please fix issues before continuing."
        exit 1
    fi
else
    echo "⚠️  preflight_check.sh not found, skipping..."
fi
echo ""

# Step 2: Execute restructuring
echo "Step 2/4: Executing documentation restructuring..."
echo "───────────────────────────────────────────────────────────────"
if [ -f "./execute_restructure.sh" ]; then
    chmod +x ./execute_restructure.sh
    ./execute_restructure.sh
    if [ $? -ne 0 ]; then
        echo ""
        echo "❌ Restructuring failed. Check errors above."
        exit 1
    fi
else
    echo "❌ execute_restructure.sh not found!"
    exit 1
fi
echo ""

# Step 3: Validate links
echo "Step 3/4: Validating internal links..."
echo "───────────────────────────────────────────────────────────────"
if [ -f "scripts/validate_links.py" ]; then
    if command -v python3 &> /dev/null; then
        python3 scripts/validate_links.py
        if [ $? -ne 0 ]; then
            echo ""
            echo "⚠️  Some broken links found. Review and fix before committing."
        fi
    else
        echo "⚠️  Python 3 not available, skipping link validation"
    fi
else
    echo "⚠️  Link validation script not found, skipping..."
fi
echo ""

# Step 4: Generate summary
echo "Step 4/4: Generating summary report..."
echo "───────────────────────────────────────────────────────────────"

# Count changed files
renamed_files=$(git status --porcelain | grep -c "^R" || echo "0")
modified_files=$(git status --porcelain | grep -c "^ M" || echo "0")
new_files=$(git status --porcelain | grep -c "^A" || echo "0")

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "  ✅ Documentation Restructuring Complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📊 Changes Summary:"
echo "  • Files renamed:  $renamed_files"
echo "  • Files modified: $modified_files"
echo "  • New files:      $new_files"
echo ""
echo "📁 New Directory Structure:"
echo "  docs/"
echo "  ├── planning/              (4 files - project planning)"
echo "  ├── progress/              (9 files - implementation tracking)"
echo "  ├── development/sessions/  (2 files - dev session notes)"
echo "  ├── architecture/          (4 files - architecture docs)"
echo "  ├── validation/            (6 files - validation reports)"
echo "  ├── user-guides/           (2 files - user documentation)"
echo "  └── infrastructure/        (1 file - infrastructure setup)"
echo ""
echo "🔍 Review Changes:"
echo "  git status           # See all changes"
echo "  git diff             # See modifications"
echo "  git diff --cached    # See staged changes"
echo ""
echo "📝 Next Steps:"
echo ""
echo "1. Review the changes:"
echo "   git status"
echo ""
echo "2. Update docs/INDEX.md:"
echo "   - Merge content from docs/NEW_STRUCTURE_SECTION.md"
echo "   - Add new directory structure documentation"
echo ""
echo "3. Test that everything works:"
echo "   - Check a few moved files"
echo "   - Verify links in README.md work"
echo "   - Ensure CI workflows reference correct paths"
echo ""
echo "4. Commit the changes:"
echo "   git add ."
echo "   git commit -m 'docs: restructure documentation into organized subdirectories"
echo ""
echo "   - Move 27 root-level markdown files to docs/ subdirectories"
echo "   - Organize into planning, progress, development, architecture, validation, user-guides, infrastructure"
echo "   - Update all references in markdown files, workflows, and scripts"
echo "   - Update GitHub Copilot instructions with new documentation structure"
echo "   - Add comprehensive documentation index"
echo "   - Only README.md and LICENSE remain at root level"
echo ""
echo "   Refs: DOCUMENTATION_CLEANUP_INSTRUCTIONS.md'"
echo ""
echo "5. Push to remote:"
echo "   git push origin main"
echo ""
echo "6. Clean up temporary files (optional):"
echo "   rm RESTRUCTURING_README.md"
echo "   rm RESTRUCTURE_IMPLEMENTATION_COMPLETE.md"
echo "   rm docs/NEW_STRUCTURE_SECTION.md"
echo "   rm execute_restructure.sh"
echo "   rm preflight_check.sh"
echo "   rm master_execute.sh"
echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "✅ Ready to commit! Review changes and commit when ready."
echo ""
