# Excalidraw Diagrams

This directory contains architecture diagrams in Excalidraw format.

## Files

- `diagram-expert-system-architecture.excalidraw` - Main system architecture diagram
- `diagram-expert-system-architecture.png` - PNG export (to be generated)
- `theme.json` - Professional color theme
- `convert-to-png.js` - Automated conversion script

## Viewing Diagrams

### Option 1: Excalidraw Web (Interactive)
1. Go to https://excalidraw.com
2. Click "Open" and select the `.excalidraw` file
3. View and edit interactively

### Option 2: VS Code Extension
1. Install "Excalidraw" extension
2. Open `.excalidraw` files directly in VS Code

## Converting to PNG

### Method 1: Manual Export (Recommended)
1. Open https://excalidraw.com
2. Load the `.excalidraw` file
3. Click "Export image" → PNG → Scale 2x → Export
4. Save as same name with `.png` extension

### Method 2: Automated Script
```bash
# Install dependencies
npm install puppeteer

# Run conversion
node convert-to-png.js
```

## Documentation

See [ARCHITECTURE_DIAGRAM.md](../vision-transformation-2026-01-09/ARCHITECTURE_DIAGRAM.md) for complete architecture documentation.
