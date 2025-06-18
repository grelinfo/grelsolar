FROM rust:1.87-bullseye AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:1.87-bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

RUN useradd -m grust

COPY --from=builder --chown=grust:grust target/release/grust /app/grust

USER grust

CMD ["./grust"]