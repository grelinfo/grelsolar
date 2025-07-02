FROM rust:1.88 AS builder
WORKDIR /app

# Copy dependency information first
COPY Cargo.toml Cargo.lock ./

# Create empty source file to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Now copy the real source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime image
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/grelsolar ./
CMD ["./grelsolar"]
