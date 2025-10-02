# Justfile for CodeBuddy development tasks
# Install just: https://github.com/casey/just

# Default recipe to display help
default:
    @just --list

# Run all checks (clippy, format, test)
check: clippy fmt-check test

# Run clippy with strict settings
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test --all-features

# Build release binary
build:
    cargo build --release

# Build development binary
build-dev:
    cargo build

# Run the server
run *ARGS:
    cargo run --bin codebuddy -- {{ARGS}}

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Check for duplicate dependencies (helps prevent bloat)
deps-check:
    @echo "Checking for duplicate dependencies..."
    @cargo tree --duplicates

# Full dependency tree
deps-tree:
    cargo tree

# Check for outdated dependencies
deps-outdated:
    cargo outdated

# Clean build artifacts
clean:
    cargo clean

# Run benchmarks
bench:
    cargo bench

# Generate documentation
docs:
    cargo doc --no-deps --open

# Run all quality checks before commit
pre-commit: fmt clippy test deps-check
    @echo "✅ All checks passed!"

# Development setup
setup:
    @echo "Setting up development environment..."
    @echo "Installing Rust components..."
    rustup component add clippy rustfmt
    @echo "Checking for required tools..."
    @which just > /dev/null || echo "⚠️  Install 'just' command runner"
    @which cargo-outdated > /dev/null || echo "💡 Consider: cargo install cargo-outdated"
    @echo "✅ Setup complete!"

# Run security audit (requires cargo-audit)
audit:
    cargo audit

# Watch for changes and run tests
watch:
    cargo watch -x test

# Profile binary size
bloat:
    cargo bloat --release

# Show crate sizes
bloat-crates:
    cargo bloat --release --crates