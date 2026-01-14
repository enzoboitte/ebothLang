#!/bin/bash

# Eboth VS Code Extension Installer
# Usage: ./install.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXT_NAME="eboth"

# Detect VS Code extensions directory
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    VSCODE_EXT_DIR="$HOME/.vscode/extensions"
    VSCODE_INSIDERS_DIR="$HOME/.vscode-insiders/extensions"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    VSCODE_EXT_DIR="$HOME/.vscode/extensions"
    VSCODE_INSIDERS_DIR="$HOME/.vscode-insiders/extensions"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    VSCODE_EXT_DIR="$APPDATA/Code/User/extensions"
    VSCODE_INSIDERS_DIR="$APPDATA/Code - Insiders/User/extensions"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

# Check if VS Code extensions directory exists
TARGET_DIR=""
if [[ -d "$VSCODE_EXT_DIR" ]]; then
    TARGET_DIR="$VSCODE_EXT_DIR"
elif [[ -d "$VSCODE_INSIDERS_DIR" ]]; then
    TARGET_DIR="$VSCODE_INSIDERS_DIR"
else
    echo "Creating VS Code extensions directory..."
    mkdir -p "$VSCODE_EXT_DIR"
    TARGET_DIR="$VSCODE_EXT_DIR"
fi

INSTALL_DIR="$TARGET_DIR/$EXT_NAME"

echo "Installing Eboth VS Code Extension..."
echo "   Source: $SCRIPT_DIR"
echo "   Target: $INSTALL_DIR"

# Remove old installation if exists
if [[ -d "$INSTALL_DIR" ]]; then
    echo "Removing old installation..."
    rm -rf "$INSTALL_DIR"
fi

# Create extension directory
mkdir -p "$INSTALL_DIR"

# Copy extension files
echo "Copying extension files..."
cp "$SCRIPT_DIR/package.json" "$INSTALL_DIR/"
cp "$SCRIPT_DIR/language-configuration.json" "$INSTALL_DIR/"

# Copy syntaxes
mkdir -p "$INSTALL_DIR/syntaxes"
cp "$SCRIPT_DIR/syntaxes/"*.json "$INSTALL_DIR/syntaxes/"

# Copy snippets
mkdir -p "$INSTALL_DIR/snippets"
cp "$SCRIPT_DIR/snippets/"*.json "$INSTALL_DIR/snippets/"

echo ""
echo "Eboth extension installed successfully!"
echo ""
echo "Next steps:"
echo "   1. Restart VS Code (or reload window: Ctrl+Shift+P -> 'Reload Window')"
echo "   2. Open a .eb file to see syntax highlighting"
echo "   3. Type 'main' + Tab to use snippets"
echo ""
