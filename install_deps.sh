#!/bin/bash

# SnappiPay Contracts - Dependency Installation Script
# This script installs all required tools for multi-blockchain smart contracts 

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "\n${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}\n"
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

install_rust() {
    print_header "Installing Rust & Cargo"
    
    if command_exists cargo; then
        print_success "Rust/Cargo already installed"
        cargo --version
        return
    fi
    
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed successfully"
}

install_foundry() {
    print_header "Installing Foundry"
    
    if command_exists forge; then
        print_success "Foundry already installed"
        forge --version
        return
    fi
    
    print_status "Installing Foundry..."
    curl -L https://foundry.paradigm.xyz | bash
    source ~/.bashrc
    ~/.foundry/bin/foundryup
    
    # Add to PATH for current session
    export PATH="$HOME/.foundry/bin:$PATH"
    print_success "Foundry installed successfully"
}

install_aptos_cli() {
    print_header "Installing Aptos CLI"
    
    if command_exists aptos; then
        print_success "Aptos CLI already installed"
        aptos --version
        return
    fi
    
    print_status "Installing Aptos CLI..."
    curl -fsSL "https://aptos.dev/scripts/install_cli.py" | python3
    
    # Add to PATH for current session
    export PATH="$HOME/.local/bin:$PATH"
    print_success "Aptos CLI installed successfully"
}

install_scarb() {
    print_header "Installing Scarb (Cairo)"
    
    if command_exists scarb; then
        print_success "Scarb already installed"
        scarb --version
        return
    fi
    
    print_status "Installing Scarb..."
    curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh | sh
    
    # Add to PATH for current session
    export PATH="$HOME/.local/bin:$PATH"
    print_success "Scarb installed successfully"
}

install_stellar_cli() {
    print_header "Installing Stellar CLI"
    
    if command_exists stellar; then
        print_success "Stellar CLI already installed"
        stellar --version
        return
    fi
    
    print_status "Installing Stellar CLI..."
    
    # Detect OS and architecture
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case $ARCH in
        x86_64) ARCH="x86_64" ;;
        arm64|aarch64) ARCH="aarch64" ;;
        *) print_error "Unsupported architecture: $ARCH"; return 1 ;;
    esac
    
    case $OS in
        linux) OS="linux" ;;
        darwin) OS="macos" ;;
        *) print_error "Unsupported OS: $OS"; return 1 ;;
    esac
    
    # Download and install Stellar CLI
    STELLAR_VERSION="21.0.0"  # Latest stable version
    STELLAR_URL="https://github.com/stellar/stellar-cli/releases/download/v${STELLAR_VERSION}/stellar-cli-${STELLAR_VERSION}-${ARCH}-unknown-${OS}-gnu.tar.gz"
    
    if [ "$OS" = "macos" ]; then
        STELLAR_URL="https://github.com/stellar/stellar-cli/releases/download/v${STELLAR_VERSION}/stellar-cli-${STELLAR_VERSION}-${ARCH}-apple-darwin.tar.gz"
    fi
    
    print_status "Downloading Stellar CLI from: $STELLAR_URL"
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Download and extract
    curl -L "$STELLAR_URL" -o stellar-cli.tar.gz
    tar -xzf stellar-cli.tar.gz
    
    # Install to ~/.local/bin
    mkdir -p ~/.local/bin
    cp stellar ~/.local/bin/
    chmod +x ~/.local/bin/stellar
    
    # Clean up
    cd - > /dev/null
    rm -rf "$TEMP_DIR"
    
    # Add to PATH for current session
    export PATH="$HOME/.local/bin:$PATH"
    
    print_success "Stellar CLI installed successfully"
}

install_node_deps() {
    print_header "Installing Node.js Dependencies"
    
    # Check if any project needs Node.js
    if [ -f "package.json" ] || find . -name "package.json" -type f | grep -q .; then
        if command_exists npm; then
            print_status "Installing npm dependencies..."
            npm install
            print_success "Node.js dependencies installed"
        else
            print_warning "Node.js not found but package.json exists. Please install Node.js manually."
        fi
    else
        print_status "No Node.js dependencies found, skipping..."
    fi
}

update_shell_profile() {
    print_header "Updating Shell Profile"
    
    # Add paths to shell profile
    SHELL_PROFILE=""
    if [ -f ~/.bashrc ]; then
        SHELL_PROFILE=~/.bashrc
    elif [ -f ~/.zshrc ]; then
        SHELL_PROFILE=~/.zshrc
    elif [ -f ~/.profile ]; then
        SHELL_PROFILE=~/.profile
    fi
    
    if [ -n "$SHELL_PROFILE" ]; then
        print_status "Updating $SHELL_PROFILE..."
        
        # Add Rust
        if ! grep -q 'source ~/.cargo/env' "$SHELL_PROFILE"; then
            echo 'source ~/.cargo/env' >> "$SHELL_PROFILE"
        fi
        
        # Add Foundry
        if ! grep -q '.foundry/bin' "$SHELL_PROFILE"; then
            echo 'export PATH="$HOME/.foundry/bin:$PATH"' >> "$SHELL_PROFILE"
        fi
        
        # Add local bin (for Aptos CLI and Scarb)
        if ! grep -q '.local/bin' "$SHELL_PROFILE"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_PROFILE"
        fi
        
        print_success "Shell profile updated"
        print_warning "Please run 'source $SHELL_PROFILE' or restart your terminal"
    fi
}

main() {
    print_header "RampContractFoldes - Dependency Installation"
    print_status "This script will install all required tools for the multi-blockchain project"
    
    # Install all dependencies
    install_rust
    install_foundry
    install_aptos_cli
    install_scarb
    install_stellar_cli
    install_node_deps
    update_shell_profile
    
    print_header "Installation Summary"
    print_status "Checking installed tools..."
    
    # Check what's installed
    echo "Rust/Cargo: $(command_exists cargo && echo "✅ Installed" || echo "❌ Missing")"
    echo "Foundry: $(command_exists forge && echo "✅ Installed" || echo "❌ Missing")"
    echo "Aptos CLI: $(command_exists aptos && echo "✅ Installed" || echo "❌ Missing")"
    echo "Scarb: $(command_exists scarb && echo "✅ Installed" || echo "❌ Missing")"
    echo "Stellar CLI: $(command_exists stellar && echo "✅ Installed" || echo "❌ Missing")"
    
    print_success "Installation completed!"
    print_status "You can now run './build_all.sh' to build all projects"
}

main "$@"
