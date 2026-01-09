#!/usr/bin/env python3
"""
Documentation Link Validator
Validates all internal markdown links after documentation restructuring
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple, Set

class LinkValidator:
    def __init__(self, root_dir: str = "."):
        self.root_dir = Path(root_dir).resolve()
        self.broken_links: List[Tuple[str, str, str]] = []
        self.checked_files: Set[Path] = set()
        
    def find_markdown_files(self) -> List[Path]:
        """Find all markdown files in the repository"""
        md_files = []
        exclude_dirs = {'node_modules', 'target', '.git', '_bmad-output', 'pooler_test_project'}
        
        for root, dirs, files in os.walk(self.root_dir):
            # Remove excluded directories
            dirs[:] = [d for d in dirs if d not in exclude_dirs]
            
            for file in files:
                if file.endswith('.md'):
                    md_files.append(Path(root) / file)
        
        return md_files
    
    def extract_links(self, content: str) -> List[str]:
        """Extract all markdown links from content"""
        # Match [text](link) and [text](link "title")
        pattern = r'\[([^\]]+)\]\(([^)]+)\)'
        matches = re.findall(pattern, content)
        return [link for _, link in matches]
    
    def is_internal_link(self, link: str) -> bool:
        """Check if link is internal (not external URL or anchor)"""
        return not (
            link.startswith('http://') or
            link.startswith('https://') or
            link.startswith('#') or
            link.startswith('mailto:')
        )
    
    def resolve_link(self, source_file: Path, link: str) -> Path:
        """Resolve a relative link to absolute path"""
        # Remove anchor if present
        link_path = link.split('#')[0]
        if not link_path:
            return source_file  # Just an anchor
        
        # Resolve relative to source file's directory
        resolved = (source_file.parent / link_path).resolve()
        return resolved
    
    def validate_file(self, file_path: Path) -> None:
        """Validate all links in a single file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"⚠️  Error reading {file_path}: {e}")
            return
        
        links = self.extract_links(content)
        
        for link in links:
            if not self.is_internal_link(link):
                continue
            
            target = self.resolve_link(file_path, link)
            
            # Check if target exists
            if not target.exists():
                rel_source = file_path.relative_to(self.root_dir)
                self.broken_links.append((str(rel_source), link, str(target)))
    
    def validate_all(self) -> bool:
        """Validate all markdown files"""
        print("🔍 Finding markdown files...")
        md_files = self.find_markdown_files()
        print(f"📝 Found {len(md_files)} markdown files")
        print()
        
        print("🔗 Validating links...")
        for i, file_path in enumerate(md_files, 1):
            if file_path in self.checked_files:
                continue
            
            self.checked_files.add(file_path)
            self.validate_file(file_path)
            
            if i % 10 == 0:
                print(f"  Checked {i}/{len(md_files)} files...")
        
        print(f"✅ Checked {len(self.checked_files)} files")
        print()
        
        return len(self.broken_links) == 0
    
    def print_report(self) -> None:
        """Print validation report"""
        if not self.broken_links:
            print("════════════════════════════════════════════════════════════════")
            print("  ✅ All Links Valid!")
            print("════════════════════════════════════════════════════════════════")
            print()
            print(f"Validated {len(self.checked_files)} files - no broken links found")
            return
        
        print("════════════════════════════════════════════════════════════════")
        print(f"  ❌ Found {len(self.broken_links)} Broken Links")
        print("════════════════════════════════════════════════════════════════")
        print()
        
        for source, link, target in self.broken_links:
            print(f"📄 {source}")
            print(f"   🔗 Link: {link}")
            print(f"   ❌ Target not found: {target}")
            print()
        
        print("════════════════════════════════════════════════════════════════")

def main():
    """Main entry point"""
    print("════════════════════════════════════════════════════════════════")
    print("  Documentation Link Validator")
    print("════════════════════════════════════════════════════════════════")
    print()
    
    validator = LinkValidator()
    is_valid = validator.validate_all()
    validator.print_report()
    
    sys.exit(0 if is_valid else 1)

if __name__ == "__main__":
    main()
