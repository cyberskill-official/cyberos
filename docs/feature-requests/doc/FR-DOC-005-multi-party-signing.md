---
id: FR-DOC-005
title: "DOC multi-party signing workflow — ordered + parallel + counter-sign with reminder cadence and full audit trail"
module: DOC
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-DOC-001, FR-DOC-006, FR-DOC-002, FR-DOC-003, FR-DOC-004, FR-EMAIL-009, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-DOC-001, FR-DOC-006]
blocks: []

source_pages:
  - website/docs/modules/doc.html#multi-party-signing

source_decisions:
  - DEC-1750 2026-05-17 — 3 workflow types: ordered (sequential), parallel (concurrent), counter_sign (initiator → others); closed enum cardinality 3
  - DEC-1751 2026-05-17 — Closed enum `workflow_kind` = {ordered, parallel, counter_sign}; cardinality 3
  - DEC-1752 2026-05-17 — Closed enum `signature_status` = {pending, in_progress, signed, declined, expired, withdrawn}; cardinality 6
  - DEC-1753 2026-05-17 — Reminder cadence: 24h after invite + 72h + 7d; configurable per-tenant
  - DEC-1754 2026-05-17 — Per-signer FR-DOC-006 verification BEFORE signature applied; signature_status='signed' only after both verify+sign succeed
  - DEC-1755 2026-05-17 — Signing routes to CA per signer.region: VN → FR-DOC-004 (VnPay/Viettel-CA), EU → FR-DOC-002 (QTSP), other → FR-DOC-003 (AATL)
  - DEC-1756 2026-05-17 — memory audit kinds: doc.signing_workflow_started, doc.signer_invited, doc.signer_signed, doc.signer_declined, doc.signing_workflow_completed, doc.signing_workflow_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0006_signing_workflows.sql
    - services/doc/src/signing/mod.rs
    - services/doc/src/signing/workflow_engine.rs
    - services/doc/src/signing/ordered_runner.rs
    - services/doc/src/signing/parallel_runner.rs
    - services/doc/src/signing/counter_sign_runner.rs
    - services/doc/src/signing/reminder_cron.rs
    - services/doc/src/signing/ca_router.rs
    - services/doc/src/handlers/signing_routes.rs
    - services/doc/src/audit/signing_events.rs
    - services/doc/tests/signing_ordered_test.rs
    - services/doc/tests/signing_parallel_test.rs
    - services/doc/tests/signing_counter_sign_test.rs
    - services/doc/tests/signing_workflow_kind_enum_cardinality_test.rs
    - services/doc/tests/signing_signature_status_enum_cardinality_test.rs
    - services/doc/tests/signing_decline_blocks_others_test.rs
    - services/doc/tests/signing_reminder_cadence_test.rs
    - services/doc/tests/signing_ca_routing_test.rs
    - services/doc/tests/signing_audit_emission_test.rs

  modified_files:
    - services/doc/src/lib.rs

  allowed_tools:
    - file_read: services/{doc,email,auth}/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test signing

  disallowed_tools:
    - sign without verification (per DEC-1754)
    - skip CA per region (per DEC-1755)

effort_hours: 10
sub_tasks:
  - "0.4h: 0006_signing_workflows.sql"
  - "0.5h: signing/mod.rs"
  - "0.6h: workflow_engine.rs (orchestrator)"
  - "0.5h: ordered_runner.rs"
  - "0.5h: parallel_runner.rs"
  - "0.4h: counter_sign_runner.rs"
  - "0.4h: reminder_cron.rs"
  - "0.5h: ca_router.rs"
  - "0.5h: handlers/signing_routes.rs"
  - "0.3h: audit/signing_events.rs"
  - "3.5h: tests — 9 test files"
  - "2.0h: AM UI for workflow setup + status tracking"
  - "0.4h: docs"

risk_if_skipped: "Without multi-party workflow, contracts signed manually outside CyberOS → audit gap. Without DEC-1754 verify-before-sign, signature applied to wrong identity (court-ineffective). Without DEC-1755 CA routing, VN docs signed by EU CA (regulatory mismatch)."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship multi-party signing at `services/doc/src/signing/` supporting 3 workflow types, FR-DOC-006 verification per signer, FR-DOC-002/003/004 CA routing per region, reminder cron, 6 memory audit kinds.

1. **MUST** expose `POST /v1/doc/documents/{id}/signing-workflows` body `{ workflow_kind, signers: [{signer_id, region, required_assurance_level, position?}], expires_in_days?, reminder_cadence_override? }`.

2. **MUST** validate `workflow_kind` against closed enum per DEC-1751.

3. **MUST** validate `signature_status` against closed enum per DEC-1752.

4. **MUST** dispatch per kind:
   - `ordered_runner.rs::run(workflow)` — invite signer[0], wait for sign, invite signer[1], etc.
   - `parallel_runner.rs::run(workflow)` — invite all signers immediately.
   - `counter_sign_runner.rs::run(workflow)` — initiator (position=0) signs first, then invites others in parallel.

5. **MUST** for each signer per DEC-1754:
   - Invite via FR-EMAIL-009 with sign link.
   - On click: trigger FR-DOC-006 verification (method per signer's region + required_assurance_level).
   - Only if verified=verified: apply signature via FR-DOC-002/003/004 (routed per DEC-1755).
   - Set status=signed; emit audit.

6. **MUST** route CA per signer region per DEC-1755:
   - region=vn → FR-DOC-004 (VN CA)
   - region=eu → FR-DOC-002 (eIDAS QTSP)
   - region=other → FR-DOC-003 (AATL)

7. **MUST** send reminders per DEC-1753 via FR-MCP-007 cron — at 24h, 72h, 7d after invite; configurable per tenant.

8. **MUST** define tables at migration `0006`:
   ```sql
   CREATE TABLE doc_signing_workflows (
     workflow_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     workflow_kind TEXT NOT NULL CHECK (workflow_kind IN ('ordered','parallel','counter_sign')),
     status TEXT NOT NULL DEFAULT 'in_progress'
       CHECK (status IN ('in_progress','completed','withdrawn','expired','failed')),
     expires_at TIMESTAMPTZ NOT NULL,
     reminder_cadence_hours INT[] NOT NULL DEFAULT '{24,72,168}',
     created_by UUID NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ
   );
   ALTER TABLE doc_signing_workflows ENABLE ROW LEVEL SECURITY;
   CREATE POLICY workflows_rls ON doc_signing_workflows
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_signing_workflows FROM cyberos_app;
   GRANT UPDATE (status, completed_at) ON doc_signing_workflows TO cyberos_app;

   CREATE TABLE doc_signers (
     signer_row_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     workflow_id UUID NOT NULL REFERENCES doc_signing_workflows(workflow_id),
     signer_id UUID NOT NULL,
     region TEXT NOT NULL,
     required_assurance_level TEXT NOT NULL CHECK (required_assurance_level IN ('low','substantial','high')),
     position INT,  -- NULL for parallel; ordered/counter_sign use 0,1,2...
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','in_progress','signed','declined','expired','withdrawn')),
     invited_at TIMESTAMPTZ,
     signed_at TIMESTAMPTZ,
     verification_id UUID,  -- FK to FR-DOC-006
     ca_routing TEXT,
     signature_payload BYTEA,
     last_reminder_at TIMESTAMPTZ,
     reminder_count INT NOT NULL DEFAULT 0,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX signers_workflow_idx ON doc_signers(tenant_id, workflow_id, position);
   ALTER TABLE doc_signers ENABLE ROW LEVEL SECURITY;
   CREATE POLICY signers_rls ON doc_signers
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_signers FROM cyberos_app;
   GRANT UPDATE (status, invited_at, signed_at, verification_id, ca_routing,
                 signature_payload, last_reminder_at, reminder_count) ON doc_signers TO cyberos_app;
   ```

9. **MUST** handle decline per DEC-1752 — signer.status=declined → block workflow completion → workflow.status=failed.

10. **MUST** emit 6 memory audit kinds per DEC-1756. PII per FR-MEMORY-111: signer_id (uuid) ok; signature_payload hashed.

11. **MUST** thread trace_id through workflow → invite → verify → sign → audit.

12. **MUST NOT** apply signature without successful FR-DOC-006 verification per DEC-1754.

13. **MUST NOT** route signature to wrong CA per region per DEC-1755.

---

## §2 — Why this design

**Why 3 kinds (DEC-1750)?** Ordered = legal sign chain (junior → senior), parallel = NDA-style (all sign independently), counter_sign = our signature first then customers.

**Why verify-before-sign (DEC-1754)?** Signature attached to wrong identity = signing fraud → court-ineffective.

**Why per-region CA (DEC-1755)?** Regulatory: EU eIDAS, US AATL, VN Decree 130; cross-routing breaks legal validity.

**Why declined blocks workflow (DEC-1752)?** Without all signers, contract incomplete; workflow.status=failed lets AM restart cleanly.

---

## §3 — API contract

```text
POST   /v1/doc/documents/{id}/signing-workflows
GET    /v1/doc/signing-workflows/{id}            (status + signer list)
POST   /v1/doc/signing-workflows/{id}/withdraw   (CLO/AM only)
POST   /v1/doc/signers/{id}/sign                 (signer-authenticated callback)
POST   /v1/doc/signers/{id}/decline              (signer-authenticated)
```

Sample workflow:
```json
{
  "workflow_kind": "ordered",
  "signers": [
    {"signer_id": "uuid-cto", "region": "vn", "required_assurance_level": "high", "position": 0},
    {"signer_id": "uuid-customer", "region": "vn", "required_assurance_level": "substantial", "position": 1}
  ],
  "expires_in_days": 14
}
```

---

## §4 — Acceptance criteria
1. **3-kind workflow enum + cardinality test**. 2. **6-status enum + cardinality test**. 3. **Ordered: signer N+1 invited only after N signs**. 4. **Parallel: all invited at start**. 5. **Counter_sign: position-0 signs then others invited**. 6. **Verification required before sign**. 7. **CA routed per region (vn/eu/other)**. 8. **Reminder cron 24h/72h/7d**. 9. **Reminder cadence configurable per tenant**. 10. **Decline → workflow=failed**. 11. **All signed → workflow=completed**. 12. **Expires at expires_at → workflow=expired + remaining signers=expired**. 13. **6 memory audit kinds emitted**. 14. **PII scrubbed (signature payload SHA256)**. 15. **RLS denies cross-tenant**. 16. **Trace_id preserved**. 17. **Withdraw by initiator allowed**. 18. **Append-only via REVOKE UPDATE except status cols**. 19. **Verification level mismatch → cannot sign**. 20. **Multi-region workflow: each signer routes to correct CA**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn ordered_invites_sequentially() {
    let ctx = TestContext::ordered_workflow(3_signers).await;
    let s1 = ctx.signers()[0];
    assert!(s1.invited_at.is_some());
    let s2 = ctx.signers()[1];
    assert!(s2.invited_at.is_none());
    ctx.sign(s1.id).await;
    let s2_after = ctx.signers()[1];
    assert!(s2_after.invited_at.is_some());
}

#[tokio::test]
async fn parallel_all_invited_at_start() {
    let ctx = TestContext::parallel_workflow(3_signers).await;
    for s in ctx.signers() {
        assert!(s.invited_at.is_some());
    }
}

#[tokio::test]
async fn decline_fails_workflow() {
    let ctx = TestContext::parallel_workflow(3_signers).await;
    ctx.decline(ctx.signers()[0].id).await;
    let wf = ctx.fetch_workflow(ctx.workflow_id).await;
    assert_eq!(wf.status, "failed");
}

#[tokio::test]
async fn ca_routed_by_region() {
    let ctx = TestContext::multi_region_workflow().await;
    ctx.sign_all().await;
    let signers = ctx.signers();
    let vn = signers.iter().find(|s| s.region == "vn").unwrap();
    assert!(vn.ca_routing.as_deref().unwrap().contains("vn-ca"));
    let eu = signers.iter().find(|s| s.region == "eu").unwrap();
    assert!(eu.ca_routing.as_deref().unwrap().contains("qtsp"));
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-DOC-001, FR-DOC-006.
**Cross-module:** FR-DOC-002/003/004 (CA per region), FR-EMAIL-009 (invite + reminder), FR-MCP-007 (reminder cron), FR-AUTH-101 (initiator role), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Verification fails | check pre-sign | signer stays pending; no signature | retry |
| CA route fails | downstream err | signer status=failed; sev-1 | retry |
| Signer email bounces | FR-EMAIL-009 status | sev-2 + reminder via SMS | manual outreach |
| Workflow expires | cron check | status=expired | restart |
| Ordered position skipped | check sequence | error if N+1 invited before N | inherent |
| Decline mid-ordered | block all subsequent | workflow=failed | restart |
| Withdraw during signing | reject signed signers | inherent (signed=immutable) | inherent |
| Multi-CA concurrent | per-signer call | independent | inherent |
| Region mismatch (signer.region invalid) | validate | reject 400 | use valid |
| Cross-tenant workflow | RLS | 403 | inherent |

## §11 — Implementation notes
- §11.1 Sign link is signed token with workflow_id + signer_row_id; expires per workflow.
- §11.2 Reminder cron: per workflow, computes elapsed since invited_at; sends if matches cadence_hours[i].
- §11.3 Workflow engine state machine: in_progress → completed | failed | expired | withdrawn.
- §11.4 memory audit body: workflow_id, signer_id, kind; signature payload SHA256.
- §11.5 CA router maps region to FR-DOC-002/003/004 module; future regions extend the table.

---

*End of FR-DOC-005 spec.*
