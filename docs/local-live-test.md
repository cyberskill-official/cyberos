# Local Live Test Runbook

This runbook is the local path for testing SKILL, MEMORY, CUO, AUTH, CHAT, and
PROJ without pretending every module has the same runtime shape.

## What Runs

| Module | Local surface | Docker needed |
|---|---|---|
| SKILL | Rust host tests, VN script parity, `fr-with-tasks` corpus smoke | No |
| MEMORY | Python Layer-1 tests, Rust Layer-2 tests, optional Rust HTTP service | Postgres for live DB tests |
| CUO | Python supervisor/persona/workflow tests | No |
| AUTH | Rust service tests, optional HTTP service on `:7700` | Postgres/Redis for live DB tests |
| CHAT | Control-plane tests, optional Mattermost fork compose on `:8065` | CHAT compose for full container |
| PROJ | Rust library/schema/view tests | Postgres for migrations; no HTTP daemon yet |

PROJ currently has no service binary. Its live surface is migration + Rust
test coverage until the HTTP API lands.

## Tooling Check

```bash
scripts/local-live-test.sh doctor
```

Expected tools:

- `cargo`
- `python`
- `docker` with Docker Desktop/daemon running
- `sqlx` and `psql` are optional for manual inspection; the default migration
  path runs inside Docker.

## Non-Live Verification

This does not require Docker:

```bash
scripts/local-live-test.sh test
```

It runs:

- `modules/memory`: `python -m pytest tests runtime/tests`
- `services`: `cargo test -p cyberos-memory`
- `modules/cuo`: `python -m pytest tests`
- `modules/skill`: `cargo test --workspace`
- `modules/skill`: `python tests/parity/run_parity.py`
- `modules/skill`: `python tests/run_corpus.py fr-with-tasks --no-llm`
- `services`: `cargo test -p cyberos-auth -p cyberos-proj`
- `services/chat`: `bash tests/run_all_tests.sh`

## Shared Infra

Start Postgres/Redis:

```bash
scripts/local-live-test.sh infra-up
```

Fresh volumes create separate module databases:

- `cyberos_auth`
- `cyberos_memory`
- `cyberos_proj`

They are intentionally separate because AUTH, MEMORY, and PROJ each have their
own sqlx migration sequence starting at `0001`. Sharing one DB would collide in
`_sqlx_migrations`.

Apply migrations:

```bash
scripts/local-live-test.sh migrate
```

Run DB-gated tests:

```bash
scripts/local-live-test.sh live-db
```

Stop shared infra:

```bash
scripts/local-live-test.sh infra-down
```

Use `docker compose -f services/dev/docker-compose.yml down -v` only when you
want to delete the local databases.

## Run AUTH and MEMORY Daemons

AUTH:

```bash
cd services
DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_auth \
AUTH_LISTEN_ADDR=127.0.0.1:7700 \
cargo run -p cyberos-auth --bin cyberos-auth
```

MEMORY Layer-2:

```bash
cd services
DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_memory \
MEMORY_LISTEN_ADDR=127.0.0.1:7800 \
cargo run -p cyberos-memory --bin cyberos-memory
```

Smoke:

```bash
curl -s http://127.0.0.1:7700/healthz
curl -s http://127.0.0.1:7800/healthz
```

## CHAT Container

The full CHAT path builds Mattermost from the pinned pre-relicense SHA and is
network-heavy:

```bash
scripts/local-live-test.sh chat-up
```

Open:

```text
http://localhost:8065
```

Stop:

```bash
scripts/local-live-test.sh chat-down
```

The lightweight CHAT control-plane tests are already included in
`scripts/local-live-test.sh test`.

## Current Local Status

Docker is required for the shared infra and CHAT container paths. Check it
before running live tests:

```bash
scripts/local-live-test.sh doctor
scripts/local-live-test.sh infra-up
```

The shared infra compose builds the same Postgres 16 image used by the VPS
profile, with both Apache AGE and pgvector enabled.
