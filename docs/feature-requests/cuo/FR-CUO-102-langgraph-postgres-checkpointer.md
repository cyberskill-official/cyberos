---
id: FR-CUO-102
title: "CUO Postgres checkpointer for LangGraph state — persists supervisor graph state per run with EU AI Act Art. 12 logging"
module: CUO
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-CUO-101, FR-CUO-103, FR-MEMORY-111]
depends_on: [FR-CUO-101]
blocks: [FR-CUO-103]

source_pages:
  - website/docs/modules/cuo.html#checkpointer
  - https://eur-lex.europa.eu/eli/reg/2024/1689  # EU AI Act Art. 12

source_decisions:
  - DEC-2320 2026-05-17 — LangGraph state checkpointed to Postgres per supervisor node transition; rows immutable per EU AI Act Art. 12 logging requirement
  - DEC-2321 2026-05-17 — Closed enum `checkpoint_kind` = {node_entered, node_completed, edge_traversed, run_started, run_completed, run_failed}; cardinality 6
  - DEC-2322 2026-05-17 — Checkpointer interface implements LangGraph's `BaseCheckpointer`; persists full state JSON per checkpoint
  - DEC-2323 2026-05-17 — Append-only; 7-year retention per Art. 12; archive partition monthly
  - DEC-2324 2026-05-17 — memory audit kinds: cuo.checkpoint_written, cuo.checkpoint_replayed, cuo.checkpoint_archive_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/cuo/
  new_files:
    - services/cuo/migrations/0002_langgraph_checkpoints.sql
    - services/cuo/src/checkpointer/mod.rs
    - services/cuo/src/checkpointer/postgres_writer.rs
    - services/cuo/src/checkpointer/state_serializer.rs
    - services/cuo/src/audit/checkpoint_events.rs
    - services/cuo/tests/checkpoint_kind_enum_cardinality_test.rs
    - services/cuo/tests/checkpoint_immutable_test.rs
    - services/cuo/tests/checkpoint_serialization_roundtrip_test.rs
    - services/cuo/tests/checkpoint_audit_emission_test.rs

  modified_files:
    - services/cuo/src/supervisor.rs

  allowed_tools:
    - file_read: services/cuo/**
    - file_write: services/cuo/{src,tests,migrations}/**
    - bash: cd services/cuo && cargo test checkpoint

  disallowed_tools:
    - mutate prior checkpoint (per DEC-2323)
    - skip persistence (per DEC-2320)

effort_hours: 5
sub_tasks:
  - "0.3h: 0002_langgraph_checkpoints.sql"
  - "0.3h: checkpointer/mod.rs"
  - "0.5h: postgres_writer.rs"
  - "0.5h: state_serializer.rs"
  - "0.3h: audit/checkpoint_events.rs"
  - "1.8h: tests — 4 test files"
  - "0.8h: supervisor integration + docs"
  - "0.5h: archive partition cron"

risk_if_skipped: "Without checkpointer, supervisor state lost on crash → restart from scratch. Without DEC-2320 EU AI Act compliance, sale to EU customers blocked. Without DEC-2323 retention, audit gap."
---

## §1 — Description (BCP-14 normative)

The CUO service **MUST** ship Postgres checkpointer at `services/cuo/src/checkpointer/` implementing LangGraph BaseCheckpointer + immutable persistence + EU AI Act Art. 12 logging, 3 memory audit kinds.

1. **MUST** validate `checkpoint_kind` against closed enum per DEC-2321.

2. **MUST** persist at `postgres_writer.rs::write(run_id, kind, state)` per DEC-2320:
   - Each node entered + completed; each edge traversed
   - Full state JSON serialized via `state_serializer.rs`
   - Trace_id from FR-CUO-101 supervisor context

3. **MUST** define table at migration `0002`:
   ```sql
   CREATE TABLE cuo_langgraph_checkpoints (
     checkpoint_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     run_id UUID NOT NULL,
     parent_checkpoint_id UUID REFERENCES cuo_langgraph_checkpoints(checkpoint_id),
     node_name TEXT,
     edge_from TEXT,
     edge_to TEXT,
     kind TEXT NOT NULL
       CHECK (kind IN ('node_entered','node_completed','edge_traversed','run_started','run_completed','run_failed')),
     state_json JSONB NOT NULL,
     trace_id CHAR(32) NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   ) PARTITION BY RANGE (created_at);
   CREATE INDEX checkpoints_run_idx ON cuo_langgraph_checkpoints(tenant_id, run_id, created_at);
   ALTER TABLE cuo_langgraph_checkpoints ENABLE ROW LEVEL SECURITY;
   CREATE POLICY checkpoints_rls ON cuo_langgraph_checkpoints
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON cuo_langgraph_checkpoints FROM cyberos_app;
   ```

4. **MUST** archive monthly partitions per DEC-2323 — older than 7 years can be safely removed via DROP PARTITION.

5. **MUST** expose endpoints:
   ```text
   GET /v1/cuo/runs/{run_id}/checkpoints   (FR-CUO-103 replay reads these)
   ```

6. **MUST** emit 3 memory audit kinds per DEC-2324. PII per FR-MEMORY-111: state_json hashed; ids ok.

7. **MUST** thread trace_id from supervisor → writer → audit.

8. **MUST NOT** mutate prior checkpoint per DEC-2323 (REVOKE UPDATE/DELETE).

9. **MUST NOT** skip persistence per DEC-2320 (Art. 12 compliance).

---

## §2 — Why this design

**Why Postgres (DEC-2320)?** Already in stack; transactional; supports JSONB for state.

**Why per-node checkpoint (DEC-2320)?** Resume from last successful node on crash; replay to investigate failures.

**Why partition (DEC-2323)?** 7-year retention → millions of rows; partition by month enables fast drop.

---

## §3 — API contract

Sample checkpoint:
```json
{
  "checkpoint_id": "uuid",
  "run_id": "uuid",
  "node_name": "router",
  "kind": "node_completed",
  "state_json": {"selected_skill": "calendar.list_events", ...},
  "trace_id": "abcdef..."
}
```

---

## §4 — Acceptance criteria
1. **checkpoint_kind enum cardinality 6**. 2. **LangGraph BaseCheckpointer interface implemented**. 3. **Per-node + per-edge persistence**. 4. **State JSON serialized**. 5. **trace_id captured**. 6. **3 memory audit kinds emitted**. 7. **PII scrubbed (state_json SHA256)**. 8. **RLS denies cross-tenant**. 9. **Trace_id preserved**. 10. **Append-only via REVOKE UPDATE/DELETE**. 11. **Monthly partitioning**. 12. **7-year retention**. 13. **Archive cron via DROP PARTITION**. 14. **Replay roundtrip OK**. 15. **Performance < 5ms per checkpoint**. 16. **Run-scoped query indexed**. 17. **Parent_checkpoint_id forms DAG**. 18. **EU AI Act Art. 12 documented**. 19. **Concurrent checkpoint OK**. 20. **State size capped 5MB per checkpoint**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn checkpoint_immutable() {
    let ctx = TestContext::with_checkpoint().await;
    let r = ctx.try_update_checkpoint(ctx.cp_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn serialization_roundtrip() {
    let state = json!({"key": "value", "nested": {"a": 1}});
    let serialized = state_serializer::serialize(&state);
    let restored = state_serializer::deserialize(&serialized);
    assert_eq!(state, restored);
}

#[tokio::test]
async fn per_node_checkpoint_count() {
    let ctx = TestContext::with_supervisor_run_5_nodes().await;
    let cps = ctx.fetch_checkpoints(ctx.run_id).await;
    assert!(cps.len() >= 5 * 2);  // entered + completed per node
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-CUO-101.
**Downstream:** FR-CUO-103 (replay uses checkpoints).
**Cross-module:** FR-MEMORY-111 (PII), FR-MCP-007 (archive cron).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| State > 5MB | validate | reject; sev-2 | reduce state |
| Serialization fail | catch | sev-1; run halts | bug fix |
| Cross-tenant query | RLS | 0 rows | inherent |
| Mutation attempt | REVOKE | DB error | inherent |
| Partition table count grows | monthly partitions | OK | inherent |
| Archive cron fail | sev-2 | data retained longer | retry |
| Concurrent checkpoint | UNIQUE on (run, time) NOT needed (append) | inherent | inherent |
| State JSON invalid | validate | reject; sev-1 | bug fix |
| Trace_id missing | sev-2 | use NIL_UUID | bug fix |
| Decimal precision N/A | inherent | inherent | inherent |

## §11 — Implementation notes
- §11.1 BaseCheckpointer trait per LangGraph spec: `aput()`, `aget_tuple()`, `alist()`.
- §11.2 Partition strategy: monthly RANGE on created_at; drop > 84 months old (7 years).
- §11.3 memory audit body: run_id, node_name, kind; state_json SHA256.
- §11.4 Archive cron via FR-MCP-007 monthly; DROP PARTITION for oldest.
- §11.5 EU AI Act Art. 12: high-risk AI systems must keep automatic logs; this satisfies it.

---

*End of FR-CUO-102 spec.*
