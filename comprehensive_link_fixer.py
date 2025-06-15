#!/usr/bin/env python3
"""
Comprehensive documentation link checker and fixer.
This script analyzes actual file paths and fixes links accordingly.
"""

import os
from pathlib import Path

def check_and_fix_path(base_file_path, relative_link):
    """Check if a relative link from base_file_path actually exists and suggest fixes."""
    base_dir = Path(base_file_path).parent
    target_path = (base_dir / relative_link).resolve()
    
    if target_path.exists():
        return None  # Link is correct
    
    # Try to find the actual file
    filename = Path(relative_link).name
    
    # Search for the file in common locations
    search_paths = [
        Path("/workspaces/p2p-ai-agents/docs"),
        Path("/workspaces/p2p-ai-agents/docs/architecture"),
        Path("/workspaces/p2p-ai-agents/docs/development"),
        Path("/workspaces/p2p-ai-agents/docs/user-guides"),
        Path("/workspaces/p2p-ai-agents/docs/implementation"),
    ]
    
    for search_path in search_paths:
        potential_file = search_path / filename
        if potential_file.exists():
            # Calculate the correct relative path
            correct_relative_path = os.path.relpath(potential_file, base_dir)
            return correct_relative_path
    
    return None

def fix_links_in_file(file_path):
    """Fix all links in a single file."""
    import re
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Find all markdown links
        link_pattern = r'\[([^\]]+)\]\(([^)]+)\)'
        matches = re.findall(link_pattern, content)
        
        fixes_made = []
        for link_text, link_url in matches:
            # Skip external links and anchors
            if link_url.startswith('http') or link_url.startswith('#') or link_url.startswith('mailto:'):
                continue
            
            # Check if this is a broken link
            correct_path = check_and_fix_path(file_path, link_url)
            if correct_path and correct_path != link_url:
                old_link = f"[{link_text}]({link_url})"
                new_link = f"[{link_text}]({correct_path})"
                content = content.replace(old_link, new_link)
                fixes_made.append(f"{link_url} → {correct_path}")
        
        if fixes_made:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ Fixed {len(fixes_made)} links in {file_path}")
            for fix in fixes_made:
                print(f"   {fix}")
            return True
        
        return False
    except Exception as e:
        print(f"❌ Error fixing {file_path}: {e}")
        return False

def main():
    """Main function to fix documentation links."""
    root_dir = Path("/workspaces/p2p-ai-agents")
    docs_dir = root_dir / "docs"
    
    print("🔧 Analyzing and fixing documentation links...")
    
    # Find all markdown files
    md_files = []
    for md_file in docs_dir.rglob("*.md"):
        md_files.append(md_file)
    
    fixed_files = 0
    for md_file in md_files:
        if fix_links_in_file(md_file):
            fixed_files += 1
    
    print(f"\n📊 Summary: Fixed links in {fixed_files} files")
    
    # Create the network-manager.md file if it doesn't exist
    network_manager_path = docs_dir / "implementation/network/network-manager.md"
    if not network_manager_path.exists():
        content = """# Network Manager Implementation

## Overview

The Network Manager is responsible for managing network connections, protocols, and communication between agents in the P2P AI Agents system.

## Components

### Connection Manager
- Peer connection establishment
- Connection lifecycle management
- Connection pooling and reuse

### Protocol Manager
- Protocol negotiation
- Message routing
- Protocol-specific handling

### Discovery Manager
- Peer discovery mechanisms
- Network topology management
- Bootstrap node integration

## Implementation Status

🔄 **In Development** - This component is currently being implemented.

## API Reference

*API documentation will be added as implementation progresses.*

## Related Documentation

- [Network Types](types/message-types.md) - Message type definitions
- [Network Implementation](README.md) - Main network documentation
- [Architecture Overview](../../architecture/networking.md) - Network architecture

---
Last updated: 2025-01-27
Status: In Development
"""
        with open(network_manager_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ Created {network_manager_path}")

if __name__ == "__main__":
    main()
