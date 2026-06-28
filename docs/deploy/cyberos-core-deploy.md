# Deploying the CyberOS core (MEMORY, SKILL, CUO, AUTH, CHAT, PROJ)

Stage 2 go-live runbook for the six core modules. It is grounded in what the repo actually declares
(env vars the binaries read, per-service migrations, the VPS env file, the Cargo workspace). Where the
production wiring lives outside the repo, the step says "confirm" instead of guessing - read those
notes before you run anything.

## The deployment model (what the repo shows)

Production is a Docker Compose stack on a VPS, with Caddy terminating TLS in front. The shape is
declared by `deploy/vps/.env.local`:

- One Postgres (Postgres 16 + pgvector, the official `pgvector/pgvector:pg16` image via
  `services/dev/Dockerfile.postgres`) holding a separate database per service: `AUTH_DB`,
  `MEMORY_DB`, `PROJ_DB`, `CHAT_POSTGRES_DB`. Redis alongside it.
- Caddy reverse proxy and TLS: `CYBEROS_CADDY_SITE`, `CYBEROS_HTTP_BIND`, `CYBEROS_HTTPS_BIND`.
- Image-tag pinning: `CYBEROS_IMAGE_TAG`, `CYBEROS_POSTGRES_IMAGE_TAG`, `CYBEROS_PGVECTOR_VERSION`,
  `CYBEROS_RUST_VERSION`; chat is pinned separately (`CHAT_IMAGE_TAG`, `CHAT_PINNED_COMMIT`).

Local infra equivalent for testing the steps: `services/dev/docker-compose.yml` boots the same
Postgres 16 (pgvector) + Redis on 5432 / 6379, seeded by `services/dev/postgres-init.sql`.

### What each core module is, in deploy terms

| Module | Code | Runs as | DB | Migrations |
|---|---|---|---|---|
| AUTH | `services/auth` (Rust, `main.rs`) | HTTP service, binds `AUTH_LISTEN_ADDR` | `AUTH_DB` | `services/auth/migrations` |
| MEMORY | `services/memory` (Rust, `main.rs`) + `modules/memory` (Python) | HTTP service, binds `MEMORY_LISTEN_ADDR`; needs an embeddings endpoint `MEMORY_EMBED_URL` (see `services/embed-sidecar`) | `MEMORY_DB` | `services/memory/migrations` |
| PROJ | `services/proj` (Rust, library `cyberos_proj` - no `main.rs`) | linked into a host binary, not a standalone server - confirm which binary serves the proj API | `PROJ_DB` | `services/proj/migrations` |
| SKILL | `services/skill-broker` (Rust, `[[bin]]`) | broker service; skills ship as OCI artifacts (FR-SKILL-201-oci-registry-deploy) | - | - |
| CHAT | separate service, `services/chat/Dockerfile` | pinned container in the compose (`CHAT_IMAGE_TAG`); Fargate is an alternative (FR-CHAT-003-fargate-deployment) | `CHAT_POSTGRES_DB` | - |
| CUO | `modules/cuo` (Python, `[project.scripts]`) | orchestration runtime (personas + workflows incl. ship-feature-requests); launches via its console script | - | - |

Two of the six are not plain Rust servers: PROJ is a library (its HTTP surface is served by another
binary), and CUO is the Python orchestration layer. Treat those two specially.

## Prerequisites

- A VPS with Docker + Docker Compose v2, a domain pointed at it (for Caddy TLS and for
  `AUTH_WEBAUTHN_RP_ORIGIN` / `AUTH_WEBAUTHN_RP_ID`, which must match the public origin or WebAuthn
  login breaks).
- To build from source rather than pull images: Rust `1.88` (`rustup toolchain install 1.88.0`) and
  the Python env for `modules/cuo`.
- Postgres 16 with pgvector (Apache AGE removed; the memory graph is the relational l2_edge table). The
  dev image is the official `pgvector/pgvector:pg16` via `services/dev/Dockerfile.postgres`; pgcrypto,
  uuid-ossp, and vector are then enabled by
  `services/dev/postgres-init.sql` on first init. In production, mirror this - the base AGE image alone
  will fail the memory layer-2 migration (`VECTOR(1024)` columns). Redis 7.

## Step 1 - secrets and env

Production config is a single env file, `deploy/vps/.env.local` (consumed by the VPS compose). Its key
groups: per-service `DATABASE_URL`s (`AUTH_DATABASE_URL`, `MEMORY_DATABASE_URL`, `PROJ_DATABASE_URL`)
and DB names; `AUTH_JWT_ISSUER`, `AUTH_CURSOR_SIGNING_SECRET`; the Caddy site + binds; image tags; and
the chat block (`CHAT_POSTGRES_*`, `CHAT_SITE_URL`, `CHAT_IMAGE_TAG`).

Generate the signing secrets fresh on the host, never commit them:

```bash
openssl rand -base64 48   # AUTH_CURSOR_SIGNING_SECRET
openssl rand -base64 48   # CYBEROS_AI_OPERATOR_SECRET (if the AI gateway is in scope)
```

Security note (do this first): `deploy/obs/auth/collector.token.live`, `deploy/obs/auth/tokens.live`,
and `deploy/vps/.env.local` are live-credential files in the working tree. Confirm they are gitignored;
if any is tracked, rotate the token and remove it from history. Keep `.env.local` out of git and on the
VPS only.

The full env surface the Rust binaries read (from the source): `DATABASE_URL`, `AUTH_LISTEN_ADDR`,
`AUTH_JWT_ISSUER`, `AUTH_CURSOR_SIGNING_SECRET`, `AUTH_WEBAUTHN_RP_ID` / `_RP_NAME` / `_RP_ORIGIN`,
`AUTH_BOOTSTRAP_EMAIL` / `_PASSWORD`, `AUTH_RBAC_REFRESH_SECS`, `AUTH_GEOIP_DB` / `_ANONYMOUS_DB` /
`_REQUIRED`, `MEMORY_LISTEN_ADDR`, `MEMORY_EMBED_URL`, `MEMORY_INGEST_BATCH_SIZE`, `MEMORY_TAIL_POLL_MS`,
`MEMORY_TENANTS`, `EMAIL_BIND`, `CYBEROS_DEPLOYMENT_TIER`, `CYBEROS_AI_OPERATOR_SECRET` / `_TOKEN`,
`CYBEROS_AI_EXPIRY_TICK_SECONDS`, `AI_GATEWAY_BIND`, `AI_GATEWAY_CONFIG_DIR`, `LMSTUDIO_ENDPOINT`,
`AI_GATEWAY_FAILOVER_BUDGET_SECS` / `AI_GATEWAY_PROVIDER_TIMEOUT_SECS` (both default 30s; raise for slow
local reasoning models, kept low otherwise).

## Step 2 - database

```bash
# Bring up Postgres + Redis (local-dev infra; mirror the image/tags in production .env.local).
cd services/dev && docker compose up -d && cd -

# Create one database per service (names come from *_DB in .env.local).
for db in "$AUTH_DB" "$MEMORY_DB" "$PROJ_DB" "$CHAT_POSTGRES_DB"; do
  psql "$ADMIN_DATABASE_URL" -c "CREATE DATABASE \"$db\";" || true
done
# Extensions (pgvector, AGE, pgcrypto, uuid-ossp) are applied by services/dev/postgres-init.sql on
# first init; for an existing cluster, run that file against each DB.
```

Run migrations per service (sqlx, the `migrate` feature is enabled in the workspace):

```bash
# Needs sqlx-cli: cargo install sqlx-cli --no-default-features --features rustls,postgres
DATABASE_URL="$AUTH_DATABASE_URL"   sqlx migrate run --source services/auth/migrations
DATABASE_URL="$MEMORY_DATABASE_URL" sqlx migrate run --source services/memory/migrations
DATABASE_URL="$PROJ_DATABASE_URL"   sqlx migrate run --source services/proj/migrations
# email / ai-gateway / mcp-gateway also carry migrations if those services are in scope.
```

## Step 3 - build (or pull)

```bash
# From source:
cargo build --release -p cyberos-auth -p cyberos-memory -p cyberos-skill-broker
# proj is a library - it builds as part of whatever binary embeds it (confirm which).
# Or pull the tagged images referenced by CYBEROS_IMAGE_TAG / CHAT_IMAGE_TAG in .env.local.

# CUO (Python orchestration):
cd modules/cuo && pip install -e . && cd -   # exposes the [project.scripts] entrypoint
```

## Step 4 - bring services up in dependency order

Order: infra (done) then AUTH (identity) -> MEMORY (audit chain everything writes to) -> PROJ and
SKILL -> CHAT -> CUO (it orchestrates the others, so last). For each Rust service, set its env, run the
release binary (or `docker compose up -d <service>` against the production compose), and confirm it is
listening.

```bash
# AUTH
AUTH_DATABASE_URL=... AUTH_LISTEN_ADDR=0.0.0.0:8090 AUTH_JWT_ISSUER=https://<domain> \
AUTH_WEBAUTHN_RP_ID=<domain> AUTH_WEBAUTHN_RP_ORIGIN=https://<domain> \
AUTH_CURSOR_SIGNING_SECRET=... AUTH_BOOTSTRAP_EMAIL=... AUTH_BOOTSTRAP_PASSWORD=... \
  ./target/release/cyberos-auth

# MEMORY (point MEMORY_EMBED_URL at the embed sidecar)
MEMORY_DATABASE_URL=... MEMORY_LISTEN_ADDR=0.0.0.0:7700 MEMORY_EMBED_URL=http://127.0.0.1:5050 \
  ./target/release/cyberos-memory
```

Observed bind ports in the source (each is overridable via the service's `*_LISTEN_ADDR` / `*_BIND`):
4317 (obs OTLP collector), 5050, 7700, 7800, 8085, 8090. Assign one per service in `.env.local` and
map them in Caddy.

## Step 5 - reverse proxy and TLS (Caddy)

Caddy fronts the stack: `CYBEROS_CADDY_SITE` is the public hostname, `CYBEROS_HTTP_BIND` /
`CYBEROS_HTTPS_BIND` are its ports, and it reverse-proxies each service's `*_LISTEN_ADDR`. The
Caddyfile is not in the repo (see gaps) - confirm it on the VPS and make sure the AUTH WebAuthn origin
matches `CYBEROS_CADDY_SITE` exactly.

## Step 6 - smoke test

```bash
curl -fsS https://<domain>/healthz            # via Caddy
curl -fsS http://127.0.0.1:8090/healthz       # AUTH direct (adjust to the real health path)
# AUTH: log in with AUTH_BOOTSTRAP_EMAIL/PASSWORD, confirm a JWT comes back.
# MEMORY: write one audit row, read it back.
# CUO: run one ship-feature-requests iteration end-to-end (it now passes through both the awh and the
#      new caf gate before testing -> done).
```

## Gaps to confirm before go-live (do not skip)

1. The production `docker-compose.yml` and `Caddyfile` that consume `deploy/vps/.env.local` are not in
   the repo (`deploy/vps/` holds only `.env.local` + `data/`). Locate them on the VPS, or commit them
   so the deploy is reproducible. Every "run the service" step above assumes that compose.
2. PROJ is a library (`cyberos_proj`), not a standalone server. Confirm which binary serves its HTTP
   API (likely the mcp-gateway or a proj host bin) and deploy that.
3. CUO's exact console-script name comes from `modules/cuo/pyproject.toml` `[project.scripts]` - read it
   and use that command; confirm how it is supervised (compose service, systemd, or worker).
4. Health-check paths and the real default port per service - read each `main.rs` / `[[bin]]` to confirm
   the route and the `*_LISTEN_ADDR` default before wiring Caddy and the smoke tests.
5. Rotate and remove the in-tree secret files noted in Step 1.

## Rollback

Image tags are pinned in `.env.local` (`CYBEROS_IMAGE_TAG`, `CHAT_IMAGE_TAG`, `CHAT_PINNED_COMMIT`).
Roll back by setting the previous tag and `docker compose up -d` again. Database rollback is per-service
via the down migrations in each `services/<s>/migrations` (take a `pg_dump` of each DB before migrating).
