#!/usr/bin/env python3
"""
Smart Documentation Validator
Checks for inconsistencies, broken links, and formatting issues.
"""

import os
import re
import glob
from pathlib import Path
from typing import List, Dict, Set, Tuple

class DocumentationValidator:
    def __init__(self, root_dir: str):
        self.root_dir = Path(root_dir)
        self.docs_dir = self.root_dir / "docs"
        self.issues = []
        
    def validate_all(self):
        """Run all validation checks"""
        print("🔍 Running comprehensive documentation validation...")
        
        self.check_broken_links()
        self.check_version_consistency()
        self.check_terminology_consistency()
        self.check_file_structure()
        
        return self.report_results()
    
    def check_broken_links(self):
        """Check for broken internal links"""
        print("\n📋 Checking internal links...")
        
        # Get all markdown files
        md_files = list(self.docs_dir.rglob("*.md"))
        existing_files = {f.relative_to(self.docs_dir) for f in md_files}
        
        link_pattern = r'\[([^\]]*)\]\(([^)]+)\)'
        
        for file_path in md_files:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            for match in re.finditer(link_pattern, content):
                link_text, link_url = match.groups()
                
                # Skip external links
                if link_url.startswith(('http', 'https', 'mailto')):
                    continue
                    
                # Skip anchors
                if link_url.startswith('#'):
                    continue
                
                # Convert relative path to absolute
                if link_url.startswith('../'):
                    # Handle relative paths - check in parent directory too
                    current_dir = file_path.parent.relative_to(self.docs_dir)
                    target_path = current_dir / link_url
                    
                    # Try to resolve the path relative to docs directory
                    try:
                        normalized_path = target_path.resolve()
                        # Check if it's a file in the docs directory
                        if normalized_path.is_relative_to(self.docs_dir):
                            relative_to_docs = normalized_path.relative_to(self.docs_dir)
                            if relative_to_docs not in existing_files:
                                relative_file = file_path.relative_to(self.docs_dir)
                                self.issues.append(f"❌ Broken link in {relative_file}: {link_url}")
                        else:
                            # Check if it exists in the parent directory (project root)
                            root_target = self.root_dir / link_url.replace('../', '')
                            if not root_target.exists():
                                relative_file = file_path.relative_to(self.docs_dir)
                                self.issues.append(f"❌ Broken link in {relative_file}: {link_url}")
                    except (ValueError, OSError):
                        # If path resolution fails, mark as broken
                        relative_file = file_path.relative_to(self.docs_dir)
                        self.issues.append(f"❌ Broken link in {relative_file}: {link_url}")
                else:
                    # Handle same-directory paths
                    current_dir = file_path.parent.relative_to(self.docs_dir)
                    normalized_path = current_dir / link_url
                    
                    # Check if target file exists
                    if normalized_path not in existing_files:
                        relative_file = file_path.relative_to(self.docs_dir)
                        self.issues.append(f"❌ Broken link in {relative_file}: {link_url}")
                    
    def check_version_consistency(self):
        """Check version information consistency"""
        print("\n📋 Checking version consistency...")
        
        version_pattern = r'Current Version:\s*([^\n]+)'
        versions = {}
        
        for file_path in self.docs_dir.rglob("*.md"):
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            matches = re.findall(version_pattern, content)
            if matches:
                relative_path = file_path.relative_to(self.docs_dir)
                versions[str(relative_path)] = matches[0].strip()
        
        # Check for inconsistent versions
        unique_versions = set(versions.values())
        if len(unique_versions) > 1:
            self.issues.append(f"⚠️  Version inconsistency found: {unique_versions}")
            
    def check_terminology_consistency(self):
        """Check for consistent terminology usage"""
        print("\n📋 Checking terminology consistency...")
        
        # Define problematic patterns
        inconsistencies = {
            "Project Name": [
                (r'p2p ai agents', 'Should be "P2P AI Agents"'),
                (r'P2P-AI-Agents', 'Should be "P2P AI Agents" or "p2p-ai-agents"'),
            ],
            "Task Processing": [
                (r'task management(?! system)', 'Should be "task processing" unless referring to scheduling'),
            ]
        }
        
        for file_path in self.docs_dir.rglob("*.md"):
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            relative_path = file_path.relative_to(self.docs_dir)
            
            for category, patterns in inconsistencies.items():
                for pattern, suggestion in patterns:
                    matches = re.findall(pattern, content, re.IGNORECASE)
                    if matches:
                        self.issues.append(f"⚠️  {category} issue in {relative_path}: {suggestion}")
    
    def check_file_structure(self):
        """Check for required files and proper structure"""
        print("\n📋 Checking file structure...")
        
        # Required files in docs/
        required_files = [
            "INDEX.md",
            "README.md", 
            "GLOSSARY.md"
        ]
        
        for req_file in required_files:
            file_path = self.docs_dir / req_file
            if not file_path.exists():
                self.issues.append(f"❌ Missing required file: docs/{req_file}")
    
    def report_results(self):
        """Generate validation report"""
        print("\n" + "="*60)
        print("📊 DOCUMENTATION VALIDATION REPORT")
        print("="*60)
        
        if not self.issues:
            print("✅ All checks passed! Documentation is consistent.")
            return True
        
        print(f"Found {len(self.issues)} issues:")
        for issue in self.issues:
            print(f"  {issue}")
            
        print("\n💡 Run the maintenance script to fix common issues:")
        print("   python scripts/maintain_docs.py --fix-all")
        
        return False

if __name__ == "__main__":
    validator = DocumentationValidator("/workspaces/p2p-ai-agents")
    success = validator.validate_all()
    exit(0 if success else 1)
