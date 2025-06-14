#!/bin/bash
# Pre-commit hook for documentation consistency
# Add this to .git/hooks/pre-commit

set -e

echo "🔍 Checking documentation consistency before commit..."

PROJECT_ROOT="$(git rev-parse --show-toplevel)"
DOCS_DIR="$PROJECT_ROOT/docs"

# Check if any docs were modified
if ! git diff --cached --name-only | grep -q "^docs/"; then
    echo "✅ No documentation changes detected, skipping checks"
    exit 0
fi

echo "📝 Documentation changes detected, running consistency checks..."

# Function to check a file for common issues
check_file() {
    local file="$1"
    local issues=0
    
    # Check for TODO/FIXME in committed docs
    if grep -q "TODO\|FIXME" "$file"; then
        echo "⚠️  Warning: $file contains TODO/FIXME items"
        issues=$((issues + 1))
    fi
    
    # Check for broken internal links (basic check)
    if grep -q "](.*\.md)" "$file"; then
        while IFS= read -r line; do
            if [[ $line =~ \]\([^)]+\.md\) ]]; then
                link=$(echo "$line" | grep -o '](.*\.md)' | sed 's/^](//' | sed 's/)$//')
                # Check if it's a relative link that might be broken
                if [[ $link =~ \.\./.*\.\./.*\.md ]]; then
                    echo "⚠️  Warning: Complex relative link in $file: $link"
                    issues=$((issues + 1))
                fi
            fi
        done < "$file"
    fi
    
    # Check for version info consistency
    if grep -q "Current Version:" "$file"; then
        version=$(grep "Current Version:" "$file" | head -1)
        if [[ ! $version =~ "0.1.0" ]]; then
            echo "⚠️  Warning: Version inconsistency in $file: $version"
            issues=$((issues + 1))
        fi
    fi
    
    return $issues
}

# Check all modified documentation files
total_issues=0
git diff --cached --name-only | grep "^docs/.*\.md$" | while read -r file; do
    if [[ -f "$file" ]]; then
        check_file "$file"
        file_issues=$?
        total_issues=$((total_issues + file_issues))
    fi
done

# If there were warnings, ask user if they want to continue
if [[ $total_issues -gt 0 ]]; then
    echo ""
    echo "⚠️  Found $total_issues documentation issues."
    echo "💡 Consider running: scripts/maintain_docs.sh"
    echo ""
    echo "Do you want to commit anyway? (y/N)"
    read -r response
    if [[ ! $response =~ ^[Yy]$ ]]; then
        echo "❌ Commit aborted. Fix the issues and try again."
        exit 1
    fi
fi

echo "✅ Documentation consistency check completed"
exit 0
