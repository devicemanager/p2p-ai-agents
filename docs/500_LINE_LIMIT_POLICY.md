# 500 Line Limit Policy

This policy assumes a Rust implementation.

## Overview

P2P AI Agents enforces a **500-line maximum limit** for all files in the project. This policy ensures the codebase and documentation remain lean, accessible, and manageable for smaller models and resource-constrained environments.

## Rationale

### 🤖 AI Model Compatibility
- **Small language models** can process files under 500 lines efficiently
- **Edge devices** with limited memory can handle smaller files
- **Faster processing** and better comprehension for AI assistants
- **Reduced context window usage** for cost-effective AI operations

### 💻 Resource Efficiency
- **Lower memory footprint** on development machines
- **Faster file loading** and editing in IDEs
- **Improved git performance** with smaller diffs
- **Better network transfer** for distributed development

### 👥 Developer Experience
- **Easier code review** with manageable file sizes
- **Clearer separation of concerns** through natural boundaries
- **Improved maintainability** with focused, single-purpose files
- **Better documentation consumption** in digestible chunks

## Policy Application

### 📝 Documentation Files
- **Maximum 500 lines** per Markdown file
- **Split into logical sections** when approaching the limit
- **Create table of contents** with linked sub-documents
- **Maintain cross-references** between related documents

### 💻 Source Code Files
- **Maximum 500 lines** per Rust/JavaScript/etc. file
- **Refactor into modules** when approaching the limit
- **Use composition over inheritance** to keep classes small
- **Extract utilities** into separate files

### ⚙️ Configuration Files
- **Maximum 500 lines** per configuration file
- **Split complex configurations** into environment-specific files
- **Use includes/imports** where supported
- **Modularize settings** by feature area

## Implementation Guidelines

### When Approaching 500 Lines

1. **Analyze the content** for natural break points
2. **Identify logical sections** that can be separated
3. **Create a main file** with table of contents
4. **Extract sections** to numbered sub-files
5. **Update cross-references** and navigation

### File Naming Convention

When splitting files, use this pattern:
```
original-file.md           # Main file with table of contents
original-file-01.md        # First section
original-file-02.md        # Second section
original-file-03.md        # Third section
```

For code files:
```
feature.rs                 # Main module with imports
feature_core.rs           # Core functionality
feature_utils.rs          # Utility functions
feature_config.rs         # Configuration handling
```

## Enforcement

### Automated Checks
- **Pre-commit hooks** validate file line counts
- **CI/CD pipelines** fail builds with oversized files
- **Linting tools** warn when approaching the limit
- **Documentation tools** generate size reports

### Review Process
- **Pull requests** must maintain the 500-line limit
- **Code reviews** specifically check file sizes
- **Refactoring issues** created when files approach 400 lines
- **Regular audits** of file sizes across the project

## Benefits

### For Small Models
- **Complete file processing** within typical context windows
- **Better semantic understanding** with focused content
- **Improved accuracy** in code analysis and generation
- **Faster response times** with smaller inputs

### For Edge Devices
- **Lower memory requirements** for development environments
- **Reduced storage needs** with more modular files
- **Better performance** on resource-constrained systems
- **Improved battery life** with efficient processing

### For Development Teams
- **Clearer architecture** through forced modularization
- **Better collaboration** with smaller, focused changes
- **Easier onboarding** with digestible code chunks
- **Improved code quality** through natural refactoring

## Exceptions

### Rare Cases Where Limits May Be Exceeded
- **Generated files** (auto-generated code, compiled assets)
- **Third-party dependencies** (vendor libraries, frameworks)
- **Data files** (datasets, large configurations)
- **Legacy migrations** (temporary during refactoring)

### Approval Process for Exceptions
1. **Document the reason** for exceeding the limit
2. **Create an issue** tracking the exception
3. **Get approval** from project maintainers
4. **Plan remediation** with specific timeline
5. **Regular review** of all exceptions

## Examples

### Documentation Split Example
```markdown
# main-topic.md (Table of Contents)
1. [Introduction](README.md)
2. [Architecture](architecture/)
3. [Implementation](user-guides/)
4. [Examples](implementation/)
```

### Code Module Split Example
```rust
# agent.rs (Main interface)
from .agent_core import AgentCore
from .agent_network import NetworkManager
from .agent_tasks import TaskManager

class Agent:
    def __init__(self):
        self.core = AgentCore()
        self.network = NetworkManager()
        self.tasks = TaskManager()
```

## Monitoring

### Current Status
All files in P2P AI Agents currently comply with the 500-line limit:

| File Type | Max Lines | Current Max | Status |
|-----------|-----------|-------------|---------|
| Documentation | 500 | 367 | ✅ Compliant |
| Source Code | 500 | 139 | ✅ Compliant |
| Configuration | 500 | 201 | ✅ Compliant |

### Tools for Monitoring
```bash
# Check all file sizes
find . -name "*.rs" -o -name "*.md" | xargs wc -l | sort -nr

# Pre-commit hook for line count validation
# (automatically configured in .pre-commit-config.yaml)

# CI check for line limits
grep -r "^.{500,}" --include="*.rs" --include="*.md" .
```

## Migration Strategy

### For Existing Projects
1. **Audit current files** and identify oversized ones
2. **Prioritize by impact** (frequently changed files first)
3. **Refactor incrementally** to avoid disruption
4. **Update documentation** to reflect new structure
5. **Train team** on new practices

### For New Development
1. **Plan modules** to stay under 500 lines from start
2. **Use IDE warnings** when approaching limits
3. **Regular refactoring** as part of development process
4. **Peer review** for size compliance

---

**This policy is mandatory for all contributions to P2P AI Agents and helps ensure our project remains accessible to developers and AI systems of all sizes.**

*Policy Version: 1.0 | Effective Date: June 11, 2025*
