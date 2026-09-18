# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### 🚨 Breaking Changes
- Docker: the binary moved to `/grelsolar` and is now the image `ENTRYPOINT`
  (was `./grelsolar` as `CMD`). Update any setup that overrides the command.
- Library: `solarlog::Client::new` and `homeassistant::Client::new` take a request timeout.
- `sensor.solar_energy` no longer has a `last_reset` attribute
  (Home Assistant ignores it for `total_increasing` sensors).

### ✨ Features
- Sensors are set to `unavailable` after 60 s of failed syncs, instead of showing a stale value.
- `HTTP_TIMEOUT` configures the timeout of each HTTP request (default: 500ms).
- `sensor.solar_power` has the `power` device class.

### 🐛 Bug Fixes
- Sensors now come back within 5 minutes after Home Assistant restarts.
- The process exits when the application crashes, so Docker restarts it.
- No more crash when midnight is skipped or repeated by a daylight saving change.
- Docker images had version `0.0.0` in their metadata.

### 🛠 Improvements
- Docker image reduced from 50 MB to about 10 MB, and idle memory to under 1 MiB:
  static binary on distroless, single-threaded runtime and optimized release profile.
- arm64 images are cross-compiled instead of emulated.
- Failures are logged when they start, hourly while they last and when they end,
  instead of on every attempt.
- Missed sync ticks are skipped instead of run in a burst.

### 🔒 Security
- `reqwest` 0.13 uses rustls: OpenSSL is no longer a dependency.
- All dependencies updated; `cargo audit` reports no vulnerabilities.
- Added a security policy (`SECURITY.md`) with private vulnerability reporting.

### 📚 Documentation
- Home Assistant sensors, reliability behavior and exit codes in the README.
- Docker Compose example with a pinned version and log rotation.
- Contributing guide: Docker builds, CI/CD, Renovate and commit message rules.

### 🏗 Chore
- Rust 1.88 → 1.98 and all dependencies to their latest versions.
- GitHub Actions updated to their latest major versions.
- Renovate replaces Dependabot.

## [0.2.0] - 2025-07-09

### 🚨 Breaking Changes
- Standardized environment variable names for consistency.

### ✨ Features
- Graceful shutdown on CTRL+C and SIGTERM.
- Connection pooling and timeouts to HTTP clients.
- Dual-licensed under MIT and Apache-2.0.

### 🐛 Bug Fixes
- Solar energy sync now uses the last day.
- HTTP 411 error on solarlog logout.
- Shutdown and log formatting issues.

### 🛠 Improvements
- Refactored configuration loading using `envconfig`.
- Improved dependency injection, error handling, logging, and memory usage.
- Updated Docker and CI/CD setup; replaced Makefile with justfile.

### 📚 Documentation
- Unified and clarified environment variable documentation.
- Split README and CONTRIBUTING documentation for clarity.

### 🏗 Chore
- Bump `tokio` from 1.45.1 to 1.46.1.
- Bump `reqwest` from 0.12.20 to 0.12.22.

## [0.1.0] - 2025-06-20

### ✨ Features
- Compile-time environment variables for app name and version.
- Distroless Dockerfile and improved Docker setup.
- Release workflow for Docker image management and versioning.
- Dockerfile and docker-compose for HTTP mock service.

### 🐛 Bug Fixes
- Log message for graceful shutdown in main function.
- Login function respects the force parameter in circuit breaker call.
- Dockerfile build process and user permissions.
- Unnecessary login on all queries.
- SolarLog HTTP client token concurrency issue.

### 🛠 Improvements
- CI workflow and pre-commit configuration for Rust project.
- Logger configuration from environment variables.
- Enhanced .env.example and configuration for polling periods.
- Remove unnecessary arguments from cargo check hook in pre-commit config.

### 📚 Documentation
- Example .env file and enhanced configuration loading with defaults.

### 🏗 Chore
- Reorganized CI workflow steps for clarity and structure.
- Renamed project and updated related files.
