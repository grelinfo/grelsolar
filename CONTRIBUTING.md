# Contributing to grelsolar

Thank you for your interest in contributing!

## Development Prerequisites

- Rust toolchain (see [rustup.rs](https://rustup.rs/))
- Docker (optional, for containerized deployment)

## Setup

```sh
make setup
cargo build --release
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
APP_LOG=info
APP_LOG_STYLE=always
SOLARLOG_URL=http://192.168.1.2
SOLARLOG_PASSWORD=your_solarlog_password
HOMEASSISTANT_URL=http://homeassistant.local:8123
HOMEASSISTANT_TOKEN=your_long_lived_token
```

## CI/CD

- Automated tests, linting, and code coverage are run via GitHub Actions on every push and pull request.
- Release workflow builds and pushes Docker images to Docker Hub on new tags and main branch updates.
- Coverage reports are uploaded to Codecov.
- See `.github/workflows/ci.yml` for details on the CI/CD pipeline.

## Pull Requests

- Please open an issue before submitting major changes.
- Ensure all tests and lints pass before submitting a PR.
- Follow Rust best practices and keep code well-documented.
