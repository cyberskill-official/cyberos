# cyberos-memory — memory module runtime

Implements the **Layer-2 ingest pipeline** + future search/graph projection on top of the personal-memory Layer-1 chain. Spec source: [`docs/feature-requests/memory/FR-MEMORY-101…111`](../../docs/feature-requests/memory/).

This is the Wave 1 **first-slice scaffold** (2026-05-18). The skeleton compiles, the `/healthz` endpoint works, the chain-anchor verifier has tests. The actual ingest pipeline (`layer2::ingest`) is a `NotYetImplemented` stub — fill it in per FR-MEMORY-101.

## Quick start

```bash
# 1. Boot Postgres + Redis (one terminal)
cd ../dev
docker compose up -d

# 2. Run migrations + start the service (another terminal)
cd ../memory
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
sqlx migrate run                    # applies 0001_layer2.sql + 0002_layer2_cursor.sql
cargo run

# 3. Smoke-test
curl localhost:7800/healthz
# → {"service":"cyberos-memory","version":"0.1.0","postgres":"ok"}
```

## Layout

```
memory/
├── Cargo.toml
├── migrations/
│   ├── 0001_layer2.sql             # l2_memory + l2_entity + l2_edge
│   └── 0002_layer2_cursor.sql      # per-tenant ingest cursor (DEC-073)
├── src/
│   ├── main.rs                     # axum binary entry point
│   ├── lib.rs                      # public crate surface
│   ├── state.rs                    # AppState — PgPool + singletons
│   └── layer2/
│       ├── mod.rs
│       ├── ingest.rs               # orchestrator (stub — FR-MEMORY-101 fill-in)
│       ├── binlog_tail.rs          # Layer-1 audit-log tailer (stub)
│       ├── chain_anchor.rs         # SHA-256 verifier (working + tested)
│       ├── entity_extract.rs       # entity puller (stub)
│       ├── pgvector.rs             # pgvector upsert (stub)
│       ├── age.rs                  # Apache AGE mirror (stub)
│       └── cursor.rs               # per-tenant cursor (working)
└── tests/
    └── chain_anchor_test.rs        # pure-Rust hashing tests
```

## What ships in this slice

| Component | State | Notes |
|---|---|---|
| Cargo workspace + crate boilerplate | ✓ shipped | `services/Cargo.toml`, `services/memory/Cargo.toml` |
| Migrations 0001 + 0002 | ✓ shipped | apply via `sqlx migrate run` against the dev Postgres |
| `chain_anchor::compute` | ✓ shipped + tested | SHA-256(prev_hash ‖ body) per §1 #4 |
| `Cursor` / `PgCursorStore` (CRUD) | ✓ shipped | `load` + `advance` with audit history |
| axum `/healthz` | ✓ shipped | confirms Postgres pool is alive |
| `layer2::ingest::run_batch` | stub | returns `NotYetImplemented` |
| `binlog_tail` poll loop | stub | type-only |
| `entity_extract` | stub | type-only |
| `pgvector` upsert + embedding | stub | empty module |
| `age` graph mirror | stub | empty module |
| `/v1/memory/search` endpoint | not yet | FR-MEMORY-108 |
| Multi-device sync daemon | not yet | FR-MEMORY-103 |
| Tauri desktop app | not yet | FR-MEMORY-104 |

## Pre-existing memory module

The earlier Python implementation in [`../../modules/memory/`](../../modules/memory/) remains the source of truth for **Layer 1** (the personal-memory append-only chain). 233/235 tests pass there. This Rust service is **Layer 2 only** — it READS from Layer 1, never writes to it (DEC-070 invariant).

## Next implementation steps

Per the BACKLOG `§0.6` deploy roadmap, Wave 1 advances by filling these in order:

1. **FR-MEMORY-101** — implement `layer2::ingest::run_batch` reading from the personal-memory audit log via binlog tail, computing + verifying chain anchors, upserting into `l2_memory`, advancing the cursor. Tenant-isolation property test required.
2. **FR-MEMORY-102** — rebuild-from-Layer-1 CI gate. Spot-check + 30-minute reconcile job.
3. **FR-MEMORY-103** — multi-device sync daemon (laptop A ↔ Cloud memory ↔ laptop B).
4. **FR-MEMORY-106** — `sync_class` enforcement (private vs shareable).
5. **FR-MEMORY-108** — search API (`POST /v1/memory/search`).
6. **FR-MEMORY-104** — Tauri 2.x desktop app.
7. **FR-MEMORY-107** — fs-watcher.
8. **FR-MEMORY-110/111** — capture daemon health + pre-ingest PII detection.

## License

Apache-2.0. See repo root `LICENSE`.
