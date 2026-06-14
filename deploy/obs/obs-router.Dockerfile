FROM rust:1.88-bookworm AS builder

WORKDIR /src
COPY services ./services
WORKDIR /src/services
RUN cargo build --release -p cyberos-obs-router --bin cyberos-obs-router

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates wget \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/services/target/release/cyberos-obs-router /usr/local/bin/cyberos-obs-router

ENTRYPOINT ["cyberos-obs-router"]
