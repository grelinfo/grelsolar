.PHONY: all setup clean

all: setup

setup:
	@echo "Setting up the Rust project environment..."
	@which rustup > /dev/null || (echo "rustup not found. Please install rustup." && exit 1)
	@which pre-commit > /dev/null || (echo "pre-commit not found. Please install pre-commit." && exit 1)
	@pre-commit install
	@rustup update
	# @rustup component add rustfmt
	# @rustup component add clippy
	# @rustup component add llvm-tools-preview
	@cargo install cargo-nextest --locked
	@cargo install cargo-llvm-cov --locked

clean:
	@cargo clean
	@rm -rf target
