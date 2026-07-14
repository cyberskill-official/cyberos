---
id: TASK-AI-104
title: "AI VN provider integration — Viettel Cloud + FPT Cloud as Vn1-residency LLM/embedding providers with TASK-AI-016 region set extension"
module: AI
priority: SHOULD
status: draft
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-016, TASK-AI-006, TASK-AUTH-105, TASK-MEMORY-111]
depends_on: [TASK-AI-016]
blocks: []

source_pages:
  - website/docs/modules/ai.html#vn-providers
  - https://viettelcloud.vn/
  - https://fptcloud.com/

source_decisions:
  - DEC-2380 2026-05-17 — Add Viettel Cloud + FPT Cloud as approved Vn1-residency providers; integrates with TASK-AI-016 region resolution
  - DEC-2381 2026-05-17 — Closed enum `vn_provider` = {viettel_cloud, fpt_cloud}; cardinality 2 (extensible)
  - DEC-2382 2026-05-17 — Per-tenant provider creds in KMS (CTO-only writes); contract negotiation handled outside system
  - DEC-2383 2026-05-17 — Failover: if primary (Viettel) down, fall back to FPT; both down → return Vn1 refusal per TASK-AI-016 contract
  - DEC-2384 2026-05-17 — memory audit kinds: ai.vn_provider_invoked, ai.vn_provider_failover, ai.vn_provider_both_down, ai.vn_provider_creds_set

build_envelope:
  language: rust 1.81
  service: cyberos/services/ai/
  new_files:
    - services/ai/migrations/0010_vn_provider_creds.sql
    - services/ai/src/providers/vn/mod.rs
    - services/ai/src/providers/vn/viettel_client.rs
    - services/ai/src/providers/vn/fpt_client.rs
    - services/ai/src/providers/vn/failover.rs
    - services/ai/src/audit/vn_provider_events.rs
    - services/ai/tests/vn_provider_enum_cardinality_test.rs
    - services/ai/tests/viettel_invocation_test.rs
    - services/ai/tests/fpt_failover_test.rs
    - services/ai/tests/both_down_refusal_test.rs
    - services/ai/tests/vn_provider_audit_emission_test.rs

  modified_files:
    - services/ai/src/residency_resolver.rs

  allowed_tools:
    - file_read: services/{ai,auth}/**
    - file_write: services/ai/{src,tests,migrations}/**
    - bash: cd services/ai && cargo test vn_provider

  disallowed_tools:
    - non-CTO creds write (per DEC-2382)
    - silent fallback non-VN (per DEC-2380)

effort_hours: 12
subtasks:
  - "0.3h: 0010_vn_provider_creds.sql"
  - "0.4h: vn/mod.rs"
  - "2.5h: viettel_client.rs"
  - "2.5h: fpt_client.rs"
  - "0.6h: failover.rs"
  - "0.3h: audit/vn_provider_events.rs"
  - "0.4h: residency_resolver.rs hook"
  - "4.0h: tests — 5 test files"
  - "1.0h: docs + CTO UI"

risk_if_skipped: "Without VN providers, Vn1 tenants stuck (TASK-AI-016 refuses). Without DEC-2383 failover, single-provider outage = Vn1 service down. Without DEC-2380 region set extension, TASK-AI-016 contract unfulfilled."
---

## §1 — Description (BCP-14 normative)

The AI service **MUST** ship VN provider integration at `services/ai/src/providers/vn/` adding Viettel + FPT to TASK-AI-016 region set, failover, 4 memory audit kinds.

1. **MUST** validate `vn_provider` against closed enum per DEC-2381.

2. **MUST** add to TASK-AI-016 region set per DEC-2380 — modify `residency_resolver.rs` to include Viettel + FPT region strings for `Vn1`.

3. **MUST** dispatch at `vn/mod.rs::dispatch(tenant, request)` with failover per DEC-2383:
   - Primary: Viettel; on 5xx/timeout → FPT
   - Both down → return `vn1_provider_outage` (distinct from `vn1_no_provider_yet`)

4. **MUST** store creds in KMS per DEC-2382 (CTO-only).

5. **MUST** define table at migration `0010`:
   ```sql
   CREATE TABLE ai_vn_provider_creds (
     tenant_id UUID NOT NULL,
     provider TEXT NOT NULL CHECK (provider IN ('viettel_cloud','fpt_cloud')),
     encrypted_creds_arn TEXT NOT NULL,
     api_account_id TEXT,
     active BOOLEAN NOT NULL DEFAULT true,
     set_by UUID NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     PRIMARY KEY (tenant_id, provider)
   );
   ALTER TABLE ai_vn_provider_creds ENABLE ROW LEVEL SECURITY;
   CREATE POLICY vn_creds_rls ON ai_vn_provider_creds
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (encrypted_creds_arn, api_account_id, active, set_by, updated_at) ON ai_vn_provider_creds TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   PUT /v1/ai/vn-providers/{provider}/creds      (CTO-only)
   GET /v1/ai/vn-providers/health                 (per-provider status)
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2384. PII per TASK-MEMORY-111: prompts hashed at TASK-AI-006 layer (not duplicated here).

8. **MUST** thread trace_id from request → dispatcher → audit.

9. **MUST NOT** silently fall back outside VN per DEC-2380 (preserves regulatory contract).

10. **MUST NOT** allow non-CTO creds write per DEC-2382.

---

## §2 — Why this design

**Why Viettel + FPT (DEC-2380)?** Major VN cloud providers with data centers in VN; both serve LLM/embedding via partner agreements.

**Why failover (DEC-2383)?** Either provider individually unreliable; pair gives 99.5% combined uptime.

**Why CTO creds (DEC-2382)?** Provider integration involves contract terms + billing; CTO authority.

**Why preserve refusal contract (DEC-2380)?** TASK-AI-016 `vn1_no_provider_yet` becomes `vn1_provider_outage` after integration — both-down case still refused, never silently routed elsewhere.

---

## §3 — API contract

Sample provider health:
```json
{
  "providers": [
    {"provider": "viettel_cloud", "active": true, "status": "healthy"},
    {"provider": "fpt_cloud", "active": true, "status": "healthy"}
  ]
}
```

---

## §4 — Acceptance criteria
1. **vn_provider enum cardinality 2**. 2. **TASK-AI-016 region set extended**. 3. **Viettel primary**. 4. **FPT failover on Viettel 5xx**. 5. **Both down → refusal (not silent reroute)**. 6. **Refusal code `vn1_provider_outage` distinct from `vn1_no_provider_yet`**. 7. **CTO-only creds**. 8. **Creds in KMS**. 9. **4 memory audit kinds emitted**. 10. **PII via TASK-AI-006**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Per-provider active flag**. 14. **Health endpoint shows status**. 15. **Append-only via REVOKE except status cols**. 16. **Failover latency < 500ms**. 17. **Both-provider creds optional (single OK if other unavailable)**. 18. **Contract terms documented per provider**. 19. **Inactive provider skipped in dispatch**. 20. **Cross-tenant cred isolation**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn viettel_primary_dispatch() {
    let ctx = TestContext::with_vn_creds_both().await;
    let r = ctx.invoke("hello", "vn-1").await;
    assert!(r.dispatched_to == "viettel_cloud");
}

#[tokio::test]
async fn fpt_failover_on_viettel_5xx() {
    let ctx = TestContext::with_vn_creds_both().await;
    ctx.mock_viettel_500().await;
    let r = ctx.invoke("hello", "vn-1").await;
    assert!(r.dispatched_to == "fpt_cloud");
    let audits = ctx.fetch_memory_audits("ai.vn_provider_failover").await;
    assert!(!audits.is_empty());
}

#[tokio::test]
async fn both_down_refusal() {
    let ctx = TestContext::with_vn_creds_both().await;
    ctx.mock_viettel_500().await;
    ctx.mock_fpt_500().await;
    let r = ctx.try_invoke("hello", "vn-1").await;
    assert!(r.is_err());
    assert!(r.error_code() == "vn1_provider_outage");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-AI-016.
**Cross-module:** TASK-AI-006 (provider abstraction), TASK-AUTH-105 (KMS), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Viettel API down | client err | failover to FPT | inherent |
| FPT API down | client err | both-down refusal | inherent |
| Creds expired | 401 | sev-1; failover or refuse | CTO rotate |
| Cross-tenant cred | RLS | 0 rows | inherent |
| Provider deprecates API v1 | per-client version | upgrade required | maintenance |
| Network partition to VN | timeout | refusal | inherent |
| Both providers concurrent quota | rate-limit | refusal | upgrade |
| Inactive provider with active flag false | skip | inherent | inherent |
| Concurrent dispatch | inherent | each isolated | inherent |
| Decimal precision N/A | inherent | inherent | inherent |

## §11 — Implementation notes
- §11.1 Viettel + FPT REST APIs; auth via API key + tenant-scoped quota.
- §11.2 Failover threshold: 1 attempt at primary (no retry); immediate fall to secondary.
- §11.3 memory audit body: tenant_id, provider, status; prompts hashed via TASK-AI-006.
- §11.4 Future: add VNG Cloud or other VN providers via enum extension.
- §11.5 Per-tenant active flags allow CTO to disable a provider without removing creds.

---

*End of TASK-AI-104 spec.*
