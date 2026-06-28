# Running CyberOS locally and testing the modules

A step-by-step guide to bring the CyberOS backend up on a local machine and run every module's test
suite against it. Every command here was executed and verified on macOS (Docker Desktop) against the
current tree. For the VPS go-live procedure see `cyberos-core-deploy.md`; this document is the local
"test it now" path.

## What runs where

Local infra is two containers from `services/dev/docker-compose.yml`: one Postgres 16 (Apache AGE +
pgvector) on `localhost:5432` and one Redis 7 on `localhost:6379`. Credentials are `cyberos` / `cyberos`
/ `cyberos`. The Rust services and their test suites connect to those over `DATABASE_URL` and
`REDIS_URL`.

The Rust workspace is under `services/`. The DB-backed crates are `auth`, `memory`, `ai-gateway`,
`email`, `proj`, `mcp-gateway` (OAuth + the DB-slice store-of-record, migrations 0013-0017), plus the
obs services (`obs-compliance-view`, `obs-router`) and the shared crates.

## Prerequisites

- Docker Desktop running (confirm with `docker info`; on a cold start it can take a minute to respond).
- Rust 1.88 (`rustup toolchain install 1.88.0`) - the workspace pins this floor.
- The Python env for `modules/cuo` only if you are exercising CUO.

## Step 1 - bring up Postgres and Redis

```bash
cd services
docker compose -f dev/docker-compose.yml up -d --build
```

The `--build` matters: the base `apache/age` image does not ship pgvector, which the memory layer-2
migration needs, so `dev/Dockerfile.postgres` layers pgvector on top. On first boot (a fresh volume)
`dev/postgres-init.sql` enables pgcrypto, uuid-ossp, vector, and age. Confirm both containers are
healthy and the extensions are present:

```bash
docker compose -f dev/docker-compose.yml ps
docker compose -f dev/docker-compose.yml exec -T postgres \
  psql -U cyberos -d cyberos -tAc "SELECT extname FROM pg_extension ORDER BY 1;"
# expect: age, pgcrypto, plpgsql, uuid-ossp, vector
```

## Step 2 - apply the migrations

The integration tests connect to the shared `cyberos` database and expect the schema to exist already
(they do not self-migrate). Apply each module's migrations in dependency order - auth first, because its
`0004_rls_roles.sql` creates the `cyberos_app` role that the memory grants reference.

If you have `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features rustls,postgres`):

```bash
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
# mcp-gateway comes right after auth: its 0013-0017 reference auth's tenants/subjects/cyberos_app/keys.
for m in auth mcp-gateway memory ai-gateway email proj; do
  sqlx migrate run --source $m/migrations
done
```

Without sqlx-cli, apply the files directly through the container in order:

```bash
for crate in auth mcp-gateway memory ai-gateway email proj; do
  for f in $(ls $crate/migrations/*.sql 2>/dev/null | sort); do
    docker compose -f dev/docker-compose.yml exec -T postgres \
      psql -U cyberos -d cyberos -v ON_ERROR_STOP=1 -q -f - < "$f" || echo "FAILED: $f"
  done
done
```

### MCP store-of-record (the DB slice): two local modes

The mcp-gateway runs locally in two modes:

- Quick smoke (in-memory): leave `MCP_REQUIRE_AUTH` unset. tools/call, elicitations, and tasks use the
  in-memory stores; no token or KMS key needed. This is the `scripts/mcp_demo.sh` path (Step 6, Tier 3),
  enough to prove the gateway, registration, and the destructive-confirm gate.
- Store-of-record (the DB slice): set `MCP_DATABASE_URL`, `MCP_REQUIRE_AUTH=1`, and `MCP_KMS_KEY` (base64
  of 32 bytes - generate with `openssl rand -base64 32`). Elicitations and tasks then persist to
  `mcp_elicitations` / `mcp_tasks` (migrations 0016/0017, applied in Step 2), payloads are sealed at rest,
  and a held confirmation survives a restart. This path is caller-scoped, so it needs a real OAuth access
  token: run the FR-MCP-004 flow (register a client, authorize, exchange for a token) and send it as
  `Authorization: Bearer <token>`. Without `MCP_KMS_KEY` the gateway refuses to start in this mode, by
  design - a held confirmation would otherwise be un-respondable.

## Step 3 - run the test suites

Set both connection strings, then run each crate. Two flags matter:

- `--test-threads=1` - the integration tests share one database, so they are not parallel-safe. Run them
  serially (the auth and ai-gateway gates already specify this).
- `--include-ignored` - the DB-backed tests are marked `#[ignore]` so they are skipped when no database
  is configured; include them once Postgres is up.

```bash
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
export REDIS_URL=redis://127.0.0.1:6379

for crate in cyberos-auth cyberos-memory cyberos-email cyberos-proj \
             cyberos-obs-compliance-view cyberos-obs-router cyberos-mcp-gateway; do
  echo "==== $crate ===="
  cargo test -p $crate -- --include-ignored --test-threads=1
done

# ai-gateway is the long one (Redis property tests); run it on its own.
cargo test -p cyberos-ai-gateway -- --include-ignored --test-threads=1
```

The shared crates (`cyberos-obs-sdk`, `cyberos-audit-chain`, `cyberos-cli-exit`, `cyberos-types`) are
pure and need no database: `cargo test -p cyberos-obs-sdk` and so on.

### Known environment-sensitive results

- `auth::create_subject_p95_latency_under_200ms` asserts a p95 under 200 ms. Docker Desktop on macOS is
  slower than CI, so it can trip locally. Not a logic failure - relax or skip that threshold for local
  runs (`--skip create_subject_p95`).
- `ai-gateway::cost_hold_expiry::*` spawn the FR-AI-003 Writer (`python3 -m cyberos.writer put`), so the
  memory package must be importable when you run the ai-gateway suite. Either `pip install -e
  modules/memory` once, or set `PYTHONPATH=$PWD/../modules/memory` (from `services/`) before `cargo test`.
  Without it the audit emit fails, the expiry rolls back, and the two `tick_skips_*` tests fail. This was
  a real cross-module contract gap (the `cyberos.writer` module was missing); it is fixed - see
  `docs/KNOWN-ISSUES.md` issue 1.
- If a run leaves the DB dirty and a later run fails, reset with a clean volume: `docker compose
  -f dev/docker-compose.yml down -v && docker compose -f dev/docker-compose.yml up -d --build`, then
  re-apply migrations (step 2).

## Step 4 - run a service and hit it

Example: the AI gateway HTTP server, with the in-repo echo backend (no provider key needed).

```bash
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
export AI_GATEWAY_BIND=0.0.0.0:8080
export AI_GATEWAY_CONFIG_DIR=services/ai-gateway/config/tenants   # confirm the tenant config dir
cargo run -p cyberos-ai-gateway --bin cyberos-gateway

# in another shell:
curl -fsS http://127.0.0.1:8080/healthz
curl -fsS -X POST http://127.0.0.1:8080/v1/chat \
  -H 'content-type: application/json' -H 'x-tenant-id: org:cyberskill' \
  -d '{"alias":"chat.smart","messages":[{"role":"user","content":"hello"}]}'
# the response carries a `traceparent` header (FR-OBS-005); the logs are JSON with trace_id per line.
```

AUTH and MEMORY bind `AUTH_LISTEN_ADDR` / `MEMORY_LISTEN_ADDR` and read the env surface listed in
`cyberos-core-deploy.md` Step 1. MEMORY also needs an embeddings endpoint (`MEMORY_EMBED_URL`, the
`embed-sidecar`).

## Step 5 - the telemetry stack (optional)

The RED metrics, traces, and the tenant-scoped Grafana proxy live in `deploy/obs/`. Bring up Grafana +
Loki + Prometheus + Tempo + the obs-proxy with `cd deploy/obs && docker compose up --build`, then set
`OBS_OTLP_ENDPOINT` on each service so its metrics and traces export. See `deploy/obs/README.md`. This
is heavier (it pulls four backend images) and is only needed for the end-to-end correlation path.

## Step 6 - end-to-end smoke (the demoable path)

Steps 1-5 prove each module in isolation. This step runs the live P0 path so you can see the stack work
as one system. Do it in tiers; each tier stands alone.

Tier 1 - chat path through the gateway (no model needed). With infra up (Step 1) and migrations applied
(Step 2), run the gateway with the in-repo echo backend and hit it:

```bash
cd services
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
AI_GATEWAY_BIND=127.0.0.1:8080 AI_GATEWAY_CONFIG_DIR=ai-gateway/config/tenants \
  cargo run -p cyberos-ai-gateway --bin cyberos-gateway
# another shell:
curl -fsS http://127.0.0.1:8080/healthz
curl -fsS -X POST http://127.0.0.1:8080/v1/chat -H 'content-type: application/json' \
  -H 'x-tenant-id: org:cyberskill' \
  -d '{"alias":"chat.smart","messages":[{"role":"user","content":"hello"}]}'
```

Tier 2 - real inference through the stack (the demo). Start LM Studio (or Ollama) with a model loaded,
then point the gateway at it. The tenant config `ai-gateway/config/tenants/org-cyberskill.yaml` maps
`chat.smart` to the local model - edit the model id there if yours differs. The audit path (FR-AI-003)
shells out to the memory Writer, so make the memory package importable first:

```bash
cd services
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
export PYTHONPATH=$PWD/../modules/memory          # or: pip install -e ../modules/memory
AI_GATEWAY_BIND=127.0.0.1:8080 AI_GATEWAY_CONFIG_DIR=ai-gateway/config/tenants \
  LMSTUDIO_ENDPOINT=http://127.0.0.1:1234 \
  cargo run -p cyberos-ai-gateway --bin cyberos-gateway
# the same /v1/chat curl now returns a real completion from your local model.
```

Tier 3 - MCP tool federation (independent of the gateway above). One command starts the mcp-gateway,
the reference module, and the obs triage module, all self-registering:

```bash
bash scripts/mcp_demo.sh
# another shell - list and call tools through the gateway:
bash scripts/mcp_call.sh cyberos.demo.echo '{"message":"hi"}'
bash scripts/mcp_call.sh cyberos.obs.execute_triage \
  '{"alert":{"name":"HighErrorRate","severity":"sev2","summary":"5xx above 2%"}}'
```

The triage tool name is `cyberos.obs.execute_triage` (the SEP-986 form); a non-conforming registration
is now rejected at the gateway, so this is also a live check of FR-MCP-003 enforcement.

Tier 4 - the desktop app (optional, Tauri build on your Mac). Build and run `apps/desktop` per its
README; it drives the running gateway's chat-trigger path and, next iteration, the `tools/list` picker.

Tier 5 - telemetry correlation (optional, heavier). Bring up `deploy/obs/` (Step 5) and set
`OBS_OTLP_ENDPOINT` on the gateway; the `traceparent` from the `/v1/chat` response should resolve to a
trace in Grafana.

What success looks like: infra healthy with all five extensions; every crate suite GREEN under
`--include-ignored`; `/v1/chat` returns an echo (Tier 1) then a real model completion (Tier 2); the two
MCP tool calls return their results (Tier 3). At that point the core is proven locally. MCP is now
feature-complete (all 8 FRs + the DB-slice store-of-record, audited 2026-06-28); the next step is the
VPS deploy.

## Teardown

```bash
cd services
docker compose -f dev/docker-compose.yml down       # stop, keep data
docker compose -f dev/docker-compose.yml down -v     # stop and wipe the volumes (clean slate)
```

## Going to the VPS

The production path is the same migrations and binaries against a per-service database, fronted by Caddy
for TLS. The production `docker-compose.yml` + `Caddyfile` now live in `deploy/vps/` (see its README);
follow `cyberos-core-deploy.md` for the full runbook. Before go-live: copy `deploy/vps/.env.local` to
`.env` and fill real secrets, rotate any in-tree live-secret files and keep them out of git, and apply
the one-time repo-wide DB-role-grant hardening noted in `deploy/vps/README.md` so a least-privilege
`cyberos_app` login can write the oauth + mcp tables.
```
