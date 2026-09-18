# Contributing to grelsolar

Thank you for your interest in contributing!

## Development Prerequisites

- Rust toolchain (see [rustup.rs](https://rustup.rs/)). The version is pinned in `rust-toolchain.toml` and installed automatically.
- [just](https://github.com/casey/just) to run the project commands
- [uv](https://docs.astral.sh/uv/) to install the pre-commit hooks
- Docker (optional, for containerized deployment)

## Setup

```sh
just setup
just build
```

## Running Tests

```sh
just test
```

## Linting and Formatting

```sh
just lint
just format-check
```

## Configuration

Copy the example environment file and edit as needed:

```sh
cp .env.example .env
```

Edit `.env` to set your SolarLog and Home Assistant credentials and endpoints.

#### Example `.env` file

```dotenv
APP_LOG=debug
APP_LOG_STYLE=always
SOLARLOG_URL=http://192.168.1.2
SOLARLOG_PASSWORD=your_solarlog_password
HOMEASSISTANT_URL=http://homeassistant.local:8123
HOMEASSISTANT_TOKEN=your_long_lived_token
```

## Docker Image

```sh
just docker-build
just docker-run
```

The Dockerfile builds a static binary and copies it into a distroless image (no shell, runs as non-root).
Images for other architectures are cross-compiled with [xx](https://github.com/tonistiigi/xx), not emulated:

```sh
docker buildx build --platform linux/amd64,linux/arm64 .
```

## CI/CD

- Every push and pull request runs linting, tests and coverage, and builds the Docker image for amd64 and arm64.
- Coverage reports are uploaded to Codecov from `main` and tags.
- A version tag (for example `0.3.0`) publishes the crate to crates.io and pushes the Docker images to Docker Hub.
- See `.github/workflows/ci.yml` for details on the CI/CD pipeline.

## Dependency Updates

[Renovate](https://docs.renovatebot.com/) keeps dependencies up to date (see `renovate.json`):

- Once a month, one pull request groups all minor and patch updates and merges itself when CI passes.
- Each major update gets its own pull request to review.
- The Rust version is updated in `rust-toolchain.toml` and the `Dockerfile` together.
- Security updates are opened immediately.

## Commit Messages

Commit messages start with a [gitmoji](https://gitmoji.dev/), for example `🐛 Fix token refresh`.
The subject has at most 50 characters and body lines at most 72. The pre-commit hook checks this.

## Pull Requests

- Please open an issue before submitting major changes.
- Ensure all tests and lints pass before submitting a PR.
- Follow Rust best practices and keep code well-documented.
