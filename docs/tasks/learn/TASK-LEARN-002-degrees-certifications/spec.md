---
id: TASK-LEARN-002
title: "LEARN bằng cấp + chứng chỉ — degree + certification evidence types with issuer + expiry + verification link"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: LEARN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-LEARN-001, TASK-DOC-001, TASK-MEMORY-111]
depends_on: [TASK-LEARN-001]
blocks: []

source_pages:
  - website/docs/modules/learn.html#degrees-certs

source_decisions:
  - DEC-2090 2026-05-17 — Per-Member evidence records: degree (bachelor/master/PhD) or certification (AWS/PMP/etc.) with issuer + date + expiry + verification URL
  - DEC-2091 2026-05-17 — Closed enum `evidence_kind` = {degree, certification, course_completion, license, award}; cardinality 5
  - DEC-2092 2026-05-17 — Evidence linked to skill_id (TASK-LEARN-001); upload scan/photo via TASK-DOC-001
  - DEC-2093 2026-05-17 — Expiry monitoring: nightly batch flags expired/expiring (30d) certifications; CHRO notification
  - DEC-2094 2026-05-17 — memory audit kinds: learn.evidence_added, learn.evidence_verified, learn.evidence_expired, learn.evidence_renewed

build_envelope:
  language: rust 1.81
  service: cyberos/services/learn/
  new_files:
    - services/learn/migrations/0002_evidence.sql
    - services/learn/src/evidence/mod.rs
    - services/learn/src/evidence/expiry_cron.rs
    - services/learn/src/handlers/evidence_routes.rs
    - services/learn/src/audit/evidence_events.rs
    - services/learn/tests/evidence_kind_enum_cardinality_test.rs
    - services/learn/tests/evidence_expiry_alert_test.rs
    - services/learn/tests/evidence_skill_link_test.rs
    - services/learn/tests/evidence_doc_ref_test.rs
    - services/learn/tests/evidence_audit_emission_test.rs

  modified_files:
    - services/learn/src/lib.rs

  allowed_tools:
    - file_read: services/{learn,doc}/**
    - file_write: services/learn/{src,tests,migrations}/**
    - bash: cd services/learn && cargo test evidence

  disallowed_tools:
    - mutate prior evidence (per DEC-2090 append-only)

effort_hours: 4
subtasks:
  - "0.3h: 0002_evidence.sql"
  - "0.3h: evidence/mod.rs"
  - "0.5h: expiry_cron.rs"
  - "0.4h: handlers/evidence_routes.rs"
  - "0.3h: audit/evidence_events.rs"
  - "1.6h: tests — 5 test files"
  - "0.4h: docs + UI"
  - "0.2h: cron registration"

risk_if_skipped: "Without evidence records, mastery claims unverifiable. Without DEC-2093 expiry alert, expired certs persist in member profile (audit failure). Without DEC-2092 DOC link, can't see actual diploma."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship evidence at `services/learn/src/evidence/` with 5-kind closed enum + skill link + TASK-DOC-001 scan ref + expiry cron, 4 memory audit kinds.

1. **MUST** validate `evidence_kind` against closed enum per DEC-2091.

2. **MUST** define table at migration `0002`:
   ```sql
   CREATE TABLE learn_evidence (
     evidence_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     skill_id UUID REFERENCES learn_skills(skill_id),
     kind TEXT NOT NULL CHECK (kind IN ('degree','certification','course_completion','license','award')),
     name TEXT NOT NULL,
     issuer TEXT NOT NULL,
     issued_date DATE NOT NULL,
     expires_at DATE,
     verification_url TEXT,
     doc_id UUID,  -- TASK-DOC-001 scan/photo
     verified BOOLEAN NOT NULL DEFAULT false,
     verified_by UUID,
     verified_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX evidence_member_idx ON learn_evidence(tenant_id, member_id);
   CREATE INDEX evidence_expiry_idx ON learn_evidence(tenant_id, expires_at) WHERE expires_at IS NOT NULL;
   ALTER TABLE learn_evidence ENABLE ROW LEVEL SECURITY;
   CREATE POLICY evidence_rls ON learn_evidence
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_evidence FROM cyberos_app;
   GRANT UPDATE (verified, verified_by, verified_at) ON learn_evidence TO cyberos_app;
   ```

3. **MUST** run expiry cron per DEC-2093 at 02:00 tenant_tz — flag certs expiring ≤30d; notify CHRO via TASK-CHAT-005.

4. **MUST** expose endpoints:
   ```text
   POST /v1/learn/members/{id}/evidence       body: {kind, name, issuer, issued_date, expires_at?, verification_url?, doc_id?, skill_id?}
   POST /v1/learn/evidence/{id}/verify        (CHRO marks verified)
   GET  /v1/learn/members/{id}/evidence       (list)
   ```

5. **MUST** emit 4 memory audit kinds per DEC-2094. PII per TASK-MEMORY-111: name + issuer text SHA256.

6. **MUST** thread trace_id from add → audit.

7. **MUST NOT** mutate prior evidence per DEC-2090 (append-only).

---

## §2 — Why this design

**Why 5 kinds (DEC-2091)?** Covers degree, certification, course, license, award — bounded.

**Why expiry monitoring (DEC-2093)?** Lapsed certifications expose compliance risk (e.g. legal practice cert).

**Why TASK-DOC-001 link (DEC-2092)?** Visual evidence; auditors can see scans without downloading from external systems.

---

## §3 — API contract

Sample evidence:
```json
{
  "kind": "certification",
  "name": "AWS Solutions Architect Associate",
  "issuer": "AWS",
  "issued_date": "2025-01-15",
  "expires_at": "2028-01-15",
  "verification_url": "https://aws.amazon.com/verify/abc-123",
  "doc_id": "uuid-scan",
  "skill_id": "uuid-aws-skill"
}
```

---

## §4 — Acceptance criteria
1. **evidence_kind enum cardinality 5**. 2. **Issuer required**. 3. **Issued_date required**. 4. **expires_at optional**. 5. **doc_id optional TASK-DOC-001 ref**. 6. **skill_id optional TASK-LEARN-001 link**. 7. **Verification flag CHRO-only set**. 8. **Expiry cron 02:00 daily**. 9. **30d-expiring flagged + CHAT notification**. 10. **4 memory audit kinds emitted**. 11. **PII scrubbed (name+issuer SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE except verify cols**. 15. **expiry_idx for fast cron**. 16. **member_idx for profile view**. 17. **Verification_url stored as URL string**. 18. **FK to doc_id allows NULL**. 19. **FK to skill_id allows NULL**. 20. **Renewal = new evidence row (preserves history)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn evidence_kind_enum_enforced() {
    let r = ctx.add_evidence(ctx.member_id, "invalid_kind", ...).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn expiry_30d_flagged() {
    let ctx = TestContext::with_cert_expiring_in_25d().await;
    ctx.run_expiry_cron().await;
    let audits = ctx.fetch_memory_audits("learn.evidence_expired").await;
    assert!(!audits.is_empty() || ctx.check_expiring_alert_sent().await);
}

#[tokio::test]
async fn append_only_no_update() {
    let ctx = TestContext::with_evidence().await;
    let r = ctx.try_update_evidence(ctx.evidence_id, "new name").await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-LEARN-001.
**Cross-module:** TASK-DOC-001 (scans), TASK-MCP-007 (cron), TASK-CHAT-005 (notification), TASK-AUTH-101 (CHRO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid kind | CHECK | 400 | use valid |
| Missing issuer | validate | 400 | provide |
| Cross-tenant doc_id | FK + RLS | 404 | inherent |
| Expiry cron skipped | catch-up | inherent | inherent |
| Verification by non-CHRO | role check | 403 | request CHRO |
| Expired evidence renewal | new row | inherent | inherent |
| Skill_id deleted | FK NULL | inherent | data fix |
| verification_url malformed | validate | 400 | fix |
| Cross-tenant evidence | RLS | 0 rows | inherent |
| Large bulk import | batch | inherent | inherent |

## §11 — Implementation notes
- §11.1 Expiry cron via TASK-MCP-007 `kind: 'learn.evidence_expiry'`, daily 02:00.
- §11.2 Renewal flow: new evidence row with updated dates; UI links to prior for chain.
- §11.3 memory audit body: member_id, evidence_id, kind; name+issuer SHA256.
- §11.4 Verification process: CHRO manual, optional TASK-AI-003 auto-verify via verification_url scrape.
- §11.5 PDPL: evidence may contain PII; covered by tenant data scope.

---

*End of TASK-LEARN-002 spec.*
