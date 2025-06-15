#!/usr/bin/env python3
"""
Create missing documentation files and fix remaining broken links.
"""

import os
from pathlib import Path

def create_missing_files():
    """Create missing documentation files that are referenced."""
    root_dir = Path("/workspaces/p2p-ai-agents")
    
    # Create network-manager.md file
    network_manager_path = root_dir / "docs/implementation/network/network-manager.md"
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

def fix_remaining_links():
    """Fix the remaining broken links."""
    root_dir = Path("/workspaces/p2p-ai-agents")
    
    # Fix links in newly created README files
    link_fixes = {
        # Fix architecture/README.md
        f"{root_dir}/docs/architecture/README.md": {
            "../system-overview.md": "system-overview.md",
            "../../development/README.md": "../development/README.md"
        },
        
        # Fix user-guides/README.md
        f"{root_dir}/docs/user-guides/README.md": {
            "../system-overview.md": "../architecture/system-overview.md",
            "../../development/README.md": "../development/README.md"
        },
        
        # Fix implementation/network/security/README.md  
        f"{root_dir}/docs/implementation/network/security/README.md": {
            "../system-overview.md": "../../../architecture/system-overview.md",
            "../../development/README.md": "../../../development/README.md"
        },
        
        # Fix implementation/network/performance/README.md
        f"{root_dir}/docs/implementation/network/performance/README.md": {
            "../system-overview.md": "../../../architecture/system-overview.md", 
            "../../development/README.md": "../../../development/README.md"
        }
    }
    
    for file_path, replacements in link_fixes.items():
        if os.path.exists(file_path):
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
            except Exception as e:
                print(f"❌ Error fixing {file_path}: {e}")

def main():
    print("🔧 Creating missing files and fixing remaining links...")
    create_missing_files()
    fix_remaining_links()
    print("✅ Done!")

if __name__ == "__main__":
    main()
