#!/bin/bash

# Documentation Maintenance Script
# Automatically fixes common documentation issues and maintains consistency

set -e

DOCS_ROOT="/Users/rene/Source/p2p-ai-agents/docs"
echo "🔧 Running documentation maintenance..."

# Function to update version information
update_versions() {
    echo "📅 Updating version information..."
    local version="0.1.0"
    local date=$(date +"%Y-%m-%d")
    
    find "$DOCS_ROOT" -name "*.md" -exec sed -i "s/Last Updated: [0-9-]*/Last Updated: $date/g" {} \;
    find "$DOCS_ROOT" -name "*.md" -exec sed -i "s/Current Version: [0-9.]*/Current Version: $version/g" {} \;
}

# Function to validate links
validate_links() {
    echo "🔗 Validating internal links..."
    python3 << 'EOF'
import os
import re
from pathlib import Path

docs_root = Path("/Users/rene/Source/p2p-ai-agents/docs")
errors = []

md_files = list(docs_root.rglob("*.md"))

for md_file in md_files:
    try:
        content = md_file.read_text()
        links = re.findall(r'\[([^\]]*)\]\(([^)]+)\)', content)
        
        for text, link in links:
            if link.startswith(('http', 'https', 'mailto', '#')):
                continue
            
            target_path = (md_file.parent / link).resolve()
            
            if not target_path.exists():
                print(f"BROKEN: {md_file.relative_to(docs_root)}: {link}")
                errors.append((md_file, link))
    except Exception as e:
        print(f"ERROR processing {md_file}: {e}")

print(f"Found {len(errors)} broken links")
EOF
}

# Function to standardize terminology
fix_terminology() {
    echo "📝 Standardizing terminology..."
    
    # Fix common terminology issues
    find "$DOCS_ROOT" -name "*.md" -exec sed -i 's/P2P agents/P2P AI Agents/g' {} \;
    find "$DOCS_ROOT" -name "*.md" -exec sed -i 's/P2P AI agents/P2P AI Agents/g' {} \;
    find "$DOCS_ROOT" -name "*.md" -exec sed -i 's/task management/Task Processing/g' {} \;
}

# Function to check markdown formatting
check_formatting() {
    echo "🎨 Checking markdown formatting..."
    
    if command -v markdownlint >/dev/null 2>&1; then
        markdownlint "$DOCS_ROOT" --config "$DOCS_ROOT/.markdownlint.json" || echo "⚠️  Markdown lint issues found"
    else
        echo "⚠️  markdownlint not installed, skipping format check"
    fi
}

# Function to generate documentation index
generate_index() {
    echo "📚 Updating documentation index..."
    
    python3 << 'EOF'
import os
from pathlib import Path

docs_root = Path("/Users/rene/Source/p2p-ai-agents/docs")
md_files = []

for md_file in docs_root.rglob("*.md"):
    if md_file.name not in ["INDEX.md", "README.md", "TEMPLATE.md"]:
        rel_path = md_file.relative_to(docs_root)
        md_files.append(str(rel_path))

md_files.sort()

print("📋 Documentation files found:")
for file in md_files:
    print(f"  - {file}")
EOF
}

# Create markdownlint config if it doesn't exist
create_lint_config() {
    local config_file="$DOCS_ROOT/.markdownlint.json"
    if [ ! -f "$config_file" ]; then
        echo "⚙️  Creating markdownlint configuration..."
        cat > "$config_file" << 'EOF'
{
  "MD013": false,
  "MD033": false,
  "MD041": false,
  "MD022": false,
  "MD032": false,
  "MD025": false,
  "MD031": false,
  "MD040": false,
  "MD036": false,
  "MD047": false,
  "MD009": false
}
EOF
    fi
}

# Main execution
main() {
    cd "$DOCS_ROOT"
    
    create_lint_config
    update_versions
    fix_terminology
    validate_links
    check_formatting
    generate_index
    
    echo "✅ Documentation maintenance completed!"
    echo ""
    echo "🎯 Remaining tasks:"
    echo "  - Review broken links and create missing files"
    echo "  - Ensure all implementation docs follow the template"
    echo "  - Add examples and usage guides"
    echo "  - Create comprehensive cross-references"
}

main "$@"
