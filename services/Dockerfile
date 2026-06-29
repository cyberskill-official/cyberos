# syntax=docker/dockerfile:1
# Shared multi-stage build for the CyberOS Rust workspace HTTP services (auth, memory, ai-gateway,
# mcp-gateway). Build one binary by passing its Cargo package and bin name; the build CONTEXT must be
# services/ (the Cargo workspace root: services/Cargo.toml + all crates + services/shared/).
#
#   cd services && docker build -f Dockerfile \
#     --build-arg PACKAGE=cyberos-auth --build-arg BIN=cyberos-auth \
#     -t cyberos-auth:local .
#
# The production compose (deploy/vps/docker-compose.yml) passes these args per service with
# build.context = ../../services.

ARG RUST_VERSION=1.88.0

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /build
ARG PACKAGE
ARG BIN
COPY . .
# Cache the registry + target across builds; the binary must be copied out inside the same RUN, before
# the target cache mount is unmounted.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    cargo build --release -p "${PACKAGE}" --bin "${BIN}" \
    && cp "/build/target/release/${BIN}" /usr/local/bin/app

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 cyberos
COPY --from=builder /usr/local/bin/app /usr/local/bin/app
USER cyberos
ENTRYPOINT ["/usr/local/bin/app"]
