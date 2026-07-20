---
id: TASK-CUO-103
title: "CUO Phase 2 trace rows include prompt + model + temperature + seed for deterministic replay"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: CUO
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CUO-102, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CUO-101, TASK-CUO-102]
blocks: []

source_pages:
  - website/docs/modules/cuo.html#trace-replay

source_decisions:
  - DEC-2330 2026-05-17 — Each AI inference call within supervisor logged with prompt + model + temperature + seed; enables byte-identical replay (assuming model determinism)
  - DEC-2331 2026-05-17 — Closed enum `trace_call_kind` = {router_decision, tool_call, response_generation, validation_check}; cardinality 4
  - DEC-2332 2026-05-17 — Per-call row IMMUTABLE; correction = new row with `correction_of` link
  - "DEC-2333 2026-05-17 — Replay test: re-run with same seed + prompt; expect identical output (caveat: vendor model may drift; mark drift as sev-2)"
  - DEC-2334 2026-05-17 — memory audit kinds: cuo.trace_row_added, cuo.trace_replay_match, cuo.trace_replay_drift, cuo.trace_replay_failed

language: rust 1.81
service: cyberos/services/cuo/
new_files:
  - services/cuo/migrations/0003_trace_rows.sql
  - services/cuo/src/trace/mod.rs
  - services/cuo/src/trace/writer.rs
  - services/cuo/src/trace/replay.rs
  - services/cuo/src/handlers/trace_routes.rs
  - services/cuo/src/audit/trace_events.rs
  - services/cuo/tests/trace_call_kind_enum_cardinality_test.rs
  - services/cuo/tests/trace_replay_byte_identical_test.rs
  - services/cuo/tests/trace_replay_drift_detection_test.rs
  - services/cuo/tests/trace_immutable_test.rs
  - services/cuo/tests/trace_audit_emission_test.rs

modified_files:
  - services/cuo/src/supervisor.rs

allowed_tools:
  - file_read: services/{cuo,ai}/**
  - file_write: services/cuo/{src,tests,migrations}/**
  - bash: cd services/cuo && cargo test trace

disallowed_tools:
  - mutate prior trace (per DEC-2332)
  - skip seed capture (per DEC-2330)

effort_hours: 4
subtasks:
  - "0.3h: 0003_trace_rows.sql"
  - "0.3h: trace/mod.rs"
  - "0.4h: writer.rs"
  - "0.5h: replay.rs"
  - "0.4h: handlers/trace_routes.rs"
  - "0.3h: audit/trace_events.rs"
  - "1.5h: tests — 5 test files"
  - "0.3h: docs"

risk_if_skipped: "Without replay capability, AI bugs unreproducible. Without DEC-2330 seed capture, even with prompt log replay non-deterministic. Without DEC-2333 drift detection, vendor model changes go unnoticed."
---

## §1 — Description (BCP-14 normative)

The CUO service **MUST** ship trace replay rows at `services/cuo/src/trace/` capturing prompt+model+temp+seed per AI call + replay test, 4 memory audit kinds.

1. **MUST** validate `trace_call_kind` against closed enum per DEC-2331.

2. **MUST** capture per call at `writer.rs::write(call)` per DEC-2330:
- prompt (full text)
- model (e.g. "claude-sonnet-4-6")
- temperature
- seed (or null if vendor doesn't support)
- response (full output)

3. **MUST** replay at `replay.rs::replay(trace_id)` per DEC-2333:
- Recall prompt + model + temp + seed
- Re-invoke AI provider with same params
- Compare output bytes
- Match → audit "match"; differ → audit "drift" sev-2

4. **MUST** define table at migration `0003`:
   ```sql
   CREATE TABLE cuo_trace_rows (
     trace_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     run_id UUID NOT NULL,
     checkpoint_id UUID,  -- TASK-CUO-102 ref
     kind TEXT NOT NULL
       CHECK (kind IN ('router_decision','tool_call','response_generation','validation_check')),
     prompt_text TEXT NOT NULL,
     model TEXT NOT NULL,
     temperature NUMERIC(5,4) NOT NULL,
     seed BIGINT,
     response_text TEXT NOT NULL,
     correction_of UUID REFERENCES cuo_trace_rows(trace_id),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   ) PARTITION BY RANGE (created_at);
   CREATE INDEX trace_run_idx ON cuo_trace_rows(tenant_id, run_id, created_at);
   ALTER TABLE cuo_trace_rows ENABLE ROW LEVEL SECURITY;
   CREATE POLICY trace_rls ON cuo_trace_rows
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON cuo_trace_rows FROM cyberos_app;
   ```

5. **MUST** expose endpoints:
   ```text
   POST /v1/cuo/trace/{id}/replay     (CDO triggers replay test)
   GET  /v1/cuo/runs/{run_id}/trace   (list trace rows)
   ```

6. **MUST** emit 4 memory audit kinds per DEC-2334. PII per TASK-MEMORY-111: prompt + response SHA-256 hashed.

7. **MUST** thread trace_id from supervisor → AI call → writer → audit.

8. **MUST NOT** mutate prior trace per DEC-2332 (REVOKE UPDATE/DELETE).

9. **MUST NOT** skip seed capture per DEC-2330 (even if null, document why).

---

## §2 — Why this design

**Why prompt + model + temp + seed (DEC-2330)?** Reproducibility — these 4 fields are the AI call's inputs; with seed, output deterministic.

**Why immutable (DEC-2332)?** Audit lineage; without immutability, replay becomes circular.

**Why drift detection (DEC-2333)?** Vendor models version-drift; without drift alert, replay quietly produces different output.

---

## §3 — API contract

Sample trace row:
```json
{
  "trace_id": "uuid",
  "run_id": "uuid",
  "kind": "router_decision",
  "prompt_text": "Given user request: ... Choose tool from: ...",
  "model": "claude-sonnet-4-6",
  "temperature": 0.0,
  "seed": 42,
  "response_text": "calendar.list_events"
}
```

---

## §4 — Acceptance criteria
1. **trace_call_kind enum cardinality 4**. 2. **prompt + model + temp + seed captured**. 3. **Immutable rows**. 4. **Replay test exists**. 5. **Match → audit emitted**. 6. **Drift → sev-2 audit**. 7. **4 memory audit kinds emitted**. 8. **PII scrubbed (prompt + response SHA256)**. 9. **RLS denies cross-tenant**. 10. **Trace_id preserved**. 11. **Append-only via REVOKE**. 12. **Monthly partitioning**. 13. **Replay async (LLM call expensive)**. 14. **rust_decimal for temperature**. 15. **bigint for seed**. 16. **NULL seed allowed (vendor-dependent)**. 17. **Correction_of for re-recorded traces**. 18. **CDO-only replay trigger**. 19. **Run-scoped query indexed**. 20. **Partition retention same as TASK-CUO-102 (7 years)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn replay_byte_identical_with_seed() {
    let ctx = TestContext::with_trace_seed_42().await;
    let r = ctx.replay(ctx.trace_id).await;
    assert_eq!(r.status, "match");
}

#[tokio::test]
async fn drift_detected_when_response_differs() {
    let ctx = TestContext::with_trace().await;
    ctx.mock_ai_returns_different_response().await;
    let r = ctx.replay(ctx.trace_id).await;
    assert_eq!(r.status, "drift");
    let audits = ctx.fetch_memory_audits("cuo.trace_replay_drift").await;
    assert!(!audits.is_empty());
}

#[tokio::test]
async fn immutable_no_update() {
    let ctx = TestContext::with_trace().await;
    let r = ctx.try_update_trace(ctx.trace_id).await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-CUO-102. **Cross-module:** TASK-AI-003 (LLM calls captured), TASK-AUTH-101 (CDO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Mutation attempt | REVOKE | DB error | inherent |
| Replay drift | comparison | sev-2 | investigate vendor |
| Vendor model deprecated | replay fail | sev-2 | model upgrade plan |
| Cross-tenant query | RLS | 0 rows | inherent |
| NULL seed (vendor) | OK | replay marked non-deterministic | inherent |
| Prompt > 100k chars | validate | reject; sev-2 | reduce |
| Replay rate limit | inherent | retry | inherent |
| Concurrent trace | append | inherent | inherent |
| Decimal precision | rust_decimal | inherent | inherent |
| Partition fill | monthly | inherent | inherent |

## §11 — Implementation notes
- §11.1 Seed defaults to 0 if vendor supports; null if not (e.g. older Claude models).
- §11.2 Replay is async — LLM call latency; result populated when complete.
- §11.3 memory audit body: trace_id, run_id, kind; prompt + response SHA256.
- §11.4 Drift alert via TASK-CHAT-005 to CDO with link to trace + replay diff.
- §11.5 Partition same as TASK-CUO-102 — monthly, 7-year retention.

---

*End of TASK-CUO-103 spec.*
