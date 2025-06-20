FROM rust:1 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/grelsolar ./
CMD ["./grelsolar"]
