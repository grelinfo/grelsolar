FROM rust:1.87-bullseye AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m grelsolar
WORKDIR /app
COPY --from=builder /app/target/release/grelsolar /app/grelsolar
RUN chown grelsolar:grelsolar /app/grelsolar && chmod +x /app/grelsolar
USER grelsolar
CMD ["./grelsolar"]