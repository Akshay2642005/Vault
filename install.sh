#!/bin/bash

# Vault Password Manager Installation Script
# This script installs the Vault CLI tool

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/vault/vault"
BINARY_NAME="vault"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

case $OS in
    linux) OS="unknown-linux-gnu" ;;
    darwin) OS="apple-darwin" ;;
    *) echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"

echo -e "${BLUE}🔐 Vault Password Manager Installer${NC}"
echo -e "${BLUE}====================================${NC}"
echo ""
echo -e "Target: ${GREEN}$TARGET${NC}"
echo ""

# Check if running as root for system-wide install
if [[ $EUID -eq 0 ]]; then
    echo -e "${YELLOW}⚠️  Running as root - installing system-wide${NC}"
    INSTALL_DIR="/usr/local/bin"
else
    echo -e "${BLUE}ℹ️  Installing to user directory${NC}"
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Check if vault is already installed
if command -v vault &> /dev/null; then
    CURRENT_VERSION=$(vault --version 2>/dev/null | head -n1 || echo "unknown")
    echo -e "${YELLOW}⚠️  Vault is already installed: $CURRENT_VERSION${NC}"
    read -p "Do you want to continue and overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Installation cancelled${NC}"
        exit 0
    fi
fi

# Create temporary directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo -e "${BLUE}📦 Downloading Vault...${NC}"

# For now, we'll build from source since releases aren't set up yet
if command -v cargo &> /dev/null; then
    echo -e "${BLUE}🔨 Building from source...${NC}"
    
    cd "$TMP_DIR"
    git clone "$REPO_URL" vault-src
    cd vault-src/app
    
    echo -e "${BLUE}🔧 Compiling (this may take a few minutes)...${NC}"
    cargo build --release
    
    BINARY_PATH="target/release/$BINARY_NAME"
else
    echo -e "${RED}❌ Cargo not found. Please install Rust first:${NC}"
    echo -e "${BLUE}   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

# Install binary
echo -e "${BLUE}📋 Installing to $INSTALL_DIR...${NC}"

if [[ ! -f "$TMP_DIR/vault-src/app/$BINARY_PATH" ]]; then
    echo -e "${RED}❌ Binary not found after build${NC}"
    exit 1
fi

cp "$TMP_DIR/vault-src/app/$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Verify installation
if command -v vault &> /dev/null; then
    VERSION=$(vault --version 2>/dev/null | head -n1 || echo "unknown")
    echo -e "${GREEN}✅ Vault installed successfully!${NC}"
    echo -e "${GREEN}   Version: $VERSION${NC}"
    echo -e "${GREEN}   Location: $(which vault)${NC}"
else
    echo -e "${RED}❌ Installation failed - vault command not found${NC}"
    echo -e "${YELLOW}💡 You may need to add $INSTALL_DIR to your PATH${NC}"
    echo -e "${BLUE}   Add this to your shell profile (.bashrc, .zshrc, etc.):${NC}"
    echo -e "${BLUE}   export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}🚀 Quick Start:${NC}"
echo -e "${BLUE}===============${NC}"
echo -e "1. Initialize a vault:    ${GREEN}vault init --tenant my-org --admin admin@example.com${NC}"
echo -e "2. Login:                 ${GREEN}vault login --tenant my-org${NC}"
echo -e "3. Store a secret:        ${GREEN}vault put my-secret${NC}"
echo -e "4. Retrieve a secret:     ${GREEN}vault get my-secret${NC}"
echo ""
echo -e "${BLUE}📚 For more information:${NC}"
echo -e "   Documentation: https://vault.dev/docs"
echo -e "   GitHub: $REPO_URL"
echo ""
echo -e "${GREEN}🎉 Happy secret managing!${NC}"