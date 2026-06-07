---
id: FR-DOC-007
title: "DOC lifecycle metadata — parties + effective_date + expiry_date + renewal_terms + parent_contract_id for contract document substrate"
module: DOC
priority: MUST
status: ready_to_implement
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-DOC-001, FR-DOC-008, FR-DOC-009, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-DOC-001]
blocks: [FR-DOC-008, FR-DOC-009]

source_pages:
  - website/docs/modules/doc.html#lifecycle

source_decisions:
  - DEC-1710 2026-05-17 — Lifecycle metadata = first-class columns: parties[], effective_date, expiry_date, renewal_terms (jsonb), parent_contract_id (self-ref for amendments)
  - DEC-1711 2026-05-17 — Closed enum `lifecycle_status` = {draft, active, expiring, expired, terminated, renewed}; cardinality 6
  - DEC-1712 2026-05-17 — Auto-status computation: now < effective_date → draft; effective ≤ now < expiry-90d → active; expiry-90d ≤ now < expiry → expiring; now >= expiry → expired
  - DEC-1713 2026-05-17 — Parties stored as JSONB array: `[{party_id, party_type, role}]`; party_type enum closed
  - DEC-1714 2026-05-17 — Parent contract chain: amendments point to parent via parent_contract_id; UI shows tree
  - DEC-1715 2026-05-17 — memory audit kinds: doc.lifecycle_set, doc.lifecycle_status_computed, doc.parent_link_added

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0002_lifecycle_metadata.sql
    - services/doc/src/lifecycle/mod.rs
    - services/doc/src/lifecycle/status_computer.rs
    - services/doc/src/handlers/lifecycle_routes.rs
    - services/doc/src/audit/lifecycle_events.rs
    - services/doc/tests/lifecycle_status_computed_test.rs
    - services/doc/tests/lifecycle_parent_chain_test.rs
    - services/doc/tests/lifecycle_status_enum_cardinality_test.rs
    - services/doc/tests/lifecycle_parties_jsonb_test.rs
    - services/doc/tests/lifecycle_audit_emission_test.rs

  modified_files:
    - services/doc/src/documents.rs

  allowed_tools:
    - file_read: services/doc/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test lifecycle

  disallowed_tools:
    - mutate prior parent link (per DEC-1714)
    - skip status compute on lifecycle field change (per DEC-1712)

effort_hours: 5
sub_tasks:
  - "0.3h: 0002_lifecycle_metadata.sql"
  - "0.4h: lifecycle/mod.rs"
  - "0.5h: status_computer.rs"
  - "0.4h: handlers/lifecycle_routes.rs"
  - "0.3h: audit/lifecycle_events.rs"
  - "0.3h: documents.rs hook"
  - "2.0h: tests — 5 test files"
  - "0.8h: docs + UI lifecycle widget"

risk_if_skipped: "Without lifecycle metadata, contracts can't be queried by status/expiry — FR-DOC-008 alerts impossible. Without DEC-1712 auto-status, manual status updates drift from reality. Without DEC-1714 parent chain, amendments float disconnected from parents."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** extend FR-DOC-001 documents with lifecycle metadata at `services/doc/src/lifecycle/` — parties, dates, renewal terms, parent chain, auto-status, 3 memory audit kinds.

1. **MUST** define table extension at migration `0002`:
   ```sql
   ALTER TABLE doc_documents ADD COLUMN parties JSONB;
   ALTER TABLE doc_documents ADD COLUMN effective_date DATE;
   ALTER TABLE doc_documents ADD COLUMN expiry_date DATE;
   ALTER TABLE doc_documents ADD COLUMN renewal_terms JSONB;
   ALTER TABLE doc_documents ADD COLUMN parent_contract_id UUID REFERENCES doc_documents(document_id);
   ALTER TABLE doc_documents ADD COLUMN lifecycle_status TEXT
     CHECK (lifecycle_status IS NULL OR lifecycle_status IN
       ('draft','active','expiring','expired','terminated','renewed'));
   ALTER TABLE doc_documents ADD COLUMN status_computed_at TIMESTAMPTZ;
   CREATE INDEX docs_expiry_idx ON doc_documents(tenant_id, expiry_date)
     WHERE expiry_date IS NOT NULL;
   CREATE INDEX docs_parent_idx ON doc_documents(tenant_id, parent_contract_id)
     WHERE parent_contract_id IS NOT NULL;
   GRANT UPDATE (parties, effective_date, expiry_date, renewal_terms,
                 parent_contract_id, lifecycle_status, status_computed_at) ON doc_documents TO cyberos_app;
   ```

2. **MUST** validate `lifecycle_status` against closed enum per DEC-1711.

3. **MUST** validate parties JSONB structure per DEC-1713 — array of `{party_id, party_type, role}`. party_type ∈ {tenant, customer, vendor, employee, authority}.

4. **MUST** compute status at `status_computer.rs::compute(doc, now)` per DEC-1712:
   - Triggered on field change + nightly cron (FR-MCP-007).
   - Updates `lifecycle_status` + `status_computed_at`.

5. **MUST** support parent chain per DEC-1714 — amendments link via `parent_contract_id`; expose tree query endpoint.

6. **MUST** expose endpoints:
   ```text
   PUT    /v1/doc/documents/{id}/lifecycle    body: {parties, effective_date, expiry_date, renewal_terms, parent_contract_id?}
   GET    /v1/doc/documents/{id}/lifecycle
   GET    /v1/doc/documents/{id}/parent-chain   (returns ancestor tree)
   ```

7. **MUST** emit 3 memory audit kinds per DEC-1715. PII per FR-MEMORY-111: parties JSON hashed; dates and status enum ok.

8. **MUST** thread trace_id from set → compute → audit.

9. **MUST NOT** mutate prior `parent_contract_id` (parent re-link rare; if needed, new row).

10. **MUST NOT** skip auto-status on field change per DEC-1712.

---

## §2 — Why this design

**Why first-class columns (DEC-1710)?** JSONB-only would force every query to parse; columnar = indexable + queryable for FR-DOC-008.

**Why auto-status (DEC-1712)?** Manual updates drift; computed status reflects truth at query time.

**Why party type enum (DEC-1713)?** Roles drive notification routing (employees ≠ customers ≠ authorities).

**Why parent chain (DEC-1714)?** Amendments form contract lineage; legal needs to see full tree.

---

## §3 — API contract

Sample lifecycle:
```json
{
  "document_id": "uuid",
  "parties": [
    {"party_id": "uuid-tenant", "party_type": "tenant", "role": "service_provider"},
    {"party_id": "uuid-acme", "party_type": "customer", "role": "client"}
  ],
  "effective_date": "2026-01-01",
  "expiry_date": "2027-12-31",
  "renewal_terms": {"auto_renew": true, "notice_days": 60, "term_months": 24},
  "parent_contract_id": null,
  "lifecycle_status": "active",
  "status_computed_at": "2026-05-17T02:00:00Z"
}
```

Parent chain:
```json
[
  {"document_id": "uuid-amendment-2", "level": 0},
  {"document_id": "uuid-amendment-1", "level": 1},
  {"document_id": "uuid-original-msa", "level": 2}
]
```

---

## §4 — Acceptance criteria
1. **All lifecycle fields settable**. 2. **6-status enum + cardinality test**. 3. **Auto-status compute on field change**. 4. **Nightly cron refresh**. 5. **Parties JSONB structure validated**. 6. **Party type enum (5: tenant/customer/vendor/employee/authority)**. 7. **Parent chain query returns ancestors**. 8. **Indexed on expiry_date for FR-DOC-008**. 9. **Indexed on parent_contract_id for tree query**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (parties JSONB SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Status thresholds correct (90d = expiring)**. 15. **Terminated status manual-only (CLO action)**. 16. **Renewed status set on renewal contract creation**. 17. **Append-only via REVOKE UPDATE except 7 cols**. 18. **NULL allowed for legacy docs**. 19. **Self-referential FK enforced**. 20. **Status compute idempotent (same input → same result)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn status_active_within_window() {
    let ctx = TestContext::doc_with_dates("2026-01-01", "2027-12-31").await;
    ctx.compute_status(ctx.doc_id, "2026-06-01").await;
    let d = ctx.fetch_doc(ctx.doc_id).await;
    assert_eq!(d.lifecycle_status.as_deref(), Some("active"));
}

#[tokio::test]
async fn status_expiring_within_90d() {
    let ctx = TestContext::doc_with_dates("2026-01-01", "2026-06-30").await;
    ctx.compute_status(ctx.doc_id, "2026-04-15").await;
    let d = ctx.fetch_doc(ctx.doc_id).await;
    assert_eq!(d.lifecycle_status.as_deref(), Some("expiring"));
}

#[tokio::test]
async fn parent_chain_returns_ancestors() {
    let ctx = TestContext::with_amendment_chain(3).await;
    let chain = ctx.fetch_parent_chain(ctx.leaf_id).await;
    assert_eq!(chain.len(), 3);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-DOC-001.
**Downstream:** FR-DOC-008 (expiry alerts), FR-DOC-009 (renewal proposals).
**Cross-module:** FR-MCP-007 (cron), FR-AI-003 (parties extraction skill — future), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid party_type | validate | reject 400 | use valid |
| Expiry before effective | validate | reject 400 | fix dates |
| Self-referential parent | validate | reject (cycle) | fix link |
| Parent chain >20 deep | sanity warn | sev-3; allow | inherent |
| Status compute fails on bad date | catch | NULL + sev-2 | fix data |
| JSONB schema mismatch | parse | reject 400 | fix structure |
| Renewal_terms unparseable | validate | reject 400 | fix |
| Cross-tenant parent FK | RLS | 404 (treat as missing) | inherent |
| Concurrent status compute | last-writer-wins | inherent | inherent |
| Cron skipped | catch on next | inherent | inherent |

## §11 — Implementation notes
- §11.1 Status computer pure function: `(effective, expiry, now) → status`.
- §11.2 Nightly cron runs at 03:00 tenant_tz, recomputes all docs with expiry_date set.
- §11.3 Parent chain query recursive CTE: `WITH RECURSIVE chain AS (...)`.
- §11.4 memory audit body: doc_id, status, status_computed_at; parties JSONB hashed.
- §11.5 Future FR can ingest contract PDFs and auto-extract parties via FR-AI-003 — this FR provides the schema.

---

*End of FR-DOC-007 spec.*
