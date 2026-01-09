#!/bin/bash
# BMAD-METHOD Update Script
# This script helps update the BMAD-METHOD framework for this project

set -e

echo "═══════════════════════════════════════════════════"
echo "  BMAD-METHOD Update Script"
echo "  P2P AI Agents Project"
echo "═══════════════════════════════════════════════════"
echo ""

# Check current version
echo "📋 Current BMAD Installation:"
if [ -f "_bmad/_config/manifest.yaml" ]; then
    echo ""
    cat _bmad/_config/manifest.yaml
    echo ""
else
    echo "❌ BMAD not found in _bmad/ directory"
    echo "   This script must be run from the project root."
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════"
echo ""
echo "🔄 Update Options:"
echo ""
echo "  1) Update to latest ALPHA version (recommended for OpenCode)"
echo "  2) Update to latest STABLE version"
echo "  3) Fast update (preserve customizations)"
echo "  4) Cancel"
echo ""
read -p "Select option (1-4): " choice

case $choice in
    1)
        echo ""
        echo "🚀 Updating to latest ALPHA version..."
        echo "   This includes OpenCode support and latest features."
        echo ""
        echo "   The installer will ask you to choose your IDE:"
        echo "   - GitHub Copilot (current)"
        echo "   - OpenCode (new)"
        echo "   - Claude Code"
        echo "   - Codex"
        echo ""
        read -p "Press Enter to continue..."
        npx bmad-method@alpha install
        ;;
    2)
        echo ""
        echo "🚀 Updating to latest STABLE version..."
        echo ""
        read -p "Press Enter to continue..."
        npx bmad-method install
        ;;
    3)
        echo ""
        echo "⚡ Running fast update..."
        echo "   This preserves your customizations."
        echo ""
        npx bmad-method update
        ;;
    4)
        echo ""
        echo "❌ Update cancelled."
        exit 0
        ;;
    *)
        echo ""
        echo "❌ Invalid option. Exiting."
        exit 1
        ;;
esac

echo ""
echo "═══════════════════════════════════════════════════"
echo ""
echo "✅ Update process completed!"
echo ""
echo "📋 New BMAD Configuration:"
if [ -f "_bmad/_config/manifest.yaml" ]; then
    echo ""
    cat _bmad/_config/manifest.yaml
    echo ""
else
    echo "⚠️  Manifest file not found. Check installation."
fi

echo ""
echo "═══════════════════════════════════════════════════"
echo ""
echo "📚 Next Steps:"
echo ""
echo "  1. Verify the version and IDE settings above"
echo "  2. If you selected OpenCode, ensure it's installed:"
echo "     npm install -g @opencode/cli"
echo "  3. Check for new configuration files:"
echo "     ls -la .opencode*"
echo "  4. Review documentation:"
echo "     docs/development/bmad-method-update-guide.md"
echo ""
echo "═══════════════════════════════════════════════════"
echo ""
echo "Need help? Check the documentation:"
echo "  • Full guide: docs/development/bmad-method-update-guide.md"
echo "  • Quick ref: docs/development/bmad-quick-reference.md"
echo ""
