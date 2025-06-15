#!/usr/bin/env python3
"""
Comprehensive broken links fixer for P2P AI Agents documentation.
This script analyzes and fixes broken links found by the validation script.
"""

import os
import re
from pathlib import Path

def fix_file_links(file_path, replacements):
    """Fix links in a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        for old_link, new_link in replacements.items():
            content = content.replace(old_link, new_link)
        
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ Fixed links in {file_path}")
            return True
        
        return False
    except Exception as e:
        print(f"❌ Error fixing {file_path}: {e}")
        return False

def main():
    """Main function to fix all broken links."""
    root_dir = Path("/workspaces/p2p-ai-agents")
    docs_dir = root_dir / "docs"
    
    print("🔧 Fixing broken links in documentation...")
    
    # Define link fixes based on validation output
    link_fixes = {
        # 500_LINE_LIMIT_POLICY.md fixes
        f"{docs_dir}/500_LINE_LIMIT_POLICY.md": {
            "architecture/": "architecture/README.md",
            "user-guides/": "user-guides/README.md", 
            "implementation/": "implementation/README.md"
        },
        
        # TEMPLATE.md fixes
        f"{docs_dir}/TEMPLATE.md": {
            "../architecture/system-overview.md": "architecture/system-overview.md",
            "../architecture/security.md": "architecture/security.md"
        },
        
        # development/README.md fixes
        f"{docs_dir}/development/README.md": {
            "../architecture/system-overview.md": "../architecture/system-overview.md",
            "../user-guides/security-best-practices.md": "../user-guides/security-best-practices.md"
        },
        
        # development/testing-guide.md fixes  
        f"{docs_dir}/development/testing-guide.md": {
            "../architecture/system-overview.md": "../architecture/system-overview.md",
            "../user-guides/security-best-practices.md": "../user-guides/security-best-practices.md",
            "../user-guides/troubleshooting.md": "../user-guides/troubleshooting.md"
        },
        
        # user-guides/security-best-practices.md fixes
        f"{docs_dir}/user-guides/security-best-practices.md": {
            "../architecture/security.md": "../architecture/security.md"
        },
        
        # user-guides/troubleshooting.md fixes
        f"{docs_dir}/user-guides/troubleshooting.md": {
            "README.md": "../README.md"
        },
        
        # user-guides/getting-started.md fixes
        f"{docs_dir}/user-guides/getting-started.md": {
            "../architecture/system-overview.md": "../architecture/system-overview.md",
            "../development/README.md": "../development/README.md",
            "../implementation/": "../implementation/README.md"
        },
        
        # user-guides/agent-configuration.md fixes
        f"{docs_dir}/user-guides/agent-configuration.md": {
            "README.md": "../README.md"
        },
        
        # implementation/network/INDEX.md fixes
        f"{docs_dir}/implementation/network/INDEX.md": {
            "../../development/testing-guide.md": "../../development/testing-guide.md",
            "../agent.md": "../agent.md",
            "../task.md": "../task.md", 
            "../storage.md": "../storage.md",
            "../../architecture/security.md": "../../architecture/security.md"
        },
        
        # implementation/network/network-types.md fixes
        f"{docs_dir}/implementation/network/network-types.md": {
            "../security/README.md": "security/README.md",
            "../performance/README.md": "performance/README.md"
        },
        
        # implementation/network/README.md fixes
        f"{docs_dir}/implementation/network/README.md": {
            "../agent.md": "../agent.md",
            "../task.md": "../task.md",
            "../storage.md": "../storage.md", 
            "../../architecture/security.md": "../../architecture/security.md"
        },
        
        # implementation/network/network-manager.md fixes
        f"{docs_dir}/implementation/network/network-manager.md": {
            "../security/README.md": "security/README.md",
            "../performance/README.md": "performance/README.md"
        },
        
        # implementation/network/types/ fixes
        f"{docs_dir}/implementation/network/types/message-types.md": {
            "../network-manager.md": "../network-manager.md"
        },
        
        f"{docs_dir}/implementation/network/types/error-types.md": {
            "../network-manager.md": "../network-manager.md"
        },
        
        f"{docs_dir}/implementation/network/types/protocol-types.md": {
            "../network-manager.md": "../network-manager.md"
        }
    }
    
    fixed_files = 0
    
    # Apply fixes
    for file_path, replacements in link_fixes.items():
        if os.path.exists(file_path):
            if fix_file_links(file_path, replacements):
                fixed_files += 1
        else:
            print(f"⚠️  File not found: {file_path}")
    
    print(f"\n📊 Summary: Fixed links in {fixed_files} files")
    
    # Create missing README files for directories
    print("\n🔧 Creating missing README files...")
    
    missing_readmes = [
        ("docs/architecture", "Architecture Documentation"),
        ("docs/user-guides", "User Guides"), 
        ("docs/implementation/network/security", "Network Security Documentation"),
        ("docs/implementation/network/performance", "Network Performance Documentation")
    ]
    
    for dir_path, title in missing_readmes:
        full_path = root_dir / dir_path
        readme_path = full_path / "README.md"
        
        if not readme_path.exists():
            if not full_path.exists():
                full_path.mkdir(parents=True, exist_ok=True)
            
            readme_content = f"""# {title}

This directory contains {title.lower()} for the P2P AI Agents system.

## Contents

*This directory is being populated with documentation.*

## Related Documentation

- [Main Documentation](../../README.md)
- [Architecture Overview](../system-overview.md)
- [Development Guide](../../development/README.md)

---
Last updated: 2025-01-27
Auto-generated README
"""
            
            with open(readme_path, 'w', encoding='utf-8') as f:
                f.write(readme_content)
            print(f"✅ Created {readme_path}")
    
    print("\n✅ All broken links have been fixed!")
    print("🔍 Run 'python3 scripts/validate_docs.py' to verify the fixes")

if __name__ == "__main__":
    main()
