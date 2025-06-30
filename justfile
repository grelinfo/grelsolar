# Justfile (alternative to Makefile)

export RUSTFLAGS := "-D warnings"
export CARGO_INCREMENTAL := "1"

# Display help message
default:
    just help

# Setup the local development environment
setup:
    @if ! command -v cargo >/dev/null 2>&1; then echo "Error: cargo is not installed. Please install Rust and Cargo first."; exit 1; fi
    @if ! command -v uv >/dev/null 2>&1; then echo "Error: uv is not installed. Please install uv (https://github.com/astral-sh/uv)."; exit 1; fi
    cargo install --path . --locked
    cargo install cargo-watch --locked
    cargo install cargo-edit --locked
    @if [ ! -f .env ]; then cp .env.example .env; echo "Please adapt .env"; fi
    @echo "Environment setup complete."

# Run the application
run:
    cargo run --locked

# Run the application in development mode
dev:
    cargo watch -x run

# Build the project (debug)
build:
    cargo build --workspace --all-targets --locked

# Update dependencies
update:
    cargo update

# Run all tests (unit + integration)
test:
    cargo nextest run --workspace --all-targets --all-features --profile ci --locked

# Lint code
lint:
    cargo clippy --all-targets --all-features --

# Fix code
fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty

# Format code
format:
    cargo fmt --all

# Format code (check only, dry run)
format-check:
    cargo fmt --all -- --check

# Create a release build (optimized)
release:
    cargo build --release --locked

version:
    @uvx dunamai from git --pattern default-unprefixed --style semver


# Run all quality checks
@ci:
    just version
    just lint
    just format-check
    just test

# Show help
help:
    @just --list
