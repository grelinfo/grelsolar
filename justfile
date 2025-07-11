# Justfile - Single source of truth for all commands

# Common environment variables for local and CI
export RUSTFLAGS := "-D warnings"
export CARGO_TERM_COLOR := "always"

# Setup environment
@setup:
    # Ensure you have Rust and Cargo installed
    rustc --version || (echo "rustc not found. Please install Rust: https://rustup.rs/" && exit 1)
    cargo --version || (echo "cargo not found. Please install Rust: https://rustup.rs/" && exit 1)
    # Ensure you have UV installed
    uv --version || (echo "uv not found. Please install UV: https://docs.astral.sh/uv/getting-started/installation/" && exit 1)
    # Install dependencies
    cargo install --locked cargo-edit cargo-nextest cargo-llvm-cov circleci-junit-fix

# Quick check (fast for local development)
@check:
    cargo check --workspace --all-targets

# Format code
@format:
    cargo fmt --all

# Format check (for CI)
@format-check:
    cargo fmt --all -- --check

# Lint with clippy
@lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fix lint issues
@fix:
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty

# Run tests
@test:
    cargo nextest run --workspace --all-targets --all-features --profile ci --locked
    cat target/nextest/ci/junit.xml | circleci-junit-fix > junit.xml

# Build all targets
@build:
    cargo build --workspace --all-targets --locked

# Generate coverage
@coverage:
    cargo llvm-cov nextest --workspace --all-targets --all-features --locked
    cargo llvm-cov report --lcov --output-path lcov.info --color always

# Pre-commit checks (fast subset for developers)
@pre-commit:
    uvx pre-commit run --all-files

# Get the current version
@version:
    uvx dunamai from git --pattern default-unprefixed --bump --style semver

# Get Docker-compatible version (without + character)
@docker-version:
    uvx dunamai from git --pattern default-unprefixed --bump --style semver | sed 's/+/-/g'

# Build Docker image
@docker-build:
    docker build -t grelsolar:0.0.0 .

# Run Docker container
@docker-run:
    docker run --rm -it -p 8080:8080 --env-file .env grelsolar:0.0.0

# Full CI checks (comprehensive)
@ci:
    SKIP=format,check,lint pre-commit run --all-files
    just format-check
    just lint
    just build
    just test
    just coverage
