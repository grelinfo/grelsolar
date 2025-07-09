# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
