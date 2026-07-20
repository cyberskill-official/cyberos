---
id: TASK-EMAIL-007
title: "EMAIL convert-to-issue — one-click create task-PROJ issue from message with thread backlink + attachment carry-over + AI summary"
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
module: EMAIL
priority: p1
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-EMAIL-001, TASK-PROJ-001, TASK-AI-003, TASK-DOC-001, TASK-MEMORY-111]
depends_on: [TASK-EMAIL-001, TASK-PROJ-001]
blocks: []

source_pages:
  - website/docs/modules/email.html#convert-to-issue

source_decisions:
  - DEC-1580 2026-05-17 — Convert action: copy message+thread context to new issue; bi-directional backlink (issue.email_thread_id ↔ message.linked_issue_id)
  - DEC-1581 2026-05-17 — AI summary via TASK-AI-003: "title: <action-oriented sentence>; description: <3-paragraph context>; priority: low|med|high"
  - DEC-1582 2026-05-17 — Closed enum `convert_source` = {email_single_message, email_full_thread, email_inline_quote}; cardinality 3
  - DEC-1583 2026-05-17 — Attachments carry over via TASK-DOC-001 references (no copy — same S3 key, new doc_links row)
  - DEC-1584 2026-05-17 — Project selection: user picks at convert; default to last-used-project per user
  - DEC-1585 2026-05-17 — memory audit kinds: email.convert_to_issue_initiated, email.convert_to_issue_completed, email.convert_to_issue_failed

language: rust 1.81
service: cyberos/services/email/
new_files:
  - services/email/src/convert/mod.rs
  - services/email/src/convert/issue_builder.rs
  - services/email/src/convert/ai_summarizer.rs
  - services/email/src/handlers/convert_routes.rs
  - services/email/src/audit/convert_events.rs
  - services/email/migrations/0009_message_issue_link.sql
  - services/email/tests/convert_single_message_test.rs
  - services/email/tests/convert_full_thread_test.rs
  - services/email/tests/convert_attachments_test.rs
  - services/email/tests/convert_backlink_test.rs
  - services/email/tests/convert_source_enum_cardinality_test.rs
  - services/email/tests/convert_audit_emission_test.rs

modified_files:
  - services/email/src/lib.rs

allowed_tools:
  - file_read: services/{email,proj,doc}/**
  - file_write: services/email/{src,tests,migrations}/**
  - bash: cd services/email && cargo test convert

disallowed_tools:
  - copy attachment bytes (per DEC-1583 — reference only)
  - convert without project_id (per DEC-1584 — must select)

effort_hours: 6
subtasks:
  - "0.3h: 0009_message_issue_link.sql"
  - "0.3h: convert/mod.rs"
  - "0.6h: issue_builder.rs"
  - "0.5h: ai_summarizer.rs (TASK-AI-003 integration)"
  - "0.4h: handlers/convert_routes.rs"
  - "0.3h: audit/convert_events.rs"
  - "2.0h: tests — 6 test files"
  - "1.6h: Missive UI convert button + project picker"

risk_if_skipped: "Without convert-to-issue, support requests stay in email + lost in inbox — project tracking broken. Without DEC-1580 backlink, can't trace issue origin or reply via email. Without DEC-1583 attachment references, S3 duplication wastes storage."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship convert-to-issue at `services/email/src/convert/` creating task-PROJ issue from message/thread, attachment refs, AI summary, bi-directional backlink, 3 memory audit kinds.

1. **MUST** expose `POST /v1/email/messages/{id}/convert-to-issue` body `{ project_id, convert_source, title_override?, priority_override? }`.

2. **MUST** validate `convert_source` against closed enum per DEC-1582 — `email_single_message` (this msg), `email_full_thread` (whole thread), `email_inline_quote` (quoted-text section).

3. **MUST** call AI summarizer per DEC-1581: `ai_summarizer.rs::summarize(message, thread, source)` → `{title, description, priority}`.

4. **MUST** build issue via `issue_builder.rs::build` calling TASK-PROJ-001 create endpoint with:
- title (AI summary or override)
- description (AI summary + linked thread context)
- priority (AI or override)
- source_message_id (backlink)
- source_thread_id (thread backlink)
- project_id

5. **MUST** carry attachments per DEC-1583 — query message attachments → create TASK-DOC-001 doc_links rows (same S3 key, new linked_to=issue_id). No file copy.

6. **MUST** define link table at migration `0009`:
   ```sql
   ALTER TABLE messages ADD COLUMN linked_issue_id UUID;
   ALTER TABLE messages ADD COLUMN converted_at TIMESTAMPTZ;
   ALTER TABLE messages ADD COLUMN converted_by UUID;
   CREATE INDEX messages_linked_issue_idx ON messages(tenant_id, linked_issue_id)
     WHERE linked_issue_id IS NOT NULL;
   GRANT UPDATE (linked_issue_id, converted_at, converted_by) ON messages TO cyberos_app;
   ```

7. **MUST** set bi-directional backlink per DEC-1580:
- `message.linked_issue_id = issue.id`
- issue created with `source_thread_id = thread.id` (TASK-PROJ-001 column)

8. **MUST** allow project selection per DEC-1584; default = `user.last_used_project_id` (TASK-AUTH-101 user prefs).

9. **MUST** emit 3 memory audit kinds per DEC-1585. PII per TASK-MEMORY-111: message body/subject SHA-256 hashed; ids ok.

10. **MUST** thread trace_id from convert action → AI → issue creation → audit.

11. **MUST NOT** copy attachment bytes per DEC-1583 — reference S3 key.

12. **MUST NOT** convert without project_id per DEC-1584.

---

## §2 — Why this design

**Why bi-directional backlink (DEC-1580)?** Engineers need to reply to original sender; PMs need to see issue origin.

**Why AI summary (DEC-1581)?** Raw email subject is rarely a good issue title; AI extracts action.

**Why 3 source modes (DEC-1582)?** Single (one message), full thread (context), inline quote (cherry-picked text); covers UX patterns.

**Why attachment references (DEC-1583)?** S3 duplication wastes cost + risks divergence; references are canonical.

---

## §3 — API contract

```text
POST   /v1/email/messages/{id}/convert-to-issue
GET    /v1/email/messages/{id}/converted-issues   (list — message may have been converted multiple times)
```

Sample request:
```json
{
  "project_id": "uuid",
  "convert_source": "email_full_thread",
  "title_override": null,
  "priority_override": "high"
}
```

Sample response:
```json
{
  "issue_id": "uuid",
  "issue_url": "/proj/issues/abc-123",
  "ai_summary": {
    "title": "Investigate API timeout for Acme webhook endpoint",
    "description": "Customer reports webhook calls timing out after 30s...",
    "priority": "high"
  },
  "attachments_linked": 3,
  "backlink_created": true
}
```

---

## §4 — Acceptance criteria
1. **POST creates issue + backlinks message**. 2. **3 source modes work distinct**. 3. **Closed enum + cardinality test**. 4. **AI summary returns title+desc+priority**. 5. **Override fields respected**. 6. **Attachments referenced (not copied)**. 7. **doc_links rows created for each attachment**. 8. **project_id required (400 if missing)**. 9. **Default project = user last_used**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (body/subject SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Bi-directional backlink (msg.linked_issue_id + issue.source_thread_id)**. 15. **Multiple conversions allowed (same msg → multiple issues)**. 16. **GET endpoint lists all conversions**. 17. **AI failure → fallback to subject as title + sev-2 audit**. 18. **converted_by audit-traceable**. 19. **Append-only (REVOKE UPDATE except link cols)**. 20. **Issue created in user's chosen project (RLS-respected)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn convert_single_message_creates_issue() {
    let ctx = TestContext::with_message_and_project().await;
    let resp = ctx.convert(ctx.message_id, ctx.project_id, "email_single_message").await;
    assert!(resp.issue_id.is_some());
    let m: Message = ctx.fetch_message(ctx.message_id).await;
    assert_eq!(m.linked_issue_id, Some(resp.issue_id.unwrap()));
}

#[tokio::test]
async fn full_thread_includes_all_messages_in_description() {
    let ctx = TestContext::with_thread_of_5().await;
    let resp = ctx.convert(ctx.last_msg, ctx.project_id, "email_full_thread").await;
    let issue: Issue = ctx.fetch_issue(resp.issue_id.unwrap()).await;
    for sender in &["alice", "bob", "carol", "dave", "eve"] {
        assert!(issue.description.contains(sender));
    }
}

#[tokio::test]
async fn attachments_referenced_not_copied() {
    let ctx = TestContext::with_message_with_3_attachments().await;
    let resp = ctx.convert(ctx.message_id, ctx.project_id, "email_single_message").await;
    let links = ctx.fetch_doc_links(resp.issue_id.unwrap()).await;
    assert_eq!(links.len(), 3);
    let orig_keys = ctx.message_attachment_s3_keys(ctx.message_id).await;
    let new_keys: Vec<_> = links.iter().map(|l| &l.s3_key).collect();
    assert_eq!(orig_keys, new_keys);  // same keys, no copy
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-EMAIL-001, TASK-PROJ-001. **Cross-module:** TASK-AI-003 (summary), TASK-DOC-001 (attachment refs), TASK-AUTH-101 (user prefs), TASK-MEMORY-111 (PII).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| TASK-AI-003 timeout | retry 1x | fallback subject + sev-2 audit | inherent |
| project_id missing/invalid | validate | 400 / 404 | provide valid |
| message already converted to issue | check | 200 with existing or 409 (configurable) | inherent |
| Attachment S3 missing | per-attachment | link still created (broken-link note) | data fix |
| TASK-PROJ-001 create fails | downstream error | rollback message update | inherent |
| User no project access | RLS | 403 | request access |
| AI returns invalid priority | enum match | default to "med" | inherent |
| Thread has >100 messages | truncate desc | last 100 included + sev-3 note | inherent |
| Concurrent convert same msg | UNIQUE | second wins or both create depending on config | inherent |
| Inline quote selection fails | parser err | fallback single-message | inherent |

## §11 — Implementation notes
- §11.1 AI prompt: "Convert this email thread into a project issue. Output JSON {title, description, priority}."
- §11.2 Description format: AI summary + `\n\n---\nOriginal thread: [link]` + thread excerpt.
- §11.3 doc_links carries `linked_to_kind: 'issue'`, `linked_to_id: issue_id`.
- §11.4 PII: message body/subject hashed in memory; ids ok.
- §11.5 last_used_project_id stored on user prefs, updated each convert.

---

*End of TASK-EMAIL-007 spec.*
