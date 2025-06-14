# 🎯 GitHub Workflows Quick Start

## Available Workflows

| Workflow | Purpose | Trigger | Auto-Run |
|----------|---------|---------|----------|
| 📚 **Quality Check** | Validate docs before merge | PR/Push | ✅ |
| 🔧 **Auto Maintenance** | Keep docs consistent | Weekly | ✅ |
| 📝 **Template Generator** | Create new docs | Manual | ❌ |
| 📊 **Health Monitor** | Track doc quality | Daily | ✅ |

## Quick Commands

```bash
# Generate new documentation
gh workflow run documentation-template.yml \
  --field doc_type=implementation \
  --field component_name="YourComponent"

# Run maintenance manually  
gh workflow run documentation-maintenance.yml

# Check documentation health
gh workflow run documentation-health.yml

# Download latest reports
gh run download --name documentation-health-report
```

## What Each Workflow Does

### 📚 Quality Check (Automatic)
- Validates links and references
- Checks for TODO/FIXME items  
- Runs markdown linting
- Blocks merging if issues found

### 🔧 Auto Maintenance (Weekly)
- Updates timestamps
- Fixes common patterns
- Creates maintenance PRs
- Keeps structure consistent

### 📝 Template Generator (On-Demand)
- Creates docs from templates
- Smart defaults per doc type
- Generates structured content
- Creates PR with template

### 📊 Health Monitor (Daily)
- Calculates health score (0-100)
- Tracks documentation metrics
- Creates issues for problems
- Provides improvement suggestions

## Health Score Guide

- **80-100** 🟢 Excellent
- **60-79** 🟡 Good  
- **40-59** 🟠 Needs Work
- **0-39** 🔴 Critical

## Setup Checklist

- [ ] Repository has write permissions for workflows
- [ ] Branch protection requires quality checks
- [ ] Team members added to reviewer lists
- [ ] Notification preferences configured

See [README.md](README.md) for detailed setup instructions.

---
*Part of the smart documentation system 🤖*
