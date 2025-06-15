#!/bin/bash
# Local Documentation Management Script
# Run documentation checks and maintenance tasks locally

set -e

PROJECT_ROOT="/workspaces/p2p-ai-agents"
DOCS_DIR="$PROJECT_ROOT/docs"

# Colors for output
RED='\03    "stats"|"stat")
        show_stats
        ;;
    "policy"|"500"|"limit")
        check_500_line_limit
        ;;
    "all"|"")
        run_all
        ;;
    "help"|"-h"|"--help")
        echo "Documentation Management Script"
        echo
        echo "Usage: $0 [command]"
        echo
        echo "Commands:"
        echo "  all        Run all checks (default)"
        echo "  validate   Run comprehensive validation"
        echo "  todos      Check for TODO/FIXME items"
        echo "  structure  Show documentation structure"
        echo "  links      Quick broken link check"
        echo "  format     Check formatting issues"
        echo "  stats      Show documentation statistics"
        echo "  policy     Check 500-line limit policy"
        echo "  help       Show this help message"[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}📚 $1${NC}"
    echo "=================================================="
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to run documentation validation
run_validation() {
    print_header "Running Documentation Validation"
    cd "$PROJECT_ROOT"
    
    if python3 scripts/validate_docs.py; then
        print_success "Documentation validation completed"
    else
        print_warning "Documentation validation found issues (non-blocking in development mode)"
    fi
    echo
}

# Function to check for TODO/FIXME items
check_todos() {
    print_header "Checking for TODO/FIXME Items"
    cd "$PROJECT_ROOT"
    
    if grep -r "TODO\|FIXME" docs/ --include="*.md" > /tmp/todos.txt 2>/dev/null; then
        echo "Found TODO/FIXME items:"
        cat /tmp/todos.txt
        print_warning "$(wc -l < /tmp/todos.txt) TODO/FIXME items found"
    else
        print_success "No TODO/FIXME items found"
    fi
    echo
}

# Function to check documentation structure
check_structure() {
    print_header "Checking Documentation Structure"
    cd "$PROJECT_ROOT"
    
    echo "📁 Documentation Structure:"
    find docs -type f -name "*.md" | sort | sed 's|^docs/|  |'
    echo
    
    total_files=$(find docs -name "*.md" | wc -l)
    total_lines=$(find docs -name "*.md" -exec cat {} \; | wc -l)
    
    print_success "Total files: $total_files"
    print_success "Total lines: $total_lines"
    echo
}

# Function to check for broken links (simple version)
check_links_simple() {
    print_header "Quick Link Check"
    cd "$PROJECT_ROOT"
    
    echo "🔗 Checking for obvious broken links..."
    
    # Check for common broken patterns
    broken_count=0
    
    # Check for non-existent .md files
    find docs -name "*.md" -exec grep -l "](.*\.md)" {} \; | while read file; do
        grep -o "](.*\.md)" "$file" | sed 's/^](//' | sed 's/)$//' | while read link; do
            if [[ "$link" != http* && "$link" != "#"* ]]; then
                file_dir=$(dirname "$file")
                if [[ "$link" == ../* ]]; then
                    target_path=$(realpath "$file_dir/$link" 2>/dev/null || echo "invalid")
                else
                    target_path="$file_dir/$link"
                fi
                
                if [[ ! -f "$target_path" && "$target_path" != "invalid" ]]; then
                    echo "  ❌ $file: $link -> $target_path"
                    ((broken_count++)) || true
                fi
            fi
        done
    done
    
    if [ $broken_count -eq 0 ]; then
        print_success "No obviously broken links found"
    else
        print_warning "$broken_count potentially broken links found"
    fi
    echo
}

# Function to format documentation
format_docs() {
    print_header "Formatting Documentation"
    cd "$PROJECT_ROOT"
    
    echo "📝 Basic formatting checks..."
    
    # Check for trailing whitespace
    if find docs -name "*.md" -exec grep -l " $" {} \; 2>/dev/null; then
        print_warning "Files with trailing whitespace found"
    else
        print_success "No trailing whitespace found"
    fi
    
    # Check for missing empty lines at end of files
    files_without_newline=0
    find docs -name "*.md" | while read file; do
        if [ "$(tail -c1 "$file" | wc -l)" -eq 0 ]; then
            echo "  ❌ $file: missing newline at end"
            ((files_without_newline++)) || true
        fi
    done
    
    print_success "Basic formatting check completed"
    echo
}

# Function to check 500-line limit policy
check_500_line_limit() {
    print_header "Checking 500-Line Limit Policy"
    cd "$PROJECT_ROOT"
    
    echo "📏 Checking file size limits..."
    
    violations=$(find . -name "*.rs" -o -name "*.md" | grep -v target | xargs wc -l | awk '$1 > 500 && $2 != "total"' | wc -l)
    
    if [ $violations -gt 0 ]; then
        echo "⚠️  Files exceeding 500-line limit:"
        find . -name "*.rs" -o -name "*.md" | grep -v target | xargs wc -l | awk '$1 > 500 && $2 != "total" {printf "  ❌ %s: %d lines (+%d over limit)\n", $2, $1, $1-500}' | sort -nr
        print_warning "$violations files exceed the 500-line policy"
    else
        print_success "All files comply with 500-line limit"
    fi
    echo
}

# Function to show quick stats
show_stats() {
    print_header "Documentation Statistics"
    cd "$PROJECT_ROOT"
    
    echo "📊 Quick Statistics:"
    echo "  • Markdown files: $(find docs -name "*.md" | wc -l)"
    echo "  • Total lines: $(find docs -name "*.md" -exec cat {} \; | wc -l)"
    echo "  • Directories: $(find docs -type d | wc -l)"
    echo "  • Average file size: $(find docs -name "*.md" -exec wc -l {} \; | awk '{sum+=$1; count++} END {printf "%.0f lines\n", sum/count}')"
    echo
    
    echo "📁 Top-level structure:"
    ls -la docs/ | grep "^d" | awk '{print "  • " $9}' | grep -v "^\.$"
    echo
}

# Function to run all checks
run_all() {
    print_header "Running All Documentation Checks"
    echo
    
    show_stats
    run_validation
    check_todos
    check_structure
    check_links_simple
    format_docs
    check_500_line_limit
    
    print_header "Documentation Check Complete"
    print_success "All local documentation checks completed!"
    echo "💡 Use individual commands for specific checks:"
    echo "   $0 validate    - Run validation only"
    echo "   $0 todos       - Check TODO/FIXME items"
    echo "   $0 links       - Quick link check"
    echo "   $0 stats       - Show statistics"
    echo "   $0 policy      - Check 500-line policy"
}

# Main script logic
case "${1:-all}" in
    "validate"|"val")
        run_validation
        ;;
    "todos"|"todo")
        check_todos
        ;;
    "structure"|"struct")
        check_structure
        ;;
    "links"|"link")
        check_links_simple
        ;;
    "format"|"fmt")
        format_docs
        ;;
    "stats"|"stat")
        show_stats
        ;;
    "policy"|"500"|"limit")
        check_500_line_limit
        ;;
    "all"|"")
        run_all
        ;;
    "help"|"-h"|"--help")
        echo "Documentation Management Script"
        echo
        echo "Usage: $0 [command]"
        echo
        echo "Commands:"
        echo "  all        Run all checks (default)"
        echo "  validate   Run comprehensive validation"
        echo "  todos      Check for TODO/FIXME items"
        echo "  structure  Show documentation structure"
        echo "  links      Quick broken link check"
        echo "  format     Check formatting issues"
        echo "  stats      Show documentation statistics"
        echo "  policy     Check 500-line limit policy"
        echo "  help       Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
