---
id: TASK-OKR-007
title: "OKR quarterly retro CUO draft — auto-generated retro with face-saving Vietnamese framing for honest reflection"
module: OKR
priority: SHOULD
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OKR-001, TASK-OKR-005, TASK-CUO-101, TASK-MEMORY-111]
depends_on: [TASK-OKR-001, TASK-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/okr.html#quarterly-retro

source_decisions:
  - DEC-2020 2026-05-17 — Auto-drafted on Q-end (Mar 31 / Jun 30 / Sep 30 / Dec 31) tenant_tz; CEO review queue
  - DEC-2021 2026-05-17 — Closed enum `retro_framing` = {vn_face_saving, direct_western, balanced}; cardinality 3
  - DEC-2022 2026-05-17 — VN face-saving framing emphasizes lessons + future actions; minimizes blame; uses softer language patterns from VN business culture
  - DEC-2023 2026-05-17 — Sections: KR results summary, hits & misses, lessons learned, next-quarter recommendations
  - DEC-2024 2026-05-17 — Draft requires CEO review before memory persistence — never auto-persist (TASK-CRM-007 pattern)
  - DEC-2025 2026-05-17 — memory audit kinds: okr.retro_drafted, okr.retro_approved, okr.retro_dismissed, okr.retro_persisted

build_envelope:
  language: rust 1.81
  service: cyberos/services/okr/
  new_files:
    - services/okr/migrations/0007_quarterly_retros.sql
    - services/okr/src/retro/mod.rs
    - services/okr/src/retro/draft_generator.rs
    - services/okr/src/retro/framing_picker.rs
    - services/okr/src/retro/memory_persister.rs
    - services/okr/src/handlers/retro_routes.rs
    - services/okr/src/audit/retro_events.rs
    - services/okr/tests/retro_auto_on_q_end_test.rs
    - services/okr/tests/retro_framing_enum_cardinality_test.rs
    - services/okr/tests/retro_vn_framing_test.rs
    - services/okr/tests/retro_no_auto_persist_test.rs
    - services/okr/tests/retro_audit_emission_test.rs

  modified_files:
    - services/okr/src/lib.rs

  allowed_tools:
    - file_read: services/{okr,cuo,memory,ai}/**
    - file_write: services/okr/{src,tests,migrations}/**
    - bash: cd services/okr && cargo test retro

  disallowed_tools:
    - auto-persist memory memory (per DEC-2024)
    - blame-heavy framing for VN tenants (per DEC-2022)

effort_hours: 6
subtasks:
  - "0.3h: 0007_quarterly_retros.sql"
  - "0.3h: retro/mod.rs"
  - "0.7h: draft_generator.rs"
  - "0.4h: framing_picker.rs"
  - "0.4h: memory_persister.rs"
  - "0.4h: handlers/retro_routes.rs"
  - "0.3h: audit/retro_events.rs"
  - "2.0h: tests — 5 test files"
  - "1.2h: CEO UI for retro review + docs"

risk_if_skipped: "Without retro draft, CEO writes manually each quarter (16h+ work). Without DEC-2024 manual review, AI hallucination locks wrong narrative as truth. Without DEC-2022 VN framing, blame-heavy retros damage VN team morale."
---

## §1 — Description (BCP-14 normative)

The OKR service **MUST** ship quarterly retro at `services/okr/src/retro/` auto-drafted Q-end, framing-aware, CEO-reviewed, memory-persisted on approval, 4 memory audit kinds.

1. **MUST** trigger on Q-end per DEC-2020 via TASK-MCP-007 cron (Mar 31, Jun 30, Sep 30, Dec 31).

2. **MUST** validate `retro_framing` against closed enum per DEC-2021.

3. **MUST** pick framing at `framing_picker.rs::pick(tenant)`:
   - tenant.residency = vn-1 → default vn_face_saving
   - Other → balanced
   - CEO can override per retro

4. **MUST** draft at `draft_generator.rs::draft(tenant, quarter, framing)` per DEC-2023:
   - Pull all KRs of quarter; compute hit/miss
   - Pull check-ins from TASK-OKR-005
   - TASK-AI-003 prompt with framing-aware tone
   - Output: results_summary, hits_and_misses, lessons_learned, next_quarter_recommendations

5. **MUST** queue for CEO review per DEC-2024 — never auto-persist.

6. **MUST** persist to memory on approval at `memory_persister.rs::persist(retro)` per TASK-CRM-007 pattern — searchable kind=lessons.

7. **MUST** define table at migration `0007`:
   ```sql
   CREATE TABLE okr_quarterly_retros (
     retro_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     quarter CHAR(7) NOT NULL,  -- 'YYYY-Q1' format
     framing TEXT NOT NULL CHECK (framing IN ('vn_face_saving','direct_western','balanced')),
     draft_jsonb JSONB NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending_review'
       CHECK (status IN ('pending_review','approved','dismissed','persisted')),
     reviewed_by UUID,
     reviewed_at TIMESTAMPTZ,
     memory_memory_path TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, quarter)
   );
   ALTER TABLE okr_quarterly_retros ENABLE ROW LEVEL SECURITY;
   CREATE POLICY retros_rls ON okr_quarterly_retros
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON okr_quarterly_retros FROM cyberos_app;
   GRANT UPDATE (framing, draft_jsonb, status, reviewed_by, reviewed_at, memory_memory_path) ON okr_quarterly_retros TO cyberos_app;
   ```

8. **MUST** expose endpoints:
   ```text
   GET  /v1/okr/retros                          (list)
   POST /v1/okr/retros/{id}/approve             (CEO; persists to memory)
   POST /v1/okr/retros/{id}/dismiss
   POST /v1/okr/retros/{id}/regenerate          body: {framing}  (re-draft with different framing)
   ```

9. **MUST** emit 4 memory audit kinds per DEC-2025. PII per TASK-MEMORY-111: draft text SHA-256 hashed.

10. **MUST** thread trace_id from cron → draft → CEO review → memory persist → audit.

11. **MUST NOT** auto-persist per DEC-2024.

12. **MUST NOT** use blame-heavy framing for vn-1 tenants per DEC-2022 (default to vn_face_saving).

---

## §2 — Why this design

**Why VN face-saving (DEC-2022)?** VN business culture values harmony; blame-heavy retros undermine team trust + future honesty.

**Why 3 framings (DEC-2021)?** Cultural fit varies; CEO picks per audience.

**Why CEO review (DEC-2024)?** Retro narrative becomes institutional memory; mistakes here persist for years.

**Why memory persist (DEC-2025)?** Future retros reference past lessons; without persist, learning evaporates.

---

## §3 — API contract

Sample retro draft:
```json
{
  "retro_id": "uuid",
  "quarter": "2026-Q2",
  "framing": "vn_face_saving",
  "draft_jsonb": {
    "results_summary": "Q2 đã đạt 8/10 KRs (80%)...",
    "hits_and_misses": [...],
    "lessons_learned": ["Quy trình tuyển dụng cần được cải thiện..."],
    "next_quarter_recommendations": [...]
  },
  "status": "pending_review"
}
```

---

## §4 — Acceptance criteria
1. **Auto-trigger Q-end cron**. 2. **retro_framing enum cardinality 3**. 3. **VN tenant defaults to vn_face_saving**. 4. **Other tenant defaults to balanced**. 5. **4 sections drafted**. 6. **CEO can override framing**. 7. **Regenerate endpoint with new framing**. 8. **Never auto-persist to memory**. 9. **Approve writes memory memory**. 10. **4 memory audit kinds emitted**. 11. **PII scrubbed (draft text SHA256)**. 12. **RLS denies cross-tenant**. 13. **CEO-only review**. 14. **Trace_id preserved**. 15. **UNIQUE(tenant_id, quarter) idempotency**. 16. **Append-only via REVOKE except status cols**. 17. **memory_memory_path populated post-approve**. 18. **Dismiss → status=dismissed (audit retained)**. 19. **AI failure → status=failed + sev-2 + retry**. 20. **Status workflow: pending_review → approved → persisted | dismissed**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn vn_tenant_defaults_vn_framing() {
    let ctx = TestContext::vn_tenant().await;
    ctx.run_quarter_end_cron("2026-Q2").await;
    let retro = ctx.fetch_retro_draft("2026-Q2").await;
    assert_eq!(retro.framing, "vn_face_saving");
}

#[tokio::test]
async fn never_auto_persists() {
    let ctx = TestContext::with_retro_draft().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let mem_count = ctx.memory_memory_count_for_retro(ctx.retro_id).await;
    assert_eq!(mem_count, 0);
    let retro = ctx.fetch_retro(ctx.retro_id).await;
    assert_eq!(retro.status, "pending_review");
}

#[tokio::test]
async fn approve_writes_memory() {
    let ctx = TestContext::with_pending_retro().await;
    ctx.approve_retro_as_ceo(ctx.retro_id).await;
    let retro = ctx.fetch_retro(ctx.retro_id).await;
    assert_eq!(retro.status, "persisted");
    assert!(retro.memory_memory_path.is_some());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-OKR-001, TASK-CUO-101.
**Cross-module:** TASK-OKR-005 (check-in history), TASK-AI-003 (draft + framing), TASK-MEMORY-111 (PII + persist), TASK-AUTH-101 (CEO role), TASK-MCP-007 (cron).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| AI timeout | retry 1x | sev-2; minimal draft fallback | manual fill |
| Duplicate quarter | UNIQUE | skip | inherent |
| 0 KRs in quarter | inherent | empty draft + sev-3 | inherent |
| memory write fail | sev-1 | retry | inherent |
| User dismisses | status=dismissed | retained for audit | inherent |
| Regenerate while reviewing | last-wins | inherent | inherent |
| Wrong framing for tenant | CEO override | inherent | regenerate |
| Cross-tenant approve | RLS | 403 | inherent |
| Concurrent approve | UPDATE WHERE pending | first wins | inherent |
| Q-end on weekend | cron runs Monday | inherent | inherent |

## §11 — Implementation notes
- §11.1 Cron via TASK-MCP-007 `kind: 'okr.quarterly_retro'`, runs Q-end + 1 day.
- §11.2 AI prompt varies by framing — VN face-saving emphasizes "we learned"; western direct emphasizes "what went wrong".
- §11.3 memory memory path: `memories/lessons/okr/<tenant>/<quarter>.md`.
- §11.4 memory audit body: retro_id, quarter, framing; draft text SHA256.
- §11.5 Future retros via TASK-MEMORY-108 search return past lessons for context.

---

*End of TASK-OKR-007 spec.*
