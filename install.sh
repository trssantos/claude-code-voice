#!/bin/bash
# Claude Code Voice - One-line installation script
# Usage: curl -sSL https://raw.githubusercontent.com/trssantos/claude-code-voice/main/install.sh | bash

set -e

REPO="trssantos/claude-code-voice"
BINARY_NAME="claude-code-voice"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS
detect_os() {
    case "$OSTYPE" in
        linux-gnu*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        msys*|win32)
            OS="windows"
            ;;
        *)
            error "Unsupported OS: $OSTYPE"
            ;;
    esac
}

# Install system dependencies
install_system_deps() {
    info "Installing system dependencies..."

    if [[ "$OS" == "linux" ]]; then
        if command -v apt-get &> /dev/null; then
            info "Detected Ubuntu/Debian"
            sudo apt-get update
            sudo apt-get install -y libasound2-dev pkg-config
        elif command -v dnf &> /dev/null; then
            info "Detected Fedora"
            sudo dnf install -y alsa-lib-devel
        elif command -v pacman &> /dev/null; then
            info "Detected Arch"
            sudo pacman -S --noconfirm alsa-lib
        else
            warn "Could not detect package manager. Please install ALSA development libraries manually."
        fi
    elif [[ "$OS" == "macos" ]]; then
        info "macOS detected - no additional system dependencies needed"
    elif [[ "$OS" == "windows" ]]; then
        info "Windows detected - no additional system dependencies needed"
    fi
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        info "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        info "Rust is already installed"
    fi
}

# Install the tool
install_tool() {
    info "Installing $BINARY_NAME from source..."
    cargo install --git "https://github.com/$REPO"
}

# Add to PATH if needed
setup_path() {
    CARGO_BIN="$HOME/.cargo/bin"

    # Check if cargo bin is in PATH
    if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
        info "Adding $CARGO_BIN to PATH..."

        # Determine shell config file
        if [[ -n "$ZSH_VERSION" ]]; then
            SHELL_CONFIG="$HOME/.zshrc"
        elif [[ -n "$BASH_VERSION" ]]; then
            SHELL_CONFIG="$HOME/.bashrc"
        else
            SHELL_CONFIG="$HOME/.profile"
        fi

        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$SHELL_CONFIG"
        export PATH="$CARGO_BIN:$PATH"

        info "Added to PATH in $SHELL_CONFIG"
        warn "Please run: source $SHELL_CONFIG"
    fi
}

# Download base model
download_model() {
    info "Would you like to download the base Whisper model now? (~142MB)"
    read -p "Download model? [Y/n] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        info "Downloading base model..."
        "$BINARY_NAME" download base || warn "Model download failed. You can download it later with: $BINARY_NAME download base"
    else
        info "Skipping model download. Download later with: $BINARY_NAME download base"
    fi
}

# Main installation flow
main() {
    echo "=================================="
    echo "  Claude Code Voice Installer"
    echo "=================================="
    echo ""

    detect_os
    install_system_deps
    check_rust
    install_tool
    setup_path

    info "Installation complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Download a Whisper model: $BINARY_NAME download base"
    echo "  2. Start the daemon: $BINARY_NAME start"
    echo "  3. Press Ctrl+Shift+Space to record, release to transcribe"
    echo ""
    echo "For help: $BINARY_NAME --help"
    echo ""

    # Only offer to download model if binary is in PATH
    if command -v "$BINARY_NAME" &> /dev/null; then
        download_model
    fi
}

main
