# FR-EMAIL-001 gateway image for the Rust adapter/API service.
#
# Compose builds this file from the repository root so Cargo can see the full
# services workspace and shared crates.

FROM rust:1.88-bookworm AS builder
WORKDIR /workspace/services

COPY services ./

RUN cargo build --release -p cyberos-email --bin cyberos-email

FROM debian:bookworm-slim
RUN groupadd --system cyberos && \
    useradd --system --gid cyberos --home-dir /nonexistent --shell /usr/sbin/nologin cyberos && \
    apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /workspace/services/target/release/cyberos-email /usr/local/bin/cyberos-email

USER cyberos
EXPOSE 8085
ENTRYPOINT ["/usr/local/bin/cyberos-email"]
