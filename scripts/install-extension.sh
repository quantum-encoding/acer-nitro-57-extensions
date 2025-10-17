#!/bin/bash
set -e

# GNOME Extension Installation Script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
EXTENSION_DIR="$HOME/.local/share/gnome-shell/extensions/boreas@jesternet.org"

echo "========================================="
echo " Installing GNOME Shell Extension"
echo "========================================="
echo ""

# Check if GNOME Shell is installed
if ! command -v gnome-shell &> /dev/null; then
    echo "ERROR: GNOME Shell not found. This extension requires GNOME."
    exit 1
fi

GNOME_VERSION=$(gnome-shell --version | grep -oP '\d+' | head -1)
echo "Detected GNOME Shell version: $GNOME_VERSION"

if [ "$GNOME_VERSION" -lt 45 ]; then
    echo "WARNING: This extension is tested on GNOME 45+. Your version may not be compatible."
    read -p "Continue anyway? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Create extension directory
echo "Installing extension..."
mkdir -p "$EXTENSION_DIR"

# Copy extension files
cp -r "$PROJECT_ROOT/extension/"* "$EXTENSION_DIR/"

echo "✓ Extension files copied to $EXTENSION_DIR"
echo ""
echo "========================================="
echo " Next Steps"
echo "========================================="
echo ""
echo "1. Restart GNOME Shell:"
echo "   - Press Alt+F2"
echo "   - Type 'r'"
echo "   - Press Enter"
echo ""
echo "   OR log out and log back in"
echo ""
echo "2. Enable the extension:"
echo "   gnome-extensions enable boreas@jesternet.org"
echo ""
echo "3. Look for the weather icon in your top bar"
echo ""
echo "To uninstall:"
echo "   rm -rf $EXTENSION_DIR"
echo "   gnome-extensions disable boreas@jesternet.org"
echo ""
