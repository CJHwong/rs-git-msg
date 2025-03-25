#!/bin/bash

# Exit on error
set -e

echo "Building rs-git-msg..."
cargo build --release

# Determine install location
if [ -d "$HOME/.local/bin" ] && [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    INSTALL_DIR="$HOME/.local/bin"
    SUDO=""
elif [ -d "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
    SUDO="sudo"
else
    INSTALL_DIR="$HOME/.local/bin"
    SUDO=""
    mkdir -p "$INSTALL_DIR"
    echo "Adding $INSTALL_DIR to your PATH..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null || true
fi

# Copy the binary
echo "Installing rs-git-msg to $INSTALL_DIR..."
$SUDO cp target/release/rs-git-msg "$INSTALL_DIR"

echo "Installation complete! You can now use rs-git-msg."
echo "Try it with: rs-git-msg -h"
