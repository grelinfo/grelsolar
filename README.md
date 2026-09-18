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
- Built to run unattended: recovers from outages on both sides and shows them in Home Assistant
- Small footprint: a ~10 MB Docker image using under 1 MiB of RAM when idle
- Docker images for amd64 and arm64

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
| `HTTP_TIMEOUT`            | Timeout of each HTTP request (default: 500ms) | `2s`                |
| `APP_LOG`                 | Log level (default: info)          | `debug`                        |

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

For a setup that runs unattended, pin a version and rotate the logs:

```yaml
services:
  grelsolar:
    image: grelinfo/grelsolar:0.3 # Receives 0.3.x fixes only, never breaking changes
    restart: unless-stopped # Restarts the container if the application crashes
    environment:
      SOLARLOG_URL: "http://192.168.1.10"
      SOLARLOG_PASSWORD: "secret"
      HOMEASSISTANT_URL: "http://192.168.1.20:8123"
      HOMEASSISTANT_TOKEN: "your_token"
    logging:
      options:
        max-size: "10m" # Rotate the log file at 10 MB
        max-file: "3" # Keep 3 files: at most 30 MB of logs
```

## Home Assistant Sensors

grelsolar creates these sensors in Home Assistant. You do not need to configure them.

| Sensor                | Value                                  | Unit | Home Assistant class                   |
|-----------------------|----------------------------------------|------|----------------------------------------|
| `sensor.solar_power`  | Current power production               | W    | `power`, `measurement`                 |
| `sensor.solar_energy` | Energy produced today, resets at midnight | kWh | `energy`, `total_increasing`         |
| `sensor.solar_status` | Inverter status, for example `On-grid` | -    | -                                      |

`sensor.solar_energy` can be used as solar production in the Home Assistant Energy dashboard.

A value is sent when it changes, and at least every 5 minutes. Home Assistant does not keep these sensors
across its own restarts, so they come back within 5 minutes after Home Assistant restarts.

## Reliability

grelsolar is designed to run for long periods without attention:

| Situation                               | Behavior                                                                  |
|-----------------------------------------|---------------------------------------------------------------------------|
| Request fails                           | Retried up to 3 times with a short, randomized backoff                    |
| Device keeps failing                    | After 5 failures in a row, requests stop for 60 s to let it recover       |
| SolarLog session expires                | Logs in again automatically                                               |
| SolarLog unreachable for 60 s           | Sensors are set to `unavailable`, so no stale value is shown as live      |
| Sync recovers                           | Sensors get their current value back immediately                          |
| Failure lasts                           | Logged once when it starts, once an hour while it lasts, once when it ends |
| Application crashes                     | The process exits with an error, so Docker restarts it (`restart: unless-stopped`) |
| `docker stop` or Ctrl+C                 | Stops cleanly and logs out of SolarLog                                    |

### Exit Codes

| Code | Meaning                                                  |
|------|----------------------------------------------------------|
| 0    | Stopped cleanly                                          |
| 1    | Application crashed                                      |
| 2    | Invalid configuration, for example a missing variable    |
| 3    | Did not stop within 30 s after a stop request            |

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes.

## License

The grelsolar project is dual-licensed (see [LICENSE.md](LICENSE.md)):
- Apache License, Version 2.0
- MIT license
