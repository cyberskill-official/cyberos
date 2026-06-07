---
id: FR-EMAIL-011
title: "EMAIL DSAR message export — every message a subject authored or received + chained memory audit hashes for FR-PORTAL-008 bundle"
module: EMAIL
priority: MUST
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-EMAIL-001, FR-EMAIL-009, FR-PORTAL-008, FR-AUTH-101, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-EMAIL-001]
blocks: []

source_pages:
  - website/docs/modules/email.html#dsar

source_decisions:
  - DEC-1500 2026-05-17 — DSAR export = subject_id-scoped retrieval of every message the subject authored OR received OR was cc/bcc'd; consumed by FR-PORTAL-008 DSAR bundle
  - DEC-1501 2026-05-17 — Output format: JSONL stream (one message per line); attachments separate S3-key references; mailbox structure preserved
  - DEC-1502 2026-05-17 — memory audit chain hashes included per message — receiver can verify chain inclusion years later
  - DEC-1503 2026-05-17 — Cross-tenant scoping: subject's messages from OTHER tenants NOT included (each tenant's DSAR is per-tenant)
  - DEC-1504 2026-05-17 — Async via FR-MCP-007 Tasks (large mailboxes ~100MB+ take minutes)
  - DEC-1505 2026-05-17 — memory audit kinds: email.dsar_export_started, email.dsar_export_completed, email.dsar_export_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/email/
  new_files:
    - services/email/migrations/0007_dsar_export_jobs.sql
    - services/email/src/dsar/mod.rs
    - services/email/src/dsar/aggregator.rs
    - services/email/src/dsar/jsonl_writer.rs
    - services/email/src/dsar/chain_anchor.rs
    - services/email/src/audit/dsar_events.rs
    - services/email/src/handlers/dsar_routes.rs
    - services/email/tests/dsar_authored_messages_test.rs
    - services/email/tests/dsar_received_messages_test.rs
    - services/email/tests/dsar_cross_tenant_excluded_test.rs
    - services/email/tests/dsar_attachments_referenced_test.rs
    - services/email/tests/dsar_chain_hashes_test.rs
    - services/email/tests/dsar_async_via_tasks_test.rs
    - services/email/tests/dsar_audit_emission_test.rs

  modified_files:
    - services/email/src/lib.rs

  allowed_tools:
    - file_read: services/{email,portal}/**
    - file_write: services/email/{src,tests,migrations}/**
    - bash: cd services/email && cargo test dsar

  disallowed_tools:
    - include cross-tenant messages (per DEC-1503)
    - inline attachment bytes (per DEC-1501 — reference only)
    - skip chain anchors (per DEC-1502)

effort_hours: 5
sub_tasks:
  - "0.3h: 0007_dsar_export_jobs.sql"
  - "0.3h: dsar/mod.rs"
  - "0.5h: aggregator.rs (mailbox + sent + cc/bcc query)"
  - "0.5h: jsonl_writer.rs"
  - "0.4h: chain_anchor.rs (memory row hash lookup per message)"
  - "0.3h: audit/dsar_events.rs"
  - "0.3h: handlers/dsar_routes.rs"
  - "1.4h: tests — 7 test files"
  - "1.0h: FR-PORTAL-008 integration smoke"

risk_if_skipped: "Without EMAIL DSAR export, FR-PORTAL-008 bundle is incomplete — GDPR Art. 15 obligation unmet. Without DEC-1502 chain anchors, recipient cannot verify message authenticity years later. Without DEC-1503 cross-tenant exclusion, one DSAR leaks other tenants' data."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship DSAR message export at `services/email/src/dsar/` returning subject-scoped JSONL of every authored + received + cc'd message with attachment S3 refs + per-message memory chain anchor, async via FR-MCP-007, 3 memory audit kinds.

1. **MUST** expose `POST /v1/email/dsar/export` body `{ subject_id }`. Caller is FR-PORTAL-008 task (system-tenant). Enqueues FR-MCP-007 task per DEC-1504; returns task_id.

2. **MUST** aggregate via `aggregator.rs::aggregate(tenant_id, subject_id)`:
   - SELECT messages WHERE author_subject_id=$subject AND tenant_id=$tenant.
   - UNION SELECT WHERE recipient or cc or bcc matches subject's email_addresses.
   - Per DEC-1503: cross-tenant NEVER included.

3. **MUST** write JSONL per DEC-1501 — one message per line, structure: `{ id, from, to, cc, subject, body_text, body_html, sent_at, attachments: [{filename, s3_key, sha256, size}], memory_audit_chain_hash }`.

4. **MUST** include attachment S3 references per DEC-1501 — never inline bytes (size + bundle bloat).

5. **MUST** include memory chain anchor per DEC-1502 via `chain_anchor.rs::lookup(message_id)` — finds the `email.send_queued` or `email.message_received` memory row for the message; embeds chain hash.

6. **MUST** define `dsar_export_jobs` table at migration `0007`: `(job_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL, status TEXT NOT NULL DEFAULT 'pending', message_count INT, attachment_count INT, output_s3_key TEXT, started_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, trace_id CHAR(32))`. Append-only.

7. **MUST** stream output to S3 (FR-DOC-001 path); never load entire mailbox into memory.

8. **MUST** emit 3 memory audit kinds per DEC-1505.

9. **MUST** thread trace_id from FR-PORTAL-008 task through aggregator + writer.

10. **MUST NOT** include cross-tenant per DEC-1503.

11. **MUST NOT** inline attachment bytes per DEC-1501.

---

## §2 — Why this design

**Why JSONL + S3 refs (DEC-1501)?** Streaming-friendly format; large mailboxes don't OOM. S3 refs preserve attachments without bundle bloat.

**Why chain anchor per message (DEC-1502)?** Tamper-evident: recipient can prove message wasn't altered post-export.

**Why async (DEC-1504)?** Large mailbox aggregation is minutes; sync would timeout.

---

## §3 — API contract

```sql
CREATE TABLE dsar_export_jobs (
  job_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','running','completed','failed')),
  message_count INT,
  attachment_count INT,
  output_s3_key TEXT,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  failure_reason TEXT,
  trace_id CHAR(32)
);
ALTER TABLE dsar_export_jobs ENABLE ROW LEVEL SECURITY;
CREATE POLICY dsar_jobs_rls ON dsar_export_jobs
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON dsar_export_jobs FROM cyberos_app;
GRANT UPDATE (status, message_count, attachment_count, output_s3_key,
              started_at, completed_at, failure_reason) ON dsar_export_jobs TO cyberos_app;
```

Endpoints:
```text
POST   /v1/email/dsar/export          (system-tenant via FR-PORTAL-008)
GET    /v1/email/dsar/jobs/{id}       (poll status)
```

---

## §4 — Acceptance criteria
1. **All authored messages included**. 2. **All received messages included**. 3. **CC/BCC matches included**. 4. **Cross-tenant excluded**. 5. **Attachments by S3 ref only**. 6. **Chain anchor present per message**. 7. **JSONL format valid**. 8. **Async via FR-MCP-007**. 9. **Status poll returns progress**. 10. **3 memory audit kinds emitted**. 11. **Trace_id from FR-PORTAL-008 preserved**. 12. **Large mailbox (10k msgs) completes < 30min**. 13. **PII not in audit chain** (only message counts). 14. **RLS denies non-system caller**. 15. **Job idempotency** — duplicate request returns existing job_id. 16. **Stream to S3 (no OOM)**. 17. **Chain anchor missing → noted in output**. 18. **Per-message size cap respected**. 19. **Output S3 key persistent**. 20. **Failure path emits sev-2**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dsar_exports_authored_and_received() {
    let ctx = TestContext::with_subject_messages(100, 50).await;  // 100 authored, 50 received
    let job = ctx.start_dsar_export(ctx.subject_id).await;
    ctx.wait_completion(job).await;
    let s3_key: String = ctx.get_job_output(job).await;
    let lines = ctx.read_s3_jsonl(&s3_key).await;
    assert_eq!(lines.len(), 150);
}

#[tokio::test]
async fn cross_tenant_excluded() {
    let ctx = TestContext::with_subject_in_two_tenants().await;
    ctx.send_message_in_tenant(ctx.tenant_a, ctx.subject_id, "in-a").await;
    ctx.send_message_in_tenant(ctx.tenant_b, ctx.subject_id, "in-b").await;
    let job = ctx.start_dsar_export_for(ctx.tenant_a, ctx.subject_id).await;
    let lines = ctx.read_s3_jsonl_from_job(job).await;
    assert!(lines.iter().any(|m| m.contains("in-a")));
    assert!(!lines.iter().any(|m| m.contains("in-b")));
}

#[tokio::test]
async fn chain_anchor_per_message() {
    let ctx = TestContext::with_subject_messages(5, 0).await;
    let job = ctx.complete_dsar().await;
    let lines = ctx.read_s3_jsonl_from_job(job).await;
    for line in lines {
        let msg: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert!(msg["memory_audit_chain_hash"].is_string());
    }
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-EMAIL-001.
**Cross-module:** FR-PORTAL-008 (caller), FR-MCP-007 (async task), FR-DOC-001 (S3), FR-AI-003, FR-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Mailbox empty | aggregator returns 0 | Empty JSONL with header; job completed | Inherent |
| Attachment S3 missing | reference still emitted | Recipient sees broken link | Per-attachment audit |
| Chain anchor missing | lookup miss | Note in output; sev-2 audit | Investigate audit log |
| Subject_id wrong | RLS | Empty result | Caller validates |
| Task timeout | FR-MCP-007 30min | Status=failed; retry once | Inherent |
| Subject has > 100k messages | size cap | Paginated S3 outputs (slice 3) | Slice-2 = caps at 100k |
| Cross-tenant attempt | tenant_id check | Excluded | Inherent |
| Concurrent export same subject | idempotency | Returns existing job | Inherent |
| S3 upload fail | retry | Sev-2 | S3 recovery |
| KMS for attachment metadata | error | Skipped + noted | Inherent |
| Failure mid-stream | partial output | Marked failed | Re-run |
| Output > 5 GiB | S3 multipart | Inherent | None |

## §11 — Implementation notes
- §11.1 JSONL streaming via tokio AsyncWriter to S3 multipart upload.
- §11.2 Chain anchor lookup batched (in chunks of 1000 messages) for performance.
- §11.3 Aggregator uses SELECT with UNION + tenant + subject filters; indexed by (tenant_id, author_subject_id) and (tenant_id, recipient_subject_id).
- §11.4 Per-message size cap 25 MiB (consistent with attachment cap).
- §11.5 PII: message bodies not in memory chain; only counts.

---

*End of FR-EMAIL-011 spec.*
