# grelsolar

A Rust application for bridging SolarLog and Home Assistant, providing solar power, energy, and status data from SolarLog devices to Home Assistant via HTTP API.


[![Crates.io](https://img.shields.io/crates/v/grelsolar)](https://crates.io/crates/grelsolar)
[![Docs.rs](https://img.shields.io/docsrs/grelsolar)](https://docs.rs/grelsolar)
[![CI](https://github.com/grelinfo/grelsolar/actions/workflows/ci.yml/badge.svg)](https://github.com/grelinfo/grelsolar/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/grelinfo/grelsolar/graph/badge.svg?token=GDFY0AEFWR)](https://codecov.io/gh/grelinfo/grelsolar)
[![Docker Hub](https://img.shields.io/docker/pulls/grelinfo/grelsolar)](https://hub.docker.com/r/grelinfo/grelsolar)


## Features
- Polls SolarLog for power, energy, and status data
- Integrates with Home Assistant via HTTP API
- Configurable polling periods and endpoints
- Docker-ready and CI/CD enabled

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and contribution guidelines.

## Configuration

Copy the example environment file and edit as needed:
```sh
cp .env.example .env
```

Set the following environment variables (for both native and Docker usage):

| Variable                  | Description                        | Example                        |
|---------------------------|------------------------------------|--------------------------------|
| `SOLARLOG_URL`            | URL of your SolarLog device        | `http://192.168.1.10`          |
| `SOLARLOG_PASSWORD`       | Password for SolarLog              | `secret`                       |
| `HOMEASSISTANT_URL`       | URL of Home Assistant API          | `http://192.168.1.20:8123`     |
| `HOMEASSISTANT_TOKEN`     | Long-lived access token            | `eyJ0eXAiOiJKV1QiLCJhbGci...`  |
| `SYNC_POWER_INTERVAL`     | Power sync interval (default: 5s)  | `10s`                          |
| `SYNC_ENERGY_INTERVAL`    | Energy sync interval (default: 60s)| `120s`                         |
| `SYNC_STATUS_INTERVAL`    | Status sync interval (default: 60s)| `60s`                          |

### Running

#### Native
```sh
cargo run --release
```

#### Docker (Recommended)

```sh
docker run --rm \
  -e SOLARLOG_URL="http://your-solarlog" \
  -e SOLARLOG_PASSWORD="your_password" \
  -e HOMEASSISTANT_URL="http://your-homeassistant" \
  -e HOMEASSISTANT_TOKEN="your_token" \
  grelinfo/grelsolar:latest
```

##### Example: Docker Compose

```yaml
services:
  grelsolar:
    image: grelinfo/grelsolar:latest
    restart: unless-stopped
    environment:
      SOLARLOG_URL: "http://192.168.1.10"
      SOLARLOG_PASSWORD: "secret"
      HOMEASSISTANT_URL: "http://192.168.1.20:8123"
      HOMEASSISTANT_TOKEN: "your_token"
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes.

## License

The grelsolar project is dual-licensed (see [LICENSE.md](LICENSE.md)):
- Apache License, Version 2.0
- MIT license
