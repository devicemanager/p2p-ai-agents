# How to Update BMAD-METHOD for OpenCode Support

## Quick Answer

To update your BMAD-METHOD installation and enable OpenCode support, run:

```bash
./scripts/update-bmad.sh
```

Then select option **1** (Update to latest ALPHA version) and choose **OpenCode** when prompted for your IDE.

## Alternative: Manual Update

```bash
npx bmad-method@alpha install
```

When the installer asks which IDE you want to use, select **OpenCode**.

## What This Does

1. **Updates BMAD-METHOD** from your current version (6.0.0-alpha.22) to the latest alpha
2. **Enables OpenCode support** - OpenCode integration was added in BMAD v6
3. **Configures your IDE settings** in `_bmad/_config/manifest.yaml`
4. **Preserves your existing work** - no code changes, just framework updates

## Current vs. New Setup

### Currently You Have:
- **BMAD Version**: 6.0.0-alpha.22 (older alpha)
- **IDE**: GitHub Copilot
- **Location**: `_bmad/` directory

### After Update:
- **BMAD Version**: Latest alpha (with OpenCode support)
- **IDE**: OpenCode (or your choice)
- **Location**: Same `_bmad/` directory
- **Configuration**: `.opencode.jsonc` or `.opencode/` directory

## Important: Install OpenCode Separately

BMAD only configures the integration. You need to install OpenCode itself:

```bash
npm install -g @opencode/cli
```

Or visit [https://opencode.ai/](https://opencode.ai/) for installation instructions.

## OpenCode vs. GitHub Copilot

| Feature | GitHub Copilot | OpenCode |
|---------|---------------|----------|
| **Cost** | Paid | Free & Open Source |
| **Privacy** | Cloud-based | Can be fully local |
| **Models** | GitHub's models | 75+ providers + local |
| **Configuration** | `.github/copilot-instructions.md` | `.opencode.jsonc` |
| **Activation** | Always active | Switch to agent mode |

## Full Documentation

For complete details, see:
- **Comprehensive Guide**: [`docs/development/bmad-method-update-guide.md`](docs/development/bmad-method-update-guide.md)
- **Quick Reference**: [`docs/development/bmad-quick-reference.md`](docs/development/bmad-quick-reference.md)
- **Implementation Summary**: [`BMAD_UPDATE_IMPLEMENTATION.md`](BMAD_UPDATE_IMPLEMENTATION.md)

## Summary

✅ **Interactive script created**: `./scripts/update-bmad.sh`  
✅ **Documentation provided**: Complete guides in `docs/development/`  
✅ **Everything ready**: Just run the script and follow prompts  
✅ **No code changes needed**: Pure framework update  

**You're all set to update to OpenCode!** 🚀
