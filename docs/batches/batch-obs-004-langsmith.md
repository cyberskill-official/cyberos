---
batch: batch/obs-004-langsmith
members:
  - TASK-OBS-004
started: 2026-07-26T02:50:00Z
ended: null
route_backs: 0
gate_reasks: 0
tokens: unknown
---

# batch/obs-004-langsmith — close OBS-004 gaps on as-built gateway

## Scope

Finish TASK-OBS-004 against the existing `services/ai-gateway` LangSmith module (already
hooked from `server::chat`). Close the remaining ACs that blocked `ready_to_implement`:

- Prometheus metrics (`ai_langsmith_exports_total`, latency histogram, queue depth)
- Per-region URL map + vn-1 drop (`langsmith-config.yaml`)
- CLI `--langsmith-export` + `obs.langsmith_export_enabled` audit emit
- Deploy compose for local self-hosted LangSmith
- Integration tests with mock HTTP (opt-in/out, retries, auth-fail, non-blocking, no-PII)

## Out of scope (ledgered)

- Wiring live `cost_usd` from the cost ledger into the export (still `0.0` until the
  non-streaming reconcile path feeds the chat handler)
- Persona handle on the chat HTTP path (request type has no persona field yet)
- Splitting `langsmith/mod.rs` into `client.rs` / `payload.rs` (as-built single module)

## Verification

```sh
cd services/ai-gateway
cargo test --lib langsmith -- --test-threads=1
cargo test --test langsmith_test --test langsmith_no_pii_test -- --test-threads=1
```

## HITL

Status advanced to `ready_to_review`. Gate-1 (review → ready_to_test) and Gate-2
(testing → done) still require operator verdicts — do not flip to `done` in this PR.
