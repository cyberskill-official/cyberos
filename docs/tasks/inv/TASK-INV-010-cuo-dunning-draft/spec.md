---
id: TASK-INV-010
title: "INV CUO dunning draft — auto-generate polite/firm/legal-warning email drafts per aging bucket + CFO review queue + send-via-TASK-EMAIL-009"
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
module: inv
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
related_tasks: [TASK-INV-009, TASK-EMAIL-009, TASK-CUO-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-INV-009, TASK-EMAIL-009, TASK-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/inv.html#dunning

source_decisions:
  - DEC-1550 2026-05-17 — Dunning tone scaled to bucket: overdue_30=polite, overdue_60=firm, overdue_90=urgent, overdue_120/120plus=legal-warning + escalate to CFO
  - DEC-1551 2026-05-17 — Closed enum `dunning_tone` = {polite, firm, urgent, legal_warning}; cardinality 4
  - DEC-1552 2026-05-17 — Draft generation via TASK-AI-003 with CFO-defined template per tenant (default templates ship); tenant-customizable
  - DEC-1553 2026-05-17 — Drafts NEVER auto-send — always CFO approval required (legal-warning + emotional risk)
  - DEC-1554 2026-05-17 — Daily scan triggered at 09:00 tenant_timezone; generates drafts for new transitions only (idempotent: skip if draft exists for same invoice+bucket)
  - DEC-1555 2026-05-17 — memory audit kinds: inv.dunning_draft_generated, inv.dunning_draft_approved, inv.dunning_draft_dismissed, inv.dunning_email_sent

language: rust 1.81
service: cyberos/services/invoicing/
new_files:
  - services/invoicing/migrations/0009_dunning_drafts.sql
  - services/invoicing/src/dunning/mod.rs
  - services/invoicing/src/dunning/scanner.rs
  - services/invoicing/src/dunning/draft_generator.rs
  - services/invoicing/src/dunning/template_loader.rs
  - services/invoicing/src/handlers/dunning_routes.rs
  - services/invoicing/src/audit/dunning_events.rs
  - services/invoicing/templates/dunning_polite.md
  - services/invoicing/templates/dunning_firm.md
  - services/invoicing/templates/dunning_urgent.md
  - services/invoicing/templates/dunning_legal_warning.md
  - services/invoicing/tests/dunning_bucket_to_tone_test.rs
  - services/invoicing/tests/dunning_no_auto_send_test.rs
  - services/invoicing/tests/dunning_idempotent_scan_test.rs
  - services/invoicing/tests/dunning_tone_enum_cardinality_test.rs
  - services/invoicing/tests/dunning_template_render_test.rs
  - services/invoicing/tests/dunning_audit_emission_test.rs

modified_files:
  - services/invoicing/src/lib.rs

allowed_tools:
  - file_read: services/invoicing/**
  - file_write: services/invoicing/{src,tests,migrations,templates}/**
  - bash: cd services/invoicing && cargo test dunning

disallowed_tools:
  - auto-send (per DEC-1553)
  - duplicate draft for same invoice+bucket (per DEC-1554)
  - send legal-warning without CFO sign-off (per DEC-1553)

effort_hours: 5
subtasks:
  - "0.3h: 0009_dunning_drafts.sql"
  - "0.3h: dunning/mod.rs"
  - "0.5h: scanner.rs (daily scan + dedup)"
  - "0.7h: draft_generator.rs (TASK-AI-003 integration)"
  - "0.4h: template_loader.rs"
  - "0.4h: handlers/dunning_routes.rs"
  - "0.3h: audit/dunning_events.rs"
  - "0.4h: 4 template markdown files"
  - "1.4h: tests — 6 test files"
  - "0.3h: cron registration"

risk_if_skipped: "Without auto dunning drafts, CFO writes each manually — collection workflow slows + late-stage AR unrecovered. Without DEC-1553 manual approval, accidentally sent legal-warning emails damage customer relationships. Without DEC-1554 idempotency, daily scan floods queue with duplicates."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship dunning draft generation at `services/invoicing/src/dunning/` triggered daily, tone-scaled to aging bucket, drafts queued for CFO approval (never auto-send), send via TASK-EMAIL-009, 4 memory audit kinds.

1. **MUST** schedule daily scanner at 09:00 tenant_timezone per DEC-1554 via TASK-MCP-007 task or cron. `scanner.rs::scan(tenant)` calls TASK-INV-009 `aging.generate({as_of_date: today, group_by: 'engagement'})`.

2. **MUST** map bucket → tone per DEC-1550:
- overdue_30 → polite
- overdue_60 → firm
- overdue_90 → urgent
- overdue_120 + overdue_120plus → legal_warning

3. **MUST** validate `dunning_tone` against closed enum per DEC-1551; reject invalid values.

4. **MUST** generate draft via `draft_generator.rs::generate(invoice, tone, template)` — TASK-AI-003 call with template + invoice context. Templates loaded from `template_loader.rs::load(tenant, tone)`.

5. **MUST** be idempotent per DEC-1554: skip if `dunning_drafts` row exists for `(invoice_id, tone)`. Use `UNIQUE(tenant_id, invoice_id, tone)`.

6. **MUST** queue draft for CFO review — NEVER auto-send per DEC-1553. CFO sees in dunning queue UI (TASK-CUO-101); approves or dismisses.

7. **MUST** on CFO approval: call TASK-EMAIL-009 send_message with draft body, log `dunning_email_sent` audit.

8. **MUST** define `dunning_drafts` table at migration `0009`:
   ```sql
   CREATE TABLE dunning_drafts (
     draft_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     invoice_id UUID NOT NULL,
     engagement_id UUID NOT NULL,
     customer_id UUID NOT NULL,
     aging_bucket TEXT NOT NULL,
     tone TEXT NOT NULL CHECK (tone IN ('polite','firm','urgent','legal_warning')),
     draft_subject TEXT NOT NULL,
     draft_body TEXT NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending_review'
       CHECK (status IN ('pending_review','approved','dismissed','sent','failed_send')),
     reviewed_by UUID,
     reviewed_at TIMESTAMPTZ,
     email_message_id UUID,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE UNIQUE INDEX dunning_dedup_idx ON dunning_drafts(tenant_id, invoice_id, tone);
   ALTER TABLE dunning_drafts ENABLE ROW LEVEL SECURITY;
   CREATE POLICY dunning_drafts_rls ON dunning_drafts
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON dunning_drafts FROM cyberos_app;
   GRANT UPDATE (status, reviewed_by, reviewed_at, email_message_id) ON dunning_drafts TO cyberos_app;
   ```

9. **MUST** emit 4 memory audit kinds per DEC-1555. PII: draft_body customer-name/amounts scrubbed via TASK-MEMORY-111 SHA256 hash.

10. **MUST** thread trace_id from cron/scanner → generator → CFO review → email send.

11. **MUST NOT** auto-send any draft per DEC-1553 — UI shows approve/dismiss only.

12. **MUST NOT** generate legal_warning draft without escalation flag — surfaces in CFO inbox with red banner.

---

## §2 — Why this design

**Why tone scales by bucket (DEC-1550)?** Industry-standard collection escalation; reduces churn vs all-firm approach.

**Why manual approval always (DEC-1553)?** Legal-warning emails carry liability + customer relationship risk; auto-send is unacceptable error mode.

**Why idempotent scan (DEC-1554)?** Daily scan must not duplicate drafts; CFO sees clean queue.

**Why TASK-AI-003 generation (DEC-1552)?** Personalization (customer name, invoice ref, outstanding balance) requires templated AI inference; static templates feel robotic.

---

## §3 — API contract

```text
GET    /v1/inv/dunning/drafts             (list pending review, CFO)
POST   /v1/inv/dunning/drafts/{id}/approve  (CFO sends)
POST   /v1/inv/dunning/drafts/{id}/dismiss  (CFO rejects)
POST   /v1/inv/dunning/scan               (CFO manual trigger)
```

Sample draft response:
```json
{
  "draft_id": "uuid",
  "invoice_id": "uuid",
  "customer_id": "uuid",
  "aging_bucket": "overdue_60",
  "tone": "firm",
  "draft_subject": "Reminder: Invoice INV-2026-042 — 45 days past due",
  "draft_body": "Dear {customer_name},\n\nThis is a follow-up regarding...",
  "status": "pending_review",
  "created_at": "2026-05-17T09:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **Daily scan at 09:00 tenant_timezone**. 2. **Bucket→tone mapping correct**. 3. **Closed enum 4 values + cardinality test**. 4. **Idempotent (UNIQUE on tenant+invoice+tone)**. 5. **TASK-AI-003 generates draft**. 6. **Drafts queued for CFO review (never auto-send)**. 7. **Approve → TASK-EMAIL-009 send**. 8. **Dismiss → status=dismissed (not deleted)**. 9. **4 memory audit kinds emitted**. 10. **PII scrubbed (customer/amount → SHA256)**. 11. **RLS denies cross-tenant**. 12. **Legal_warning has red-banner UX flag**. 13. **Trace_id preserved**. 14. **Template customization per tenant**. 15. **Manual scan trigger CFO-only**. 16. **Sent draft status=sent + email_message_id linked**. 17. **Failed send → status=failed_send + retry**. 18. **Append-only (REVOKE UPDATE except status/review)**. 19. **Aging bucket re-classified on re-scan (e.g. 60→90)**. 20. **Multiple invoices same customer → one draft each (per invoice, not aggregate)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn bucket_to_tone_mapping() {
    assert_eq!(map_tone("overdue_30"), Tone::Polite);
    assert_eq!(map_tone("overdue_60"), Tone::Firm);
    assert_eq!(map_tone("overdue_90"), Tone::Urgent);
    assert_eq!(map_tone("overdue_120"), Tone::LegalWarning);
    assert_eq!(map_tone("overdue_120plus"), Tone::LegalWarning);
}

#[tokio::test]
async fn never_auto_sends() {
    let ctx = TestContext::with_overdue_invoices().await;
    ctx.run_daily_scan().await;
    let sent_emails = ctx.email_send_count().await;
    assert_eq!(sent_emails, 0);
    let drafts = ctx.fetch_drafts().await;
    assert!(drafts.iter().all(|d| d.status == "pending_review"));
}

#[tokio::test]
async fn idempotent_scan_no_duplicates() {
    let ctx = TestContext::with_overdue_invoices().await;
    ctx.run_daily_scan().await;
    let count1 = ctx.draft_count().await;
    ctx.run_daily_scan().await;
    let count2 = ctx.draft_count().await;
    assert_eq!(count1, count2);
}

// 5.4..5.10 — template render, audit, RLS, AI integration, send flow
```

---

## §6 — Skeleton

```rust
pub async fn scan(tenant: &Tenant, db: &Db) -> Result<ScanResult> {
    let report = aging::generate(AgingRequest{
        as_of_date: today_in(tenant.timezone),
        group_by: Group::Engagement, base_currency: None
    }, db).await?;
    let mut created = 0;
    for bucket_row in report.buckets {
        for invoice in bucket_row.invoices {
            let tone = map_tone(&invoice.bucket);
            if db.draft_exists(tenant.id, invoice.id, tone).await? { continue; }
            let template = template_loader::load(tenant, tone).await?;
            let body = draft_generator::generate(&invoice, tone, &template).await?;
            db.insert_draft(invoice, tone, body).await?;
            audit::emit("inv.dunning_draft_generated", json!({...}), trace).await?;
            created += 1;
        }
    }
    Ok(ScanResult{drafts_created: created})
}
```

---

## §7 — Dependencies
**Upstream:** TASK-INV-009 (aging), TASK-EMAIL-009 (send). **Cross-module:** TASK-AI-003 (template generation), TASK-CUO-101 (review UI), TASK-MCP-007 (cron).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking — CFO can iterate templates after launch.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| TASK-AI-003 timeout | generation fail | retry 3x then sev-2 | manual retry |
| Template missing | load fail | use default + sev-2 audit | CFO uploads |
| Customer has no email | scanner check | skip + audit warning | data fix |
| Duplicate scan run | UNIQUE constraint | skip | inherent |
| CFO double-approves | UPDATE WHERE pending | only first wins | inherent |
| Send fail (bounce) | TASK-EMAIL-009 status | status=failed_send + retry | CFO investigates |
| Aging changes mid-scan | snapshot semantics | uses scanner-time snapshot | next scan picks up |
| Customer paid mid-scan | aging excludes paid | invoice removed | inherent |
| Cron skipped (system down) | last_run check | catch-up on next boot | inherent |
| Legal-warning escalation missed | red-banner UX | CFO see immediately | UX gate |

## §11 — Implementation notes
- §11.1 Templates use `{customer_name} {invoice_ref} {outstanding_balance} {days_overdue}` placeholders.
- §11.2 TASK-AI-003 prompt: "Write a {tone} payment reminder using this template..." — model fills placeholders + softens/firms tone.
- §11.3 memory audit body: customer_id (uuid OK), tone, bucket; draft_body SHA256 hashed.
- §11.4 Cron via TASK-MCP-007 with `kind: 'inv.dunning_daily_scan'`, tenant_id arg.
- §11.5 Legal warning template includes "this is not legal advice" disclaimer + link to legal team contact.

---

*End of TASK-INV-010 spec.*
