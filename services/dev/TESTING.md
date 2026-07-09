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

## Known reds (fixes live on `auto/memory-enterprise`)

With the harness fixed, the ingest and interaction tests pass against a current-branch build: `brain_ingest_test` 3/3, `ingest_test` 4/4, `interaction_event_test` 7/7, `interaction_backfill_test` 3/3, `interaction_event_rls_test` 2/2.

The brain-analytics tests still fail: `brain_provenance_test`, `brain_summaries_test`, `brain_tiering_test`, and part of `brain_recall_access_scope_test`. The two representative causes are a column read as `i64` where the SQL type is `INT4`, and `resummarize` binding 2 parameters where the statement needs 3 (the summarize scope-filter placeholder bug, MEM-060). These are service-code fixes already done and gated on `auto/memory-enterprise` (MEM-060 plus the provenance/summary/tiering fixes). Merge that branch for a fully green memory DB suite; this doc and the runner are branch-independent.
