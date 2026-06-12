# FR-EMAIL-001 §1 #1 + DEC-300 — Stalwart container image.
#
# The CyberOS gateway/adapter (Rust binary `cyberos-email`) runs as a
# separate container — see services/email/Cargo.toml. This Dockerfile
# is the Stalwart server container only.

ARG STALWART_VERSION=v0.10.7

FROM stalwartlabs/mail-server:${STALWART_VERSION}

# Bake the CyberOS Stalwart config + a placeholder DKIM keystore dir.
# At runtime, the `cyberos-email` gateway syncs `dkim_keys` (Postgres)
# into /etc/stalwart/keys/<tenant_id>/cyberos.pem.
COPY stalwart.toml /etc/stalwart/cyberos.toml
RUN mkdir -p /etc/stalwart/keys

# Expose every protocol port spec'd by §1 #4.
EXPOSE 25 465 587 143 993 4190 8080

# Stalwart honours the config path via CLI; our overlay file directs
# storage + listeners + DKIM + MTA-STS + spam threshold.
CMD ["mail-server", "--config", "/etc/stalwart/cyberos.toml"]
