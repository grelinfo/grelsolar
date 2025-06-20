# grelsolar

A Rust application for bridging SolarLog and Home Assistant, providing solar power, energy, and status data from SolarLog devices to Home Assistant via HTTP API.

## Features
- Polls SolarLog for power, energy, and status data
- Integrates with Home Assistant via HTTP API
- Configurable polling periods and endpoints
- Docker-ready and CI/CD enabled

## Development

### Prerequisites

- Rust toolchain (see [rustup.rs](https://rustup.rs/))
- Docker (optional, for containerized deployment)

### Installation

Install dependencies and build:
```sh
make setup
cargo build --release
```

### Configuration

Copy the example environment file and edit as needed:
```sh
cp .env.example .env
```

Edit `.env` to set your SolarLog and Home Assistant credentials and endpoints.

#### Example `.env` file:
```dotenv
APP_LOG=info
APP_LOG_STYLE=always
SOLARLOG_URL=http://192.168.1.2
SOLARLOG_PASSWORD=your_solarlog_password
HOME_ASSISTANT_URL=http://homeassistant.local:8123
HOME_ASSISTANT_TOKEN=your_long_lived_token
```

### Running

#### Native
```sh
cargo run --release
```

#### Docker
Build and run the Docker image:
```sh
docker build -t grelsolar .
docker run --env-file .env grelsolar
```

## CI/CD
- Automated tests and linting via GitHub Actions
- Release workflow builds and pushes Docker images to Docker Hub

## Changelog
See [CHANGELOG.md](CHANGELOG.md) for release notes.

## License
MIT
