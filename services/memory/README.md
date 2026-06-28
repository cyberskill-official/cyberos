# cyberos-memory — memory module runtime

Implements the **Layer-2 ingest pipeline**, rebuild/reconcile gate, and search/graph projection on top of the personal-memory Layer-1 chain. Spec source: [`docs/feature-requests/memory/FR-MEMORY-101…111`](../../docs/feature-requests/memory/).

Wave 1 is now shipped across the Rust Layer-2 service, the Tauri desktop scaffold, and the pre-existing Python Layer-1 CLI. The Rust service reads Layer 1 only, ingests append-only audit rows into `l2_memory`/`l2_entity` (with graph edges in the relational `l2_edge` table), exposes `/v1/memory/search`, and ships `cyberos-memory-admin rebuild|reconcile` for the FR-MEMORY-102 gate.

## Quick start

```bash
# 1. Boot Postgres + Redis (one terminal)
cd ../dev
docker compose up -d

# 2. Run migrations + start the service (another terminal)
cd ../memory
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
sqlx migrate run                    # applies Layer-2 + cursor + Layer-1 audit mirror tables
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
│   ├── 0002_layer2_cursor.sql      # per-tenant ingest cursor (DEC-073)
│   └── 0003_layer1_audit_log.sql   # Layer-1 audit mirror for ingest/rebuild
├── src/
│   ├── bin/admin.rs                # rebuild + reconcile CLI
│   ├── main.rs                     # axum binary entry point
│   ├── lib.rs                      # public crate surface
│   ├── embeddings.rs               # optional bge-m3 sidecar client
│   ├── rebuild.rs                  # FR-MEMORY-102 full rebuild + spot-check
│   ├── search.rs                   # FR-MEMORY-108 hybrid search endpoint
│   ├── state.rs                    # AppState — PgPool + singletons
│   └── layer2/
│       ├── mod.rs
│       ├── ingest.rs               # FR-MEMORY-101 batch orchestrator
│       ├── binlog_tail.rs          # Layer-1 audit-log tailer + append helper
│       ├── chain_anchor.rs         # SHA-256 verifier (working + tested)
│       ├── entity_extract.rs       # @handle / #slug / [[link]] puller
│       ├── pgvector.rs             # l2_memory + l2_entity upsert helpers
│       └── cursor.rs               # per-tenant cursor (working)
├── tests/
    ├── chain_anchor_test.rs        # pure-Rust hashing tests
    └── ingest_test.rs              # ingest idempotency + tenant isolation
└── desktop/                        # FR-MEMORY-104 Tauri 2.x scaffold
```

## What ships

| Component | State | Notes |
|---|---|---|
| Cargo workspace + crate boilerplate | ✓ shipped | `services/Cargo.toml`, `services/memory/Cargo.toml` |
| Migrations 0001–0003 | ✓ shipped | apply via `sqlx migrate run` against the dev Postgres |
| `chain_anchor::compute` | ✓ shipped + tested | SHA-256(prev_hash ‖ body) per §1 #4 |
| `Cursor` / `PgCursorStore` (CRUD) | ✓ shipped | `load` + `advance` with audit history |
| axum `/healthz` + `/metrics` | ✓ shipped | confirms Postgres pool and exports daemon counters |
| `layer2::ingest::run_batch` | ✓ shipped | tails Layer-1 mirror, verifies chain anchors, upserts, advances cursor |
| `binlog_tail` poll loop | ✓ shipped | deterministic batch polling + append helper for sync imports |
| `entity_extract` | ✓ shipped | regex extraction for handles, tags, and wikilinks |
| `pgvector` upsert + embedding | ✓ shipped | degrades gracefully when embedding sidecar is unavailable |
| `l2_edge` graph edges | relational | entity->entity edges in Postgres; traversed via recursive CTEs (Phase-3 link extraction) |
| `/v1/memory/search` endpoint | ✓ shipped | lexical + vector recall with RRF-style fusion |
| `cyberos-memory-admin rebuild|reconcile` | ✓ shipped | FR-MEMORY-102 rebuild gate + sample spot-check |
| Multi-device sync + sync_class gate | ✓ shipped | Python Layer-1 CLI and sync helpers cover FR-MEMORY-103/106 |
| Capture daemon, hooks, PII guard | ✓ shipped | Python Layer-1 capture/watch/hook/PII surfaces cover FR-MEMORY-107/109/110/111 |
| Tauri desktop app | ✓ shipped | `desktop/` Tauri 2.x scaffold with dashboard/search/quick capture |

## Pre-existing memory module

The Python implementation in [`../../modules/memory/`](../../modules/memory/) remains the source of truth for **Layer 1** (the personal-memory append-only chain), including protocol writes, watcher registration, sync-class filtering, hook capture, and PII scanning. This Rust service is **Layer 2 only** — it READS from Layer 1, never writes to it (DEC-070 invariant).

## Verification

```bash
cd ../memory
cargo test -p cyberos-memory

cd ../../modules/memory
python -m pytest
```

## License

Apache-2.0. See repo root `LICENSE`.
