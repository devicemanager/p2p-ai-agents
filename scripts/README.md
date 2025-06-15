# 📚 Local Documentation Scripts

This directory contains scripts for running documentation checks and maintenance tasks locally, providing faster feedback than waiting for CI workflows.

## 🚀 Quick Start

### Main Local Documentation Script
```bash
# Run all documentation checks
./scripts/local_docs.sh

# Run specific checks
./scripts/local_docs.sh validate    # Full validation
./scripts/local_docs.sh todos       # Check TODO/FIXME items  
./scripts/local_docs.sh links       # Quick link check
./scripts/local_docs.sh stats       # Show documentation statistics
./scripts/local_docs.sh help        # Show all available commands
```

### Individual Python Scripts
```bash
# Comprehensive validation (same as CI)
python3 scripts/validate_docs.py

# Generate documentation from templates
python3 scripts/generate_docs.py

# Check and fix formatting
bash scripts/test_doc_format.sh
```

## 📋 Available Commands

| Command | Description | Usage |
|---------|-------------|-------|
| `local_docs.sh` | Main documentation management script | `./scripts/local_docs.sh [command]` |
| `validate_docs.py` | Comprehensive validation script | `python3 scripts/validate_docs.py` |
| `generate_docs.py` | Generate docs from templates | `python3 scripts/generate_docs.py` |
| `test_doc_format.sh` | Check documentation formatting | `bash scripts/test_doc_format.sh` |

## 🎯 Common Workflows

### Before Committing Changes
```bash
# Quick check
./scripts/local_docs.sh validate

# Full check with stats
./scripts/local_docs.sh
```

### Working on Documentation
```bash
# Check your changes
./scripts/local_docs.sh links

# Monitor for TODOs
./scripts/local_docs.sh todos

# See documentation stats
./scripts/local_docs.sh stats
```

### Debugging Issues
```bash
# Full validation report
python3 scripts/validate_docs.py

# Check specific formatting
bash scripts/test_doc_format.sh
```

## 🔧 Script Features

### local_docs.sh Features:
- ✅ **Non-blocking**: Won't fail on minor issues during development
- 🎨 **Colored output**: Easy to read results
- ⚡ **Fast execution**: Much faster than CI workflows
- 📊 **Statistics**: Get insights into your documentation
- 🔗 **Link checking**: Find broken internal links
- 📝 **TODO tracking**: Keep track of incomplete items

### Benefits of Local Scripts:
- **Faster Feedback**: Results in seconds instead of minutes
- **No CI Usage**: Save GitHub Actions minutes
- **Offline Work**: Works without internet connection
- **Development-Friendly**: Non-blocking for work-in-progress
- **Detailed Output**: More verbose than CI for debugging

## 💡 Tips

1. **Run before committing**: Use `./scripts/local_docs.sh validate` before pushing changes
2. **Monitor progress**: Use `./scripts/local_docs.sh stats` to track documentation growth
3. **Debug broken links**: The validation script shows exact file paths and link targets
4. **Check formatting**: Use individual commands to focus on specific issues

## 🚀 Integration

You can integrate these scripts into your development workflow:

```bash
# Add to your .bashrc or .zshrc for quick access
alias doc-check='./scripts/local_docs.sh'
alias doc-validate='./scripts/local_docs.sh validate'  
alias doc-stats='./scripts/local_docs.sh stats'

# Git pre-commit hook (optional)
echo './scripts/local_docs.sh validate' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

---

*These scripts provide the same functionality as the GitHub Actions workflows but run locally for faster development feedback.*
