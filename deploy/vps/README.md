# deploy/vps - production stack

This closes gap #1 in docs/deploy/cyberos-core-deploy.md: the production compose and Caddyfile that
consume this directory's env file now live in the repo, so the deploy is reproducible. Read the deploy
doc for the full runbook; this README covers only what is new here.

## Files

- docker-compose.yml - the stack: one Postgres (AGE + pgvector, built from services/dev/Dockerfile.postgres),
  Redis, the four core Rust HTTP services (auth, memory, mcp-gateway, ai-gateway), Caddy in front, and
  email + chat behind compose profiles.
- Caddyfile - TLS + reverse proxy, one site per service subdomain (auth./memory./mcp./ai.<base>).
- ../../services/Dockerfile - shared multi-stage build for the workspace binaries; each service passes
  its own PACKAGE and BIN build args.
- .env.local - the template. Copy to .env on the VPS and replace every placeholder. .env is gitignored.

## Bring-up

```
cd deploy/vps
cp .env.local .env            # then edit .env on the host: real secrets, CYBEROS_BASE_DOMAIN, binds
docker compose up -d --build  # postgres, redis, auth, memory, mcp-gateway, ai-gateway, caddy
# create the per-service databases + run migrations (deploy doc Step 2), then:
docker compose --profile email --profile chat up -d --build   # optional
```

Dependency order is enforced by healthchecks (services wait for Postgres healthy). The deploy doc's
Step 4 order still applies for the non-compose pieces (CUO last).

## Secrets

.gitignore already ignores .env.* and *.live (incl. deploy/**/*.live), so .env and the obs *.live tokens
are not committed. Verify once: `git ls-files | grep -E '\.live$|/\.env$'` must be empty. The committed
.env.local holds only placeholders; never put real secrets in it. Generate signing secrets on the host
(`openssl rand -base64 48`) and rotate the obs tokens per the deploy doc Step 1.

## CONFIRM before go-live (the repo cannot settle these)

- ai-gateway: its real listen env var + default port, and whether it needs Redis/DB at boot. The
  compose assumes AI_GATEWAY_BIND=0.0.0.0:8086 and health /healthz - confirm against the binary.
- memory embeddings: services/embed-sidecar has no Dockerfile. Build it (or use an external embedder)
  and point MEMORY_EMBED_URL at it, then add the embed service to the compose.
- MCP OAuth: set MCP_DATABASE_URL, create its DB, run the mcp-gateway migrations to turn FR-MCP-004 on.
  Without it the gateway runs open (fine for a first smoke, not for production).
- PROJ: cyberos_proj is a library, not a server. Decide which binary serves the proj API and add it.
- CUO: Python orchestration (modules/cuo); supervise via its [project.scripts] entrypoint (compose
  service or systemd), not wired here.
- chat: confirm the container's listen port for the Caddy route, and the Mattermost DB wiring.
- obs stack: already has its own compose at deploy/obs/docker-compose.yml; run it alongside this one.
- health paths: auth /healthz, memory /healthz, mcp-gateway /mcp/healthz, email /v1/email/healthz are
  confirmed from source; ai-gateway /healthz assumed - verify before trusting the healthchecks.

## Production hardening: DB role grants (do before connecting the app as cyberos_app)

Repo-wide item surfaced by the 2026-06-28 DB-slice audit; it is not specific to the MCP DB slice. Every
per-module migration follows the same model: create dedicated NOLOGIN roles (oauth_writer,
mcp_elicitation_writer, mcp_task_writer, etc.), `REVOKE UPDATE, DELETE ... FROM cyberos_app`, and grant the
column-scoped writes to the dedicated role - relying on auth/0004's `ALTER DEFAULT PRIVILEGES ... TO
cyberos_app` for the baseline INSERT/SELECT. Two consequences:

- The dedicated `*_writer` roles are created but never attached to a login on any table, so today they are
  inert documentation of intent.
- Because UPDATE/DELETE are revoked from `cyberos_app` and not re-granted to it, an app that connects
  literally as `cyberos_app` would lack UPDATE on the oauth and mcp tables (e.g. consuming an auth code,
  rotating a refresh token, recording an elicitation response).

In dev and local testing this never bites because the app connects as the table owner (or postgres), for
whom GRANT/REVOKE do not restrict - which is why the OAuth flow is already proven locally. Before go-live
with a least-privilege login, add one hardening migration that either `GRANT <module>_writer TO cyberos_app`
for every writer role, or grants `cyberos_app` the same column-scoped UPDATEs directly, or has each service
`SET ROLE` to its writer. Do this repo-wide in one migration; do not diverge individual tables.
