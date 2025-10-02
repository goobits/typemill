# CodeBuddy Makefile
# Simple build automation for common development tasks

.PHONY: build release test install uninstall clean setup help

# Default target
build:
	cd rust && cargo build

# Optimized release build
release:
	cd rust && cargo build --release

# Run all tests
test:
	cd rust && cargo test

# Install to ~/.local/bin (ensure it's in your PATH)
install: release
	@mkdir -p ~/.local/bin
	@cp rust/target/release/codebuddy ~/.local/bin/
	@echo "✓ Installed to ~/.local/bin/codebuddy"
	@echo ""
	@echo "Make sure ~/.local/bin is in your PATH:"
	@echo "  echo 'export PATH=\"\$$HOME/.local/bin:\$$PATH\"' >> ~/.bashrc"

# Uninstall from ~/.local/bin
uninstall:
	@rm -f ~/.local/bin/codebuddy
	@echo "✓ Removed ~/.local/bin/codebuddy"

# Clean build artifacts
clean:
	cd rust && cargo clean

# One-time developer setup (installs sccache and mold)
setup:
	@./scripts/setup-dev-tools.sh

# Show available commands
help:
	@echo "CodeBuddy - Available Commands"
	@echo "================================"
	@echo ""
	@echo "Build & Install:"
	@echo "  make build    - Build debug version"
	@echo "  make release  - Build optimized release version"
	@echo "  make install  - Install to ~/.local/bin (run after 'make release')"
	@echo "  make uninstall- Remove installed binary"
	@echo ""
	@echo "Development:"
	@echo "  make test     - Run all tests"
	@echo "  make clean    - Remove build artifacts"
	@echo "  make setup    - Install build optimization tools (sccache, mold)"
	@echo ""
	@echo "Usage:"
	@echo "  make setup    # First time only"
	@echo "  make          # Build and develop"
	@echo "  make test     # Test your changes"
	@echo "  make install  # Install to system"
