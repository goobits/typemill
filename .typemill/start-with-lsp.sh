#!/bin/bash
# Start mill with LSP servers in PATH

export PATH="$HOME/.cargo/bin:$HOME/.nvm/versions/node/v22.20.0/bin:$PATH"

# Stop any existing mill instance
./target/release/mill stop 2>/dev/null || true

# Start mill server
./target/release/mill start

echo "✅ LSP servers ready in PATH"
echo "   - rust-analyzer: $(which rust-analyzer 2>/dev/null || echo 'not found')"
echo "   - typescript-language-server: $(which typescript-language-server 2>/dev/null || echo 'not found')"
