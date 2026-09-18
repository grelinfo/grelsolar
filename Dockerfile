# syntax=docker/dockerfile:1

# Cross-compilation helpers (xx-cargo, xx-apk, xx-verify)
FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.9.0@sha256:c64defb9ed5a91eacb37f96ccc3d4cd72521c4bd18d5442905b95e2226b0e707 AS xx

# Build a static musl binary natively on the build host, for the target platform
FROM --platform=$BUILDPLATFORM rust:1.98-alpine AS builder
COPY --from=xx / /
RUN apk add --no-cache clang lld
ARG TARGETPLATFORM
RUN xx-apk add --no-cache gcc musl-dev
WORKDIR /app
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target,id=target-$TARGETPLATFORM \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry \
    xx-cargo build --locked --release --bin grelsolar && \
    cp target/$(xx-cargo --print-target-triple)/release/grelsolar /grelsolar && \
    xx-verify --static /grelsolar

# Runtime image: static binary, CA certificates, nonroot user
FROM gcr.io/distroless/static-debian13:nonroot
COPY --from=builder /grelsolar /grelsolar
ENTRYPOINT ["/grelsolar"]
