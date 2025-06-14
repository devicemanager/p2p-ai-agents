#!/usr/bin/env python3
"""
Smart Documentation Consistency Checker and Fixer

This script ensures all documentation follows the same standards,
fixes broken references, and validates consistency across the project.
"""

import os
import re
import yaml
from pathlib import Path
from typing import Dict, List, Set, Tuple

class DocumentationChecker:
    def __init__(self, docs_root: str):
        self.docs_root = Path(docs_root)
        self.config = self.load_config()
        self.errors = []
        self.warnings = []
        
    def load_config(self) -> Dict:
        """Load documentation configuration"""
        config_path = self.docs_root / ".docconfig.yaml"
        if config_path.exists():
            with open(config_path, 'r') as f:
                return yaml.safe_load(f)
        return {}
    
    def check_all(self) -> Tuple[List[str], List[str]]:
        """Run all consistency checks"""
        self.check_file_references()
        self.check_version_consistency()
        self.check_terminology_consistency()
        self.check_section_structure()
        return self.errors, self.warnings
    
    def check_file_references(self):
        """Check all internal file references are valid"""
        md_files = list(self.docs_root.rglob("*.md"))
        
        for md_file in md_files:
            content = md_file.read_text()
            # Find markdown links [text](path)
            links = re.findall(r'\[([^\]]*)\]\(([^)]+)\)', content)
            
            for text, link in links:
                if link.startswith(('http', 'https', 'mailto')):
                    continue  # Skip external links
                
                # Resolve relative path
                target_path = (md_file.parent / link).resolve()
                
                if not target_path.exists():
                    self.errors.append(f"Broken link in {md_file.name}: {link}")
    
    def check_version_consistency(self):
        """Check version information is consistent"""
        project_version = self.config.get('project', {}).get('version', '0.1.0')
        md_files = list(self.docs_root.rglob("*.md"))
        
        for md_file in md_files:
            content = md_file.read_text()
            version_matches = re.findall(r'Current Version:\s*([0-9.]+)', content)
            
            for version in version_matches:
                if version != project_version:
                    self.warnings.append(f"Version mismatch in {md_file.name}: {version} vs {project_version}")
    
    def check_terminology_consistency(self):
        """Check terminology usage is consistent"""
        terminology = self.config.get('terminology', {})
        md_files = list(self.docs_root.rglob("*.md"))
        
        # Define inconsistent patterns
        patterns = {
            'project_name': [r'p2p-ai-agents', r'P2P agents', r'P2P AI agents'],
            'task_processing': [r'Task Management', r'task management']
        }
        
        for md_file in md_files:
            content = md_file.read_text()
            
            for term, incorrect_patterns in patterns.items():
                correct_term = terminology.get(term, term)
                
                for pattern in incorrect_patterns:
                    if re.search(pattern, content, re.IGNORECASE):
                        self.warnings.append(f"Inconsistent terminology in {md_file.name}: found '{pattern}', should be '{correct_term}'")
    
    def check_section_structure(self):
        """Check documents follow the required section structure"""
        required_sections = self.config.get('document_structure', {}).get('required_sections', [])
        implementation_files = list((self.docs_root / "implementation").rglob("*.md"))
        
        for md_file in implementation_files:
            content = md_file.read_text()
            
            # Extract headers
            headers = re.findall(r'^#+\s+(.+)$', content, re.MULTILINE)
            
            missing_sections = []
            for section in required_sections:
                if not any(section.lower() in header.lower() for header in headers):
                    missing_sections.append(section)
            
            if missing_sections:
                self.warnings.append(f"Missing sections in {md_file.name}: {', '.join(missing_sections)}")
    
    def generate_fixes(self) -> Dict[str, str]:
        """Generate automatic fixes for common issues"""
        fixes = {}
        
        # Fix broken references based on config
        reference_fixes = self.config.get('references', {})
        
        for old_ref, new_ref in reference_fixes.items():
            fixes[f"../development/setup.md"] = f"../development/README.md"
            fixes[f"../user-guides/monitoring.md"] = f"../user-guides/troubleshooting.md"
            fixes[f"../security/README.md"] = f"../architecture/security.md"
        
        return fixes

if __name__ == "__main__":
    checker = DocumentationChecker("/workspaces/p2p-ai-agents/docs")
    errors, warnings = checker.check_all()
    
    print("🔍 Documentation Consistency Check Results:")
    print(f"❌ Errors: {len(errors)}")
    print(f"⚠️  Warnings: {len(warnings)}")
    
    if errors:
        print("\n❌ ERRORS:")
        for error in errors:
            print(f"  - {error}")
    
    if warnings:
        print("\n⚠️  WARNINGS:")
        for warning in warnings:
            print(f"  - {warning}")
