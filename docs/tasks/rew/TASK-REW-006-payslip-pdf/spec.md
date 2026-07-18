---
id: TASK-REW-006
title: "REW byte-identical payslip PDF render — Tectonic + pinned fonts produces deterministic PDF bytes for verification"
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
module: REW
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-REW-005, TASK-DOC-001, TASK-MEMORY-111]
depends_on: [TASK-REW-005]
blocks: []

source_pages:
  - website/docs/modules/rew.html#payslip-pdf

source_decisions:
  - DEC-2200 2026-05-17 — PDF rendered via Tectonic (deterministic LaTeX compiler) with pinned font versions; SHA256 of PDF bytes must match across runs for same payslip input
  - DEC-2201 2026-05-17 — Closed enum `pdf_render_status` = {queued, rendering, rendered, verification_failed, failed}; cardinality 5
  - DEC-2202 2026-05-17 — Stored PDF blob in TASK-DOC-001 with sha256 metadata; member can download via portal
  - DEC-2203 2026-05-17 — Verification: post-render, recompute sha256; expected matches deterministic-replay value
  - DEC-2204 2026-05-17 — memory audit kinds: rew.payslip_pdf_rendered, rew.payslip_pdf_verified, rew.payslip_pdf_verification_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0006_payslip_pdfs.sql
    - services/rew/src/pdf/mod.rs
    - services/rew/src/pdf/tectonic_renderer.rs
    - services/rew/src/pdf/verifier.rs
    - services/rew/src/handlers/payslip_pdf_routes.rs
    - services/rew/src/audit/pdf_events.rs
    - services/rew/templates/payslip.tex
    - services/rew/tests/pdf_byte_identical_test.rs
    - services/rew/tests/pdf_render_status_enum_cardinality_test.rs
    - services/rew/tests/pdf_pinned_fonts_test.rs
    - services/rew/tests/pdf_verification_test.rs
    - services/rew/tests/pdf_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/{rew,doc}/**
    - file_write: services/rew/{src,tests,migrations,templates}/**
    - bash: cd services/rew && cargo test pdf

  disallowed_tools:
    - non-pinned fonts (per DEC-2200)
    - bypass verification (per DEC-2203)

effort_hours: 6
subtasks:
  - "0.3h: 0006_payslip_pdfs.sql"
  - "0.3h: pdf/mod.rs"
  - "0.7h: tectonic_renderer.rs"
  - "0.5h: verifier.rs"
  - "0.4h: handlers/payslip_pdf_routes.rs"
  - "0.3h: audit/pdf_events.rs"
  - "0.5h: payslip.tex template"
  - "2.2h: tests — 5 test files"
  - "0.8h: docs"

risk_if_skipped: "Without deterministic render, member challenge 'this is wrong' lacks reproducible proof. Without DEC-2200 Tectonic + pinned fonts, OS upgrades change PDF bytes (audit failure)."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship payslip PDF render at `services/rew/src/pdf/` deterministic via Tectonic + pinned fonts + sha256 verification, 3 memory audit kinds.

1. **MUST** validate `pdf_render_status` against closed enum per DEC-2201.

2. **MUST** render at `tectonic_renderer.rs::render(payslip_row, template)` per DEC-2200:
   - Use Tectonic (deterministic LaTeX compiler)
   - Fonts pinned in template via explicit version
   - Output PDF bytes

3. **MUST** verify at `verifier.rs::verify(pdf_bytes)` per DEC-2203:
   - SHA256 the PDF
   - Compare to expected (from deterministic replay or prior render)
   - Mismatch → sev-1 audit + status=verification_failed

4. **MUST** store in TASK-DOC-001 per DEC-2202 with `sha256` metadata.

5. **MUST** define table at migration `0006`:
   ```sql
   CREATE TABLE rew_payslip_pdfs (
     pdf_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     payslip_id UUID NOT NULL REFERENCES rew_payslip_rows(payslip_id),
     doc_id UUID NOT NULL,  -- TASK-DOC-001 ref
     sha256 CHAR(64) NOT NULL,
     status TEXT NOT NULL DEFAULT 'queued'
       CHECK (status IN ('queued','rendering','rendered','verification_failed','failed')),
     rendered_at TIMESTAMPTZ,
     verified_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (payslip_id)
   );
   ALTER TABLE rew_payslip_pdfs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY pdfs_rls ON rew_payslip_pdfs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_payslip_pdfs FROM cyberos_app;
   GRANT UPDATE (status, doc_id, sha256, rendered_at, verified_at) ON rew_payslip_pdfs TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/rew/payslips/{id}/render          (CFO trigger)
   GET  /v1/rew/payslips/{id}/pdf             (download for member)
   POST /v1/rew/payslips/{id}/verify          (CFO re-verify)
   ```

7. **MUST** emit 3 memory audit kinds per DEC-2204. PII per TASK-MEMORY-111: sha256 in chain (not PII); content never in chain.

8. **MUST** thread trace_id from render → verify → audit.

9. **MUST NOT** use non-pinned fonts per DEC-2200.

10. **MUST NOT** bypass verification per DEC-2203.

---

## §2 — Why this design

**Why Tectonic (DEC-2200)?** LaTeX deterministic when fonts pinned; Tectonic specifically removes non-deterministic timestamps from PDF.

**Why pinned fonts (DEC-2200)?** Font rendering varies by version; different bytes = different sha256 = audit fail.

**Why sha256 verification (DEC-2203)?** Member challenges payslip → CFO can prove same input produces same PDF.

---

## §3 — API contract

```text
POST /v1/rew/payslips/{id}/render
GET  /v1/rew/payslips/{id}/pdf
```

Sample status:
```json
{
  "pdf_id": "uuid",
  "payslip_id": "uuid",
  "sha256": "abc123...",
  "status": "rendered",
  "doc_id": "uuid-doc-ref"
}
```

---

## §4 — Acceptance criteria
1. **pdf_render_status enum cardinality 5**. 2. **Tectonic deterministic**. 3. **Fonts pinned in template**. 4. **SHA256 verified post-render**. 5. **Mismatch → sev-1 + status=verification_failed**. 6. **TASK-DOC-001 storage**. 7. **3 memory audit kinds emitted**. 8. **PII: content never in chain; sha256 ok**. 9. **RLS denies cross-tenant**. 10. **CFO-only render trigger**. 11. **Member can download own payslip**. 12. **Trace_id preserved**. 13. **UNIQUE(payslip_id)**. 14. **Append-only via REVOKE except status cols**. 15. **Replay produces byte-identical**. 16. **Template versioned**. 17. **Render perf < 5s per payslip**. 18. **Bulk render parallel**. 19. **Render failure → status=failed + sev-2**. 20. **Multilingual support (vi/en)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn byte_identical_replay() {
    let ctx = TestContext::with_payslip().await;
    let r1 = ctx.render_pdf(ctx.payslip_id).await;
    let r2 = ctx.render_pdf(ctx.payslip_id).await;
    assert_eq!(r1.sha256, r2.sha256);
}

#[tokio::test]
async fn verification_catches_drift() {
    let ctx = TestContext::with_rendered_pdf().await;
    ctx.simulate_pdf_corruption(ctx.pdf_id).await;
    let r = ctx.verify(ctx.pdf_id).await;
    assert_eq!(r.status, "verification_failed");
}

#[tokio::test]
async fn pinned_fonts_in_template() {
    let template = std::fs::read_to_string("templates/payslip.tex").unwrap();
    assert!(template.contains("\\usepackage{fontspec}"));
    assert!(template.contains("\\setmainfont{"));  // pinned font
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-REW-005.
**Cross-module:** TASK-DOC-001 (storage), TASK-AUTH-101 (CFO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Tectonic fail | catch | status=failed; sev-2 | retry |
| Font missing | tex error | status=failed; sev-1 | install font |
| SHA256 mismatch | verifier | status=verification_failed; sev-1 | investigate |
| TASK-DOC-001 store fail | catch | sev-2 | retry |
| Cross-tenant render | RLS | 0 rows | inherent |
| Large bulk render | parallelize | inherent | inherent |
| Template error | TeX validation | sev-1 | fix template |
| Locale not supported | fallback to en | sev-3 | add locale |
| Concurrent render | UNIQUE on payslip | second skipped | inherent |
| Disk fill | catch | sev-2 | cleanup |

## §11 — Implementation notes
- §11.1 Tectonic Rust crate `tectonic = "0.15"`; compiles deterministically when fonts + packages pinned.
- §11.2 Fonts: pin to system path or bundle in image.
- §11.3 memory audit body: payslip_id, sha256, status; content never in chain.
- §11.4 Bulk render: tokio parallel with bounded concurrency (4 at a time).
- §11.5 Member portal access via TASK-PORTAL-001 scoped view.

---

*End of TASK-REW-006 spec.*
