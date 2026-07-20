# deploy/vps - production stack

There are two deploys in this directory.

CURRENT - the P0 product (Google login plus team chat, DMs, and file sharing), live at https://os.cyberskill.world: the `*.p0` files. docker-compose.p0.images.yml runs auth, chat, and Caddy from prebuilt GHCR images on a single origin, backed by Supabase (no Postgres container). Caddyfile.p0 is the single-origin router. Stand up a fresh server with docs/deploy/p0-server-provisioning.md; auto-deploy on push to main is in auto-deploy.md; the Google and tenant setup is in docs/deploy/p0-google-chat-runbook.md.

FUTURE - the full all-modules platform: docker-compose.yml plus Caddyfile (the subdomain model). It brings up Postgres, Redis, and the core Rust services, and is not used by P0. The rest of this README documents that full deploy.

## Files

- docker-compose.yml - the full stack: one Postgres (pgvector, built from services/dev/Dockerfile.postgres), Redis, the four core Rust HTTP services (auth, memory, mcp-gateway, ai-gateway), Caddy in front, and email + native chat behind compose profiles.
- Caddyfile - TLS + reverse proxy, one site per service subdomain (auth./memory./mcp./ai.<base>).
- ../../services/Dockerfile - shared multi-stage build for the workspace binaries; each service passes its own PACKAGE and BIN build args.
- .env.local - the template. Copy to .env on the VPS and replace every placeholder. .env is gitignored.

## Bring-up

```
cd deploy/vps
cp .env.local .env            # then edit .env on the host: real secrets, CYBEROS_BASE_DOMAIN, binds
docker compose up -d --build  # postgres, redis, auth, memory, mcp-gateway, ai-gateway, caddy
# create the per-service databases + run migrations (deploy doc Step 2), then:
docker compose --profile email --profile chat up -d --build   # optional
```

Dependency order is enforced by healthchecks (services wait for Postgres healthy). The deploy doc's Step 4 order still applies for the non-compose pieces (CUO last).

## Secrets

.gitignore already ignores .env.* and *.live (incl. deploy/**/*.live), so .env and the obs *.live tokens are not committed. Verify once: `git ls-files | grep -E '\.live$|/\.env$'` must be empty. The committed .env.local holds only placeholders; never put real secrets in it. Generate signing secrets on the host (`openssl rand -base64 48`) and rotate the obs tokens per the deploy doc Step 1.

## CONFIRM before go-live (the repo cannot settle these)

- ai-gateway: its real listen env var + default port, and whether it needs Redis/DB at boot. The compose assumes AI_GATEWAY_BIND=0.0.0.0:8086 and health /healthz - confirm against the binary.
- memory embeddings: services/embed-sidecar has no Dockerfile. Build it (or use an external embedder) and point MEMORY_EMBED_URL at it, then add the embed service to the compose.
- MCP OAuth: set MCP_DATABASE_URL, create its DB, run the mcp-gateway migrations to turn TASK-MCP-004 on. Without it the gateway runs open (fine for a first smoke, not for production).
- PROJ: cyberos_proj is a library, not a server. Decide which binary serves the proj API and add it.
- CUO: Python orchestration (modules/cuo); supervise via its [project.scripts] entrypoint (compose service or systemd), not wired here.
- chat: native cyberos-chat (Rust) on port 7720; Mattermost is retired. For P0, chat is already wired in the .p0 compose against Supabase - this CONFIRM applies only to the full-platform docker-compose.yml.
- obs stack: already has its own compose at deploy/obs/docker-compose.yml; run it alongside this one.
- health paths: auth /healthz, memory /healthz, mcp-gateway /mcp/healthz, email /v1/email/healthz are confirmed from source; ai-gateway /healthz assumed - verify before trusting the healthchecks.

## Production hardening: DB role grants (do before connecting the app as cyberos_app)

Repo-wide item surfaced by the 2026-06-28 DB-slice audit; it is not specific to the MCP DB slice. Every per-module migration follows the same model: create dedicated NOLOGIN roles (oauth_writer, mcp_elicitation_writer, mcp_task_writer, etc.), `REVOKE UPDATE, DELETE ... FROM cyberos_app`, and grant the column-scoped writes to the dedicated role - relying on auth/0004's `ALTER DEFAULT PRIVILEGES ... TO cyberos_app` for the baseline INSERT/SELECT. Two consequences:

- The dedicated `*_writer` roles are created but never attached to a login on any table, so today they are inert documentation of intent.
- Because UPDATE/DELETE are revoked from `cyberos_app` and not re-granted to it, an app that connects literally as `cyberos_app` would lack UPDATE on the oauth and mcp tables (e.g. consuming an auth code, rotating a refresh token, recording an elicitation response).

In dev and local testing this never bites because the app connects as the table owner (or postgres), for whom GRANT/REVOKE do not restrict - which is why the OAuth flow is already proven locally. Before go-live with a least-privilege login, add one hardening migration that either `GRANT <module>_writer TO cyberos_app` for every writer role, or grants `cyberos_app` the same column-scoped UPDATEs directly, or has each service `SET ROLE` to its writer. Do this repo-wide in one migration; do not diverge individual tables.
