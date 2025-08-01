#!/bin/bash
# cassh2rs installer script

set -e

REPO="github.com/casapps/cassh2rs"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="cassh2rs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}cassh2rs Installer${NC}"
echo "=================="
echo

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        OS="linux"
        ;;
    darwin)
        OS="darwin"
        ;;
    mingw*|cygwin*|msys*)
        OS="windows"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH="amd64"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        ;;
    armv7l|armhf)
        ARCH="armv7"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

BINARY_FILE="${BINARY_NAME}_${OS}_${ARCH}"
if [[ "$OS" == "windows" ]]; then
    BINARY_FILE="${BINARY_FILE}.exe"
fi

echo "Detected platform: $OS/$ARCH"
echo "Binary: $BINARY_FILE"
echo

# Check if installing from source or downloading release
if [[ "${BUILD_FROM_SOURCE:-}" == "yes" ]]; then
    echo "Building from source..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Rust not found. Installing...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Clone and build
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    git clone "https://$REPO.git"
    cd cassh2rs
    
    echo "Building cassh2rs..."
    cargo build --release
    
    # Copy binary
    mkdir -p "$INSTALL_DIR"
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    
    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"
else
    echo "Downloading latest release..."
    
    # Get latest release URL
    LATEST_URL="https://api.github.com/repos/casapps/cassh2rs/releases/latest"
    DOWNLOAD_URL=$(curl -s "$LATEST_URL" | grep "browser_download_url.*$BINARY_FILE" | cut -d '"' -f 4)
    
    if [[ -z "$DOWNLOAD_URL" ]]; then
        echo -e "${RED}Could not find binary for $OS/$ARCH${NC}"
        echo "Try building from source: BUILD_FROM_SOURCE=yes $0"
        exit 1
    fi
    
    # Download binary
    TEMP_FILE=$(mktemp)
    echo "Downloading from: $DOWNLOAD_URL"
    curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"
    
    # Install binary
    mkdir -p "$INSTALL_DIR"
    mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
fi

# Verify installation
if [[ -x "$INSTALL_DIR/$BINARY_NAME" ]]; then
    echo -e "${GREEN}✓ Successfully installed cassh2rs to $INSTALL_DIR/$BINARY_NAME${NC}"
    
    # Check if install dir is in PATH
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo
        echo -e "${BLUE}Add $INSTALL_DIR to your PATH:${NC}"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo
        echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
    fi
    
    # Show version
    echo
    "$INSTALL_DIR/$BINARY_NAME" --version
else
    echo -e "${RED}Installation failed${NC}"
    exit 1
fi

echo
echo -e "${GREEN}Installation complete!${NC}"
echo
echo "Get started:"
echo "  cassh2rs --help"
echo "  cassh2rs your_script.sh"
echo "  cassh2rs your_script.sh --wizard"
echo
echo "Documentation: https://$REPO"