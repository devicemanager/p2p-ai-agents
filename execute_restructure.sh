#!/bin/bash
# Documentation Restructuring - Complete Implementation
# This script handles all file moves and reference updates

set -e

echo "════════════════════════════════════════════════════════════════"
echo "  📚 Documentation Restructuring Script"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
FILES_MOVED=0
REFERENCES_UPDATED=0

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Step 1: Create directory structure
print_status "Creating directory structure..."
mkdir -p docs/planning
mkdir -p docs/progress  
mkdir -p docs/development/sessions
mkdir -p docs/validation
mkdir -p docs/infrastructure
print_success "Directory structure created"
echo ""

# Step 2: Move files using git mv to preserve history
print_status "Moving files to new locations..."

# Planning files
if [ -f "EPICS_AND_STORIES.md" ]; then
    git mv EPICS_AND_STORIES.md docs/planning/ && ((FILES_MOVED++))
    print_success "Moved EPICS_AND_STORIES.md"
fi

if [ -f "EPICS_INDEX.md" ]; then
    git mv EPICS_INDEX.md docs/planning/ && ((FILES_MOVED++))
    print_success "Moved EPICS_INDEX.md"
fi

if [ -f "STEP-2-EPIC-DESIGN.md" ]; then
    git mv STEP-2-EPIC-DESIGN.md docs/planning/ && ((FILES_MOVED++))
    print_success "Moved STEP-2-EPIC-DESIGN.md"
fi

if [ -f "project-context.md" ]; then
    git mv project-context.md docs/planning/ && ((FILES_MOVED++))
    print_success "Moved project-context.md"
fi

# Progress files
if [ -f "IMPLEMENTATION_PROGRESS.md" ]; then
    git mv IMPLEMENTATION_PROGRESS.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved IMPLEMENTATION_PROGRESS.md"
fi

if [ -f "STORY_1-1_IMPLEMENTATION_SUMMARY.md" ]; then
    git mv STORY_1-1_IMPLEMENTATION_SUMMARY.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-1_IMPLEMENTATION_SUMMARY.md"
fi

if [ -f "STORY_1-2_IMPLEMENTATION_SUMMARY.md" ]; then
    git mv STORY_1-2_IMPLEMENTATION_SUMMARY.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-2_IMPLEMENTATION_SUMMARY.md"
fi

if [ -f "STORY_1-3_IMPLEMENTATION_SUMMARY.md" ]; then
    git mv STORY_1-3_IMPLEMENTATION_SUMMARY.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-3_IMPLEMENTATION_SUMMARY.md"
fi

if [ -f "STORY_1-3_COMPLETION.md" ]; then
    git mv STORY_1-3_COMPLETION.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-3_COMPLETION.md"
fi

if [ -f "STORY_1-3_REVIEW_REPORT.md" ]; then
    git mv STORY_1-3_REVIEW_REPORT.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-3_REVIEW_REPORT.md"
fi

if [ -f "STORY_1-4_IMPLEMENTATION_SUMMARY.md" ]; then
    git mv STORY_1-4_IMPLEMENTATION_SUMMARY.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-4_IMPLEMENTATION_SUMMARY.md"
fi

if [ -f "STORY_1-4_REVIEW_REPORT.md" ]; then
    git mv STORY_1-4_REVIEW_REPORT.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-4_REVIEW_REPORT.md"
fi

if [ -f "STORY_1-5_IMPLEMENTATION_SUMMARY.md" ]; then
    git mv STORY_1-5_IMPLEMENTATION_SUMMARY.md docs/progress/ && ((FILES_MOVED++))
    print_success "Moved STORY_1-5_IMPLEMENTATION_SUMMARY.md"
fi

# Development session files
if [ -f "DEV_SESSION_2026-01-09.md" ]; then
    git mv DEV_SESSION_2026-01-09.md docs/development/sessions/ && ((FILES_MOVED++))
    print_success "Moved DEV_SESSION_2026-01-09.md"
fi

if [ -f "RECREATION_SUMMARY.md" ]; then
    git mv RECREATION_SUMMARY.md docs/development/sessions/ && ((FILES_MOVED++))
    print_success "Moved RECREATION_SUMMARY.md"
fi

# Architecture files
if [ -f "ARCHITECTURE_IMPROVEMENTS.md" ]; then
    git mv ARCHITECTURE_IMPROVEMENTS.md docs/architecture/ && ((FILES_MOVED++))
    print_success "Moved ARCHITECTURE_IMPROVEMENTS.md"
fi

if [ -f "DEVELOPER_GUARDRAILS.md" ]; then
    git mv DEVELOPER_GUARDRAILS.md docs/architecture/ && ((FILES_MOVED++))
    print_success "Moved DEVELOPER_GUARDRAILS.md"
fi

if [ -f "GUARDRAILS_SUMMARY.md" ]; then
    git mv GUARDRAILS_SUMMARY.md docs/architecture/ && ((FILES_MOVED++))
    print_success "Moved GUARDRAILS_SUMMARY.md"
fi

if [ -f "README_GUARDRAILS.md" ]; then
    git mv README_GUARDRAILS.md docs/architecture/ && ((FILES_MOVED++))
    print_success "Moved README_GUARDRAILS.md"
fi

# Validation files
if [ -f "FINAL_VALIDATION_REPORT.md" ]; then
    git mv FINAL_VALIDATION_REPORT.md docs/validation/ && ((FILES_MOVED++))
    print_success "Moved FINAL_VALIDATION_REPORT.md"
fi

if [ -f "START_HERE_VALIDATION.md" ]; then
    git mv START_HERE_VALIDATION.md docs/validation/ && ((FILES_MOVED++))
    print_success "Moved START_HERE_VALIDATION.md"
fi

if [ -f "VALIDATION_CHECKLIST.md" ]; then
    git mv VALIDATION_CHECKLIST.md docs/validation/ && ((FILES_MOVED++))
    print_success "Moved VALIDATION_CHECKLIST.md"
fi

if [ -f "VALIDATION_COMPLETE.md" ]; then
    git mv VALIDATION_COMPLETE.md docs/validation/ && ((FILES_MOVED++))
    print_success "Moved VALIDATION_COMPLETE.md"
fi

if [ -f "VALIDATION_INDEX.md" ]; then
    git mv VALIDATION_INDEX.md docs/validation/ && ((FILES_MOVED++))
    print_success "Moved VALIDATION_INDEX.md"
fi

if [ -f "VALIDATION_SUMMARY.md" ]; then
    git mv VALIDATION_SUMMARY.md docs/validation/ && ((FILES_MOVED++))
    print_success "Moved VALIDATION_SUMMARY.md"
fi

# User guide files
mkdir -p docs/user-guides
if [ -f "QUICK_START.md" ]; then
    git mv QUICK_START.md docs/user-guides/ && ((FILES_MOVED++))
    print_success "Moved QUICK_START.md"
fi

if [ -f "AGENTS.md" ]; then
    git mv AGENTS.md docs/user-guides/ && ((FILES_MOVED++))
    print_success "Moved AGENTS.md"
fi

# Infrastructure files
if [ -f "SUPABASE_SETUP_COMPLETE.md" ]; then
    git mv SUPABASE_SETUP_COMPLETE.md docs/infrastructure/ && ((FILES_MOVED++))
    print_success "Moved SUPABASE_SETUP_COMPLETE.md"
fi

echo ""
print_success "Moved $FILES_MOVED files"
echo ""

# Step 3: Update all references
print_status "Updating references in markdown files..."

# Function to update references in a file
update_file_references() {
    local file="$1"
    local updated=0
    
    # Skip if file doesn't exist
    [ ! -f "$file" ] && return 0
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Planning files
    if grep -q "EPICS_AND_STORIES\.md" "$file" 2>/dev/null; then
        sed -i 's|\([^/]\)EPICS_AND_STORIES\.md|\1docs/planning/EPICS_AND_STORIES.md|g' "$file"
        ((updated++))
    fi
    
    if grep -q "EPICS_INDEX\.md" "$file" 2>/dev/null; then
        sed -i 's|\([^/]\)EPICS_INDEX\.md|\1docs/planning/EPICS_INDEX.md|g' "$file"
        ((updated++))
    fi
    
    if grep -q "STEP-2-EPIC-DESIGN\.md" "$file" 2>/dev/null; then
        sed -i 's|\([^/]\)STEP-2-EPIC-DESIGN\.md|\1docs/planning/STEP-2-EPIC-DESIGN.md|g' "$file"
        ((updated++))
    fi
    
    if grep -q "project-context\.md" "$file" 2>/dev/null; then
        sed -i 's|\([^/]\)project-context\.md|\1docs/planning/project-context.md|g' "$file"
        ((updated++))
    fi
    
    # Progress files  
    sed -i 's|\([^/]\)IMPLEMENTATION_PROGRESS\.md|\1docs/progress/IMPLEMENTATION_PROGRESS.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-1_IMPLEMENTATION_SUMMARY\.md|\1docs/progress/STORY_1-1_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-2_IMPLEMENTATION_SUMMARY\.md|\1docs/progress/STORY_1-2_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-3_IMPLEMENTATION_SUMMARY\.md|\1docs/progress/STORY_1-3_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-3_COMPLETION\.md|\1docs/progress/STORY_1-3_COMPLETION.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-3_REVIEW_REPORT\.md|\1docs/progress/STORY_1-3_REVIEW_REPORT.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-4_IMPLEMENTATION_SUMMARY\.md|\1docs/progress/STORY_1-4_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-4_REVIEW_REPORT\.md|\1docs/progress/STORY_1-4_REVIEW_REPORT.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)STORY_1-5_IMPLEMENTATION_SUMMARY\.md|\1docs/progress/STORY_1-5_IMPLEMENTATION_SUMMARY.md|g' "$file" 2>/dev/null
    
    # Development files
    sed -i 's|\([^/]\)DEV_SESSION_2026-01-09\.md|\1docs/development/sessions/DEV_SESSION_2026-01-09.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)RECREATION_SUMMARY\.md|\1docs/development/sessions/RECREATION_SUMMARY.md|g' "$file" 2>/dev/null
    
    # Architecture files
    sed -i 's|\([^/]\)ARCHITECTURE_IMPROVEMENTS\.md|\1docs/architecture/ARCHITECTURE_IMPROVEMENTS.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)DEVELOPER_GUARDRAILS\.md|\1docs/architecture/DEVELOPER_GUARDRAILS.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)GUARDRAILS_SUMMARY\.md|\1docs/architecture/GUARDRAILS_SUMMARY.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)README_GUARDRAILS\.md|\1docs/architecture/README_GUARDRAILS.md|g' "$file" 2>/dev/null
    
    # Validation files
    sed -i 's|\([^/]\)FINAL_VALIDATION_REPORT\.md|\1docs/validation/FINAL_VALIDATION_REPORT.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)START_HERE_VALIDATION\.md|\1docs/validation/START_HERE_VALIDATION.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)VALIDATION_CHECKLIST\.md|\1docs/validation/VALIDATION_CHECKLIST.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)VALIDATION_COMPLETE\.md|\1docs/validation/VALIDATION_COMPLETE.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)VALIDATION_INDEX\.md|\1docs/validation/VALIDATION_INDEX.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)VALIDATION_SUMMARY\.md|\1docs/validation/VALIDATION_SUMMARY.md|g' "$file" 2>/dev/null
    
    # User guide files
    sed -i 's|\([^/]\)QUICK_START\.md|\1docs/user-guides/QUICK_START.md|g' "$file" 2>/dev/null
    sed -i 's|\([^/]\)AGENTS\.md|\1docs/user-guides/AGENTS.md|g' "$file" 2>/dev/null
    
    # Infrastructure files
    sed -i 's|\([^/]\)SUPABASE_SETUP_COMPLETE\.md|\1docs/infrastructure/SUPABASE_SETUP_COMPLETE.md|g' "$file" 2>/dev/null
    
    # Fix double docs/ paths (in case file is in docs/)
    sed -i 's|docs/docs/|docs/|g' "$file" 2>/dev/null
    
    # Remove backup if no changes
    if diff -q "$file" "$file.bak" >/dev/null 2>&1; then
        rm "$file.bak"
    else
        rm "$file.bak"
        return 1
    fi
    
    return 0
}

# Update all markdown files
find . -type f -name "*.md" \
    -not -path "./node_modules/*" \
    -not -path "./target/*" \
    -not -path "./.git/*" \
    -not -path "./_bmad-output/*" | while read file; do
    if update_file_references "$file"; then
        ((REFERENCES_UPDATED++))
        echo "  ✓ Updated: $file"
    fi
done

echo ""
print_success "Updated references in markdown files"
echo ""

# Step 4: Update workflow files
print_status "Updating GitHub workflow files..."
find .github/workflows -type f -name "*.yml" 2>/dev/null | while read file; do
    if grep -q "QUICK_START\.md\|VALIDATION" "$file" 2>/dev/null; then
        sed -i 's|QUICK_START\.md|docs/user-guides/QUICK_START.md|g' "$file" 2>/dev/null
        print_success "Updated: $file"
    fi
done
echo ""

# Step 5: Update script files
print_status "Updating script files..."
find scripts -type f \( -name "*.sh" -o -name "*.py" \) 2>/dev/null | while read file; do
    if grep -q "VALIDATION_CHECKLIST\.md" "$file" 2>/dev/null; then
        sed -i 's|VALIDATION_CHECKLIST\.md|docs/validation/VALIDATION_CHECKLIST.md|g' "$file" 2>/dev/null
        print_success "Updated: $file"
    fi
done
echo ""

# Step 6: Summary
echo "════════════════════════════════════════════════════════════════"
echo "  ✅ Documentation Restructuring Complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📊 Summary:"
echo "  • Files moved: $FILES_MOVED"
echo "  • References updated: $REFERENCES_UPDATED+"
echo "  • Directories created: 5"
echo ""
echo "📁 New Structure:"
echo "  docs/"
echo "  ├── planning/              (4 files)"
echo "  ├── progress/              (9 files)"
echo "  ├── development/sessions/  (2 files)"
echo "  ├── architecture/          (4 files)"
echo "  ├── validation/            (6 files)"
echo "  ├── user-guides/           (2 files)"
echo "  └── infrastructure/        (1 file)"
echo ""
echo "🔍 Next Steps:"
echo "  1. Review changes:         git status"
echo "  2. Update Copilot config:  .github/copilot-instructions.md"
echo "  3. Update documentation:   docs/INDEX.md"
echo "  4. Verify links:           Check all markdown links work"
echo "  5. Commit changes:         git add . && git commit -m 'docs: restructure documentation'"
echo ""
print_warning "Manual updates still needed:"
echo "  • .github/copilot-instructions.md - Update documentation references"
echo "  • docs/INDEX.md - Add new directory structure"
echo ""
echo "════════════════════════════════════════════════════════════════"
