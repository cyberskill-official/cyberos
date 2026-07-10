# Running the Postgres-gated integration tests

Many integration tests under `services/` print `ignored, requires Postgres` (or `requires DATABASE_URL with memory + eval migrations applied`) in a plain `cargo test`. That is deliberate: those tests are marked `#[ignore]` so the default `cargo test` stays hermetic and fast (no database, no Docker). They are not broken and they are not skipped by accident - they need a real Postgres with pgvector and the migrations applied.

This directory has everything to run them.

## One command

```bash
services/dev/test-db.sh                        # every workspace crate
services/dev/test-db.sh -p cyberos-memory      # one crate
services/dev/test-db.sh -p cyberos-auth --test capture_signin_test   # one test binary
```

The script boots the dev Postgres (`docker-compose.yml`, the `pgvector/pgvector:pg16` image), waits for it, ensures the `vector` / `pgcrypto` / `uuid-ossp` extensions, applies every crate's migrations idempotently (auth first for the `cyberos_app` role and tenants, then memory, eval, chat), then runs `cargo test -- --ignored --test-threads=1 --no-fail-fast`.

`--test-threads=1` is required: the tests share one database, and applying DDL from several tests at once races Postgres catalog updates (`tuple concurrently updated`).

Requires Docker running and the Rust toolchain. Containers stay up after the run:

```bash
docker compose -f services/dev/docker-compose.yml down       # stop, keep data
docker compose -f services/dev/docker-compose.yml down -v    # stop and wipe data
```

## Manual steps (what the script automates)

```bash
cd services/dev
docker compose up -d
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
docker exec cyberos-postgres psql -U cyberos -d cyberos -c 'CREATE EXTENSION IF NOT EXISTS vector;'
# apply migrations you need (memory brain tests self-apply 0003-0008; auth/eval tests expect them pre-applied)
cd ..
DATABASE_URL=$DATABASE_URL cargo test -p cyberos-memory -- --ignored --test-threads=1
```

## Which tests self-apply vs expect a migrated DB

- Memory brain tests (`brain_*`, via `tests/brain_common.rs`) and the interaction-event tests apply migrations `0003-0008` plus the eval `access_grant` table themselves, once per process. They only need `DATABASE_URL` and pgvector.
- Auth tests (`admin_*`, `capture_signin_test`) expect the DB already carrying the memory `l1_audit_log` table, the eval governance tables, and the `cyberos_app` role. The runner applies all of that first.

## The harness fix that makes them runnable

`tests/brain_common.rs` and the interaction-event harnesses applied migration files with `sqlx::query(...)`, which uses the prepared-statement protocol and rejects a multi-statement SQL file (`cannot insert multiple commands into a prepared statement`). They now use `sqlx::raw_sql(...)` (the simple protocol), and `brain_common` applies migrations exactly once per process via a `OnceCell`. This is the same fix tracked as MEM-059 on `auto/memory-enterprise`.

## Cross-module tables — why eval + chat migrations always apply

Some suites read tables owned by OTHER crates: auth's `capture_signin_test` needs eval's `monitoring_notice` (the consent gate), and memory's `interaction_backfill_test` needs chat's `chat_channels`/`chat_messages`. So the migration step applies every crate's migrations (auth -> mcp-gateway -> memory -> eval -> chat -> ai-gateway -> email -> proj) even though the eval and chat crate suites are not in `scripts/local_verify.sh`'s Step 3 set. A fresh database without them fails exactly those suites — which is what CI runs.

Corollary: a green run against a long-lived local volume can be a FALSE green (the volume may carry tables a fresh CI database lacks). To verify what CI will see, wipe first:

```bash
docker compose -f services/dev/docker-compose.yml down -v
bash scripts/local_verify.sh
```

## Status

With MEM-059/060 merged to main and the occurred-at tiering fix on this branch, the full memory DB suite and `scripts/local_verify.sh` are green from a fresh volume.
