#!/bin/bash
set -e

# Codeflow Buddy MCP Server Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/username/codeflow-buddy/main/install.sh | bash

REPO="username/codeflow-buddy"
BINARY_NAME="codeflow-buddy"
INSTALL_DIR="$HOME/.local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case $os in
        linux*) os="linux" ;;
        darwin*) os="macos" ;;
        msys*|mingw*|cygwin*) os="windows" ;;
        *) log_error "Unsupported OS: $os"; exit 1 ;;
    esac

    case $arch in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        armv7l) arch="armv7" ;;
        *) log_error "Unsupported architecture: $arch"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# Download and extract binary
install_binary() {
    local platform=$1
    local download_url="https://github.com/${REPO}/releases/latest/download/codeflow-buddy-${platform}.tar.gz"
    local temp_dir=$(mktemp -d)

    log_info "Downloading codeflow-buddy for ${platform}..."

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$download_url" -o "$temp_dir/codeflow-buddy.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$download_url" -O "$temp_dir/codeflow-buddy.tar.gz"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    log_info "Extracting binary..."
    tar -xzf "$temp_dir/codeflow-buddy.tar.gz" -C "$temp_dir"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Install binary
    cp "$temp_dir/codeflow-buddy" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    # Cleanup
    rm -rf "$temp_dir"

    log_success "Binary installed to $INSTALL_DIR/$BINARY_NAME"
}

# Update PATH if needed
update_path() {
    local shell_config=""

    # Detect shell and config file
    case $SHELL in
        */bash) shell_config="$HOME/.bashrc" ;;
        */zsh) shell_config="$HOME/.zshrc" ;;
        */fish) shell_config="$HOME/.config/fish/config.fish" ;;
        *) shell_config="$HOME/.profile" ;;
    esac

    # Check if PATH already contains install dir
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        log_info "PATH already contains $INSTALL_DIR"
        return
    fi

    # Add to PATH
    echo "" >> "$shell_config"
    echo "# Added by codeflow-buddy installer" >> "$shell_config"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_config"

    log_success "Added $INSTALL_DIR to PATH in $shell_config"
    log_warning "Please restart your shell or run: source $shell_config"
}

# Setup MCP configuration
setup_mcp() {
    local claude_config_dir="$HOME/.config/claude-code"
    local mcp_config="$claude_config_dir/mcp.json"

    # Create config directory
    mkdir -p "$claude_config_dir"

    # Create or update MCP config
    if [ -f "$mcp_config" ]; then
        log_info "MCP config exists, creating backup..."
        cp "$mcp_config" "$mcp_config.backup.$(date +%s)"
    fi

    # Generate MCP configuration
    cat > "$mcp_config" << 'EOF'
{
  "mcpServers": {
    "codeflow-buddy": {
      "command": "codeflow-buddy",
      "args": ["start"]
    }
  }
}
EOF

    log_success "MCP configuration created at $mcp_config"
}

# Run setup command
run_setup() {
    log_info "Running codeflow-buddy setup..."

    # Add install dir to current PATH for this session
    export PATH="$INSTALL_DIR:$PATH"

    if command -v codeflow-buddy >/dev/null 2>&1; then
        codeflow-buddy setup
        log_success "Setup completed!"
    else
        log_warning "Binary not found in PATH. Please restart your shell and run: codeflow-buddy setup"
    fi
}

# Main installation flow
main() {
    log_info "Installing Codeflow Buddy MCP Server..."

    # Check dependencies
    if ! command -v tar >/dev/null 2>&1; then
        log_error "tar is required but not installed."
        exit 1
    fi

    # Detect platform and install
    local platform=$(detect_platform)
    log_info "Detected platform: $platform"

    install_binary "$platform"
    update_path
    setup_mcp
    run_setup

    echo ""
    log_success "🎉 Codeflow Buddy MCP Server installed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Restart your shell or run: source ~/.bashrc (or your shell config)"
    echo "2. In any project: codeflow-buddy setup"
    echo "3. Start using with Claude Code!"
    echo ""
}

# Run installer
main "$@"