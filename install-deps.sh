#!/bin/bash
# Installation script for claude-code-voice system dependencies

set -e

echo "Installing system dependencies for claude-code-voice..."

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if command -v apt-get &> /dev/null; then
        echo "Detected Ubuntu/Debian"
        sudo apt-get update
        sudo apt-get install -y libasound2-dev pkg-config
    elif command -v dnf &> /dev/null; then
        echo "Detected Fedora"
        sudo dnf install -y alsa-lib-devel
    elif command -v pacman &> /dev/null; then
        echo "Detected Arch"
        sudo pacman -S --noconfirm alsa-lib
    else
        echo "Unsupported Linux distribution. Please install ALSA development libraries manually."
        exit 1
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "macOS detected - no additional dependencies needed"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo "Windows detected - no additional dependencies needed"
else
    echo "Unknown OS: $OSTYPE"
    exit 1
fi

echo "Dependencies installed successfully!"
echo "You can now build with: cargo build --release"
