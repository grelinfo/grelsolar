FROM rust:1.87-bullseye AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m solarlog-hass-bridge
WORKDIR /app
COPY --from=builder /app/target/release/solarlog-hass-bridge /app/solarlog-hass-bridge
RUN chown solarlog-hass-bridge:solarlog-hass-bridge /app/solarlog-hass-bridge && chmod +x /app/solarlog-hass-bridge
USER solarlog-hass-bridge
CMD ["./solarlog-hass-bridge"]