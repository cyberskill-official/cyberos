---
id: TASK-PORTAL-006
title: "PORTAL client-initiated workflows — new project request / billing inquiry / support ticket → CHAT thread with SLA + auto-routing + status tracking"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: PORTAL
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PORTAL-001, TASK-PORTAL-003, TASK-PORTAL-005, TASK-CHAT-005, TASK-PROJ-001, TASK-INV-001, TASK-AUTH-101, TASK-EMAIL-001, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007]
depends_on: [TASK-CHAT-005]
blocks: []

source_pages:
  - website/docs/modules/portal.html#workflows

source_decisions:
  - DEC-1240 2026-05-17 — Closed enum `client_workflow_kind` = {new_project_request, billing_inquiry, support_ticket, task, general_question}; CI cardinality asserts 5
  - DEC-1241 2026-05-17 — Each submission auto-creates a CHAT thread in the appropriate Engagement-scoped channel (per workflow_kind → channel mapping)
  - DEC-1242 2026-05-17 — Auto-routing: routing rules per (tenant_id, workflow_kind) → assignee subject_id; defaults to engagement_admin if no rule matches
  - DEC-1243 2026-05-17 — Closed enum `workflow_status` = {submitted, acknowledged, in_progress, awaiting_client, resolved, closed, escalated}; CI cardinality asserts 7
  - DEC-1244 2026-05-17 — SLA defaults: acknowledged ≤ 4 business hours; resolved ≤ 5 business days (overridable per (tenant, workflow_kind))
  - DEC-1245 2026-05-17 — Submitter sees status only — never sees internal CHAT messages; replies via portal UI flow back to the CHAT thread as `client_reply` type messages
  - DEC-1246 2026-05-17 — File attachments on submission via TASK-DOC-001 presigned S3 URL (max 25 MiB per file, 5 files per workflow)
  - DEC-1247 2026-05-17 — Auto-prioritisation: support_ticket with keywords {down, outage, breach, urgent, security} → escalated status + sev-1 alert to engagement_admin
  - DEC-1248 2026-05-17 — memory audit kinds: portal.workflow_submitted, portal.workflow_routed, portal.workflow_status_changed, portal.workflow_resolved, portal.workflow_sla_breach, portal.workflow_client_reply
  - DEC-1249 2026-05-17 — Rate limit 10 submissions per workflow_kind per hour per caller; over-limit → 429
  - DEC-1250 2026-05-17 — Per-workflow conversation persisted in CHAT (TASK-CHAT-005); workflow row is the metadata + routing record + status mirror
  - DEC-1251 2026-05-17 — Reopening closed workflow within 30 days returns workflow to `awaiting_client`; beyond 30 days requires new submission
  - DEC-1252 2026-05-17 — Email notification to submitter on every status change via TASK-EMAIL-001 (per-tenant template overrides per TASK-PORTAL-002)
  - DEC-1253 2026-05-17 — Internal workflow rules: tenant_admin configures via tenant config (slice 3 UI); slice 2 = YAML config

build_envelope:
  language: rust 1.81
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0018_portal_workflow_submissions.sql
    - services/portal/migrations/0019_portal_workflow_routing_rules.sql
    - services/portal/src/workflows/mod.rs
    - services/portal/src/workflows/submit.rs
    - services/portal/src/workflows/router.rs
    - services/portal/src/workflows/status_machine.rs
    - services/portal/src/workflows/sla_monitor.rs
    - services/portal/src/workflows/attachments.rs
    - services/portal/src/workflows/auto_priority.rs
    - services/portal/src/workflows/chat_bridge.rs
    - services/portal/src/audit/workflow_events.rs
    - services/portal/src/handlers/workflow_routes.rs
    - services/portal/tests/workflow_submit_test.rs
    - services/portal/tests/workflow_routing_test.rs
    - services/portal/tests/workflow_chat_thread_created_test.rs
    - services/portal/tests/workflow_sla_breach_test.rs
    - services/portal/tests/workflow_auto_priority_test.rs
    - services/portal/tests/workflow_attachments_test.rs
    - services/portal/tests/workflow_client_reply_test.rs
    - services/portal/tests/workflow_reopen_test.rs
    - services/portal/tests/workflow_kind_enum_cardinality_test.rs
    - services/portal/tests/workflow_status_enum_cardinality_test.rs
    - services/portal/tests/workflow_rate_limit_test.rs
    - services/portal/tests/workflow_audit_emission_test.rs

  modified_files:
    - services/portal/src/lib.rs
    - services/chat/src/                                              # add `client_workflow_id` to threads schema + `client_reply` message type
    - services/portal/Cargo.toml

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/chat/src/**
    - file_write: services/portal/{src,tests,migrations}/**
    - file_write: services/chat/src/**
    - bash: cd services/portal && cargo test workflows

  disallowed_tools:
    - bypass auto-routing rules (per DEC-1242)
    - expose internal CHAT messages to submitter (per DEC-1245)
    - allow file attachment > 25 MiB or > 5 files (per DEC-1246)
    - skip auto-priority on security keywords (per DEC-1247)

effort_hours: 6
subtasks:
  - "0.4h: 0018_portal_workflow_submissions.sql + 0019_portal_workflow_routing_rules.sql"
  - "0.4h: workflows/mod.rs + 2 closed enums"
  - "0.5h: workflows/submit.rs"
  - "0.5h: workflows/router.rs (rules → assignee)"
  - "0.4h: workflows/status_machine.rs"
  - "0.4h: workflows/sla_monitor.rs (scheduled job)"
  - "0.4h: workflows/attachments.rs (TASK-DOC-001 presigned)"
  - "0.3h: workflows/auto_priority.rs (keyword scan)"
  - "0.5h: workflows/chat_bridge.rs (CHAT thread create + sync)"
  - "0.3h: audit/workflow_events.rs (6 builders)"
  - "0.3h: handlers/workflow_routes.rs"
  - "1.3h: tests — 12 test files"
  - "0.3h: chat-side modifications"

risk_if_skipped: "Without client-initiated workflows, clients have no way to request anything through the portal — they fall back to email which loses audit trail + SLA tracking. Without DEC-1241 CHAT-thread integration, conversation history fragments between portal + chat. Without DEC-1244 SLA tracking, regulated tenants can't prove SLA adherence. Without DEC-1247 auto-priority on security keywords, breach reports get standard-queue handling delaying response. Without DEC-1245 internal/external message separation, internal handoff comments leak to client. The 6h effort completes the bidirectional PORTAL surface (read views + write workflows)."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship client-initiated workflows at `services/portal/src/workflows/` with 5 closed-enum workflow kinds, auto-routing to assignees via per-tenant rules, CHAT-thread bridging, SLA monitoring, auto-prioritisation on security keywords, file attachments via TASK-DOC-001, internal/external message separation, and 6 memory audit kinds.

1. **MUST** define closed `client_workflow_kind` enum: `('new_project_request','billing_inquiry','support_ticket','task','general_question')` per DEC-1240. Cardinality test asserts 5.

2. **MUST** define closed `workflow_status` enum: `('submitted','acknowledged','in_progress','awaiting_client','resolved','closed','escalated')` per DEC-1243. Cardinality test asserts 7.

3. **MUST** define `portal_workflow_submissions` table at migration `0018`: `(workflow_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, submitter_subject_id UUID NOT NULL, workflow_kind client_workflow_kind NOT NULL, status workflow_status NOT NULL DEFAULT 'submitted', title TEXT NOT NULL, body TEXT NOT NULL, attachments JSONB NOT NULL DEFAULT '[]'::jsonb, assignee_subject_id UUID, chat_thread_id UUID, sla_acknowledged_by TIMESTAMPTZ, sla_resolved_by TIMESTAMPTZ, acknowledged_at TIMESTAMPTZ, resolved_at TIMESTAMPTZ, escalated_at TIMESTAMPTZ, escalation_reason TEXT, submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), trace_id CHAR(32))`. RLS scoped to `tenant_id AND (submitter_subject_id = auth.subject_id OR assignee_subject_id = auth.subject_id OR has_role('engagement_admin'))`.

4. **MUST** define `portal_workflow_routing_rules` at migration `0019`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, workflow_kind client_workflow_kind NOT NULL, engagement_id UUID, assignee_subject_id UUID NOT NULL, priority INT NOT NULL DEFAULT 100, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by_subject_id UUID NOT NULL)`. Per-tenant rules; lower priority number wins; engagement_id NULL = tenant-wide rule.

5. **MUST** expose `POST /v1/portal/workflows/submit` body `{ engagement_id, workflow_kind, title, body, attachment_s3_keys?: [...] }`. Handler:
    - Validates JWT + Engagement membership.
    - Rate-limit per DEC-1249.
    - Validates workflow_kind in closed enum.
    - INSERTs workflow_submissions row with status='submitted'.
    - Auto-priority check per DEC-1247 — keyword scan body → if match, set status='escalated' + sev-1 alert.
    - Invokes router per §1 #6.
    - Creates CHAT thread per §1 #7.
    - Computes SLA timestamps from defaults + overrides.
    - Emit `portal.workflow_submitted` sev-2.
    - Returns 201 + `{ workflow_id, chat_thread_id, status, sla_acknowledged_by, sla_resolved_by }`.

6. **MUST** auto-route per DEC-1242. The `router.rs::route(tenant_id, workflow_kind, engagement_id)`:
    - Lookup matching rules ordered by `(engagement_id NOT NULL DESC, priority ASC)`.
    - First match's `assignee_subject_id` wins.
    - No rule matches → fallback to engagement_admin role-holder.
    - UPDATE workflow row with `assignee_subject_id`.
    - Emit `portal.workflow_routed`.

7. **MUST** create CHAT thread per DEC-1241 via `chat_bridge.rs::create_thread(workflow_id)`:
    - Calls TASK-CHAT-005 create-thread API in the Engagement-scoped channel.
    - Channel selection per workflow_kind: new_project_request→#new-projects; billing_inquiry→#billing; etc.
    - Thread initial message = workflow body + attachments rendered.
    - Records `chat_thread_id` on workflow row.
    - Adds `client_workflow_id` reference on the thread.

8. **MUST** separate internal vs external messages per DEC-1245. CHAT messages in the thread:
    - Default visibility = internal (engagement team only).
    - Messages flagged `client_visible: true` are mirrored back to portal UI for the submitter.
    - Portal client reply via `POST /v1/portal/workflows/{id}/reply` creates `client_reply` typed CHAT message (auto client_visible=true).
    - Emit `portal.workflow_client_reply`.

9. **MUST** monitor SLA per DEC-1244 via daily job:
    - For each non-terminal workflow: check `sla_acknowledged_by < now()` AND `acknowledged_at IS NULL` → emit `portal.workflow_sla_breach` sev-1 with breach_type='acknowledgement'.
    - Check `sla_resolved_by < now()` AND `resolved_at IS NULL` → emit breach with type='resolution'.
    - Mark workflow `escalated_at = now()`.

10. **MUST** support file attachments per DEC-1246. Handler:
    - Returns presigned S3 URLs from TASK-DOC-001 for each attachment slot at submission-start.
    - Validates uploaded files: max 25 MiB each, max 5 per workflow, allowed types only (pdf, png, jpg, docx, xlsx).
    - Persists S3 keys + SHA-256 in `attachments` JSONB.

11. **MUST** auto-prioritise on security keywords per DEC-1247. The `auto_priority.rs::check(body, workflow_kind)`:
    - For `support_ticket` kind: case-insensitive scan for {down, outage, breach, urgent, security, attack, leak, compromised, data loss}.
    - Match → status='escalated', escalation_reason='security_keyword_detected: <keyword>', sev-1 PagerDuty alert to engagement_admin.
    - Emit `portal.workflow_status_changed` with transition to escalated.

12. **MUST** expose `GET /v1/portal/workflows?engagement_id=...&status=...` for submitter list view. Returns own workflows + status mirror.

13. **MUST** expose `GET /v1/portal/workflows/{id}` for detail — shows status + client-visible CHAT messages + SLA timestamps; never internal messages.

14. **MUST** support reopen per DEC-1251. `POST /v1/portal/workflows/{id}/reopen`:
    - If `now() - closed_at < 30 days` → status='awaiting_client' + audit row.
    - If beyond → 400 + `reopen_window_expired; submit_new`.

15. **MUST** send email on status change per DEC-1252 via TASK-EMAIL-001. Per-tenant template overrides apply.

16. **MUST** emit 6 memory audit kinds per DEC-1248: submitted (sev-2), routed (sev-3), status_changed (sev-2), resolved (sev-2), sla_breach (sev-1), client_reply (sev-3).

17. **MUST** PII-scrub: title + body SHA256 only in chain; raw in DB.

18. **MUST** rate-limit per DEC-1249 — 10 submissions per workflow_kind per hour per caller.

19. **MUST** thread trace_id across submit → route → CHAT-create → audit.

20. **MUST** be RLS-scoped: submitter sees own; assignee sees assigned; engagement_admin sees Engagement-wide.

---

## §2 — Why this design (rationale for humans)

**Why CHAT-thread per workflow (§1 #7, DEC-1241)?** Conversation continuity. Workflow has chat history; engagement team chats internally; client sees client-visible portion. Re-implementing chat in PORTAL = waste; reusing CHAT-005 = audit + retention + search free.

**Why auto-prioritise security keywords (§1 #11, DEC-1247)?** Security incidents need response in minutes, not days. Keyword-based detection is crude but high-recall + zero-cost. False positives (e.g. "the system was down for a minute") get manual de-escalation; missed-positives (no keyword used) get standard SLA — acceptable.

**Why internal/external message separation (§1 #8, DEC-1245)?** Engagement team needs candid internal discussion ("client is wrong, but let's gently educate"). Surfacing those to client = brand damage. Default-internal + opt-in-external mirrors real consulting workflow.

**Why 30-day reopen window (§1 #14, DEC-1251)?** Resolved issues sometimes recur; convenient to reuse the original thread vs starting fresh. 30 days = "still recent in context" without indefinite resurrection.

**Why per-tenant routing rules (§1 #6, DEC-1242)?** Different tenants have different team structures. Hardcoded routing = wrong for everyone except CyberSkill. Rules + fallback to engagement_admin covers all cases.

---

## §3 — API contract

```sql
-- 0018_portal_workflow_submissions.sql
CREATE TYPE client_workflow_kind AS ENUM ('new_project_request','billing_inquiry','support_ticket','task','general_question');
CREATE TYPE workflow_status AS ENUM ('submitted','acknowledged','in_progress','awaiting_client','resolved','closed','escalated');

CREATE TABLE portal_workflow_submissions (
  workflow_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  submitter_subject_id UUID NOT NULL,
  workflow_kind client_workflow_kind NOT NULL,
  status workflow_status NOT NULL DEFAULT 'submitted',
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  attachments JSONB NOT NULL DEFAULT '[]'::jsonb,
  assignee_subject_id UUID,
  chat_thread_id UUID,
  sla_acknowledged_by TIMESTAMPTZ,
  sla_resolved_by TIMESTAMPTZ,
  acknowledged_at TIMESTAMPTZ,
  resolved_at TIMESTAMPTZ,
  escalated_at TIMESTAMPTZ,
  escalation_reason TEXT,
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
CREATE INDEX idx_workflow_submitter ON portal_workflow_submissions(submitter_subject_id, submitted_at DESC);
CREATE INDEX idx_workflow_assignee ON portal_workflow_submissions(assignee_subject_id, status) WHERE status != 'closed';
CREATE INDEX idx_workflow_sla_check ON portal_workflow_submissions(sla_acknowledged_by, sla_resolved_by) WHERE status IN ('submitted','acknowledged','in_progress');
ALTER TABLE portal_workflow_submissions ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_workflow_submissions_rls ON portal_workflow_submissions
  USING (
    tenant_id = current_setting('auth.tenant_id')::uuid
    AND (
      submitter_subject_id = current_setting('auth.subject_id')::uuid
      OR assignee_subject_id = current_setting('auth.subject_id')::uuid
      OR EXISTS (SELECT 1 FROM subject_roles
                 WHERE subject_id = current_setting('auth.subject_id')::uuid
                   AND role = 'engagement_admin'
                   AND scope_engagement_id = portal_workflow_submissions.engagement_id)
    )
  )
  WITH CHECK (
    tenant_id = current_setting('auth.tenant_id')::uuid
    AND submitter_subject_id = current_setting('auth.subject_id')::uuid
  );
REVOKE DELETE ON portal_workflow_submissions FROM cyberos_app;
GRANT UPDATE (status, assignee_subject_id, chat_thread_id, sla_acknowledged_by, sla_resolved_by,
              acknowledged_at, resolved_at, escalated_at, escalation_reason, updated_at)
  ON portal_workflow_submissions TO cyberos_app;

-- 0019_portal_workflow_routing_rules.sql
CREATE TABLE portal_workflow_routing_rules (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  workflow_kind client_workflow_kind NOT NULL,
  engagement_id UUID,
  assignee_subject_id UUID NOT NULL,
  priority INT NOT NULL DEFAULT 100,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_subject_id UUID NOT NULL
);
CREATE INDEX idx_routing_lookup ON portal_workflow_routing_rules(tenant_id, workflow_kind, engagement_id, priority);
ALTER TABLE portal_workflow_routing_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_workflow_routing_rules_rls ON portal_workflow_routing_rules
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_workflow_routing_rules FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/portal/workflows/submit                (submitter)
GET    /v1/portal/workflows?engagement_id=...     (own list)
GET    /v1/portal/workflows/{id}                   (detail)
POST   /v1/portal/workflows/{id}/reply             (submitter reply)
POST   /v1/portal/workflows/{id}/reopen            (submitter; 30d window)
POST   /v1/admin/tenants/{tid}/workflow-routes     (tenant_admin: configure rules)
```

---

## §4 — Acceptance criteria

1. **client_workflow_kind cardinality** — 5 values.
2. **workflow_status cardinality** — 7 values.
3. **Submit creates CHAT thread** — `POST /submit` → workflow row + chat thread created in proper channel.
4. **Auto-routing per rule** — rule for tenant + workflow_kind → assignee_subject_id populated.
5. **Fallback to engagement_admin** — no rule → engagement_admin assigned.
6. **Auto-priority on security keyword** — body containing "breach" → status='escalated' + sev-1 alert.
7. **SLA breach detected** — daily job marks workflow `sla_acknowledged_by < now()` → emits sev-1 breach audit.
8. **File attachment ≤ 25 MiB** — uploaded file 26 MiB → 413.
9. **Max 5 attachments** — 6th attachment → 400.
10. **Internal messages hidden from submitter** — CHAT message with `client_visible: false` NOT in submitter detail view.
11. **Client reply creates client_visible message** — POST /reply creates CHAT message with type='client_reply' + visibility=true.
12. **Reopen within 30 days** — closed workflow + 29 days later → reopen succeeds.
13. **Reopen beyond 30 days** — 31 days later → 400.
14. **Email on status change** — status transition triggers TASK-EMAIL-001 send.
15. **Rate limit 10/hr/kind/caller** — 11th submission of same kind → 429.
16. **6 memory audit kinds emitted** — full lifecycle covers all 6.
17. **PII scrubbed** — title/body SHA256 only in chain.
18. **Trace_id threaded** — submit → route → chat-create → audit all share trace_id.
19. **RLS — engagement_admin sees engagement-wide** — engagement_admin lists shows all workflows in their Engagement.
20. **RLS — submitter sees own only** — submitter list excludes other submitters' rows.

---

## §5 — Verification

```rust
// 5.1 submit creates CHAT thread
#[tokio::test]
async fn submit_creates_workflow_and_chat_thread() {
    let ctx = TestContext::with_engagement_subject().await;
    ctx.seed_chat_channel("billing").await;
    let r = ctx.post_workflow("billing_inquiry", "Q on invoice 42", "...").await;
    assert_eq!(r.status(), 201);
    let body: serde_json::Value = r.json().await.unwrap();
    assert!(body["chat_thread_id"].is_string());

    let thread_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM chat_threads WHERE id=$1)"
    ).bind(body["chat_thread_id"].as_str().unwrap().parse::<Uuid>().unwrap()).fetch_one(&ctx.pool).await.unwrap();
    assert!(thread_exists);
}

// 5.2 auto-priority on breach keyword
#[tokio::test]
async fn breach_keyword_escalates() {
    let ctx = TestContext::with_engagement_subject().await;
    let r = ctx.post_workflow("support_ticket", "URGENT: data breach detected", "...").await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["status"], "escalated");

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.workflow_status_changed"
        && r.payload["new_status"] == "escalated"));
}

// 5.3 SLA breach detection
#[tokio::test]
async fn sla_breach_emits_sev1() {
    let ctx = TestContext::new().await;
    let wid = ctx.create_workflow_with_past_sla().await;
    ctx.run_sla_monitor_job().await;
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.workflow_sla_breach" && r.severity == 1));
}

// 5.4 internal hidden, external visible
#[tokio::test]
async fn submitter_sees_only_client_visible_messages() {
    let ctx = TestContext::with_engagement_subject().await;
    let wid = ctx.create_workflow().await;
    ctx.as_engagement_team().post_chat_message(wid, "internal note", false).await;
    ctx.as_engagement_team().post_chat_message(wid, "client-visible reply", true).await;

    let r = ctx.as_submitter().get_workflow_detail(wid).await;
    let messages: Vec<&str> = ctx.extract_messages(r).await;
    assert!(messages.contains(&"client-visible reply"));
    assert!(!messages.contains(&"internal note"));
}

// 5.5 reopen 30d window
#[tokio::test]
async fn reopen_within_window() {
    let ctx = TestContext::new().await;
    let wid = ctx.create_and_close_workflow().await;
    ctx.travel_clock_forward(Duration::from_days(29)).await;
    let r = ctx.reopen_workflow(wid).await;
    assert_eq!(r.status(), 200);
    let status: String = sqlx::query_scalar("SELECT status::text FROM portal_workflow_submissions WHERE workflow_id=$1")
        .bind(wid).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "awaiting_client");
}

// 5.6..5.12: enum cardinality, rate limit, routing rule, attachment cap, email, audit
```

---

## §7 — Dependencies

**Upstream:** TASK-CHAT-005 (thread + message primitive).
**Cross-module:** TASK-PORTAL-001 (workflow shown in PORTAL list), TASK-PORTAL-003 (IdP subject), TASK-PORTAL-005 (Genie may surface workflow state), TASK-PROJ-001 (new_project_request creates PROJ entity), TASK-INV-001 (billing_inquiry may reference invoices), TASK-AUTH-101 (engagement_admin role), TASK-EMAIL-001 (notification + template overrides), TASK-AI-003 (audit), TASK-MEMORY-111 (PII scrub), TASK-OBS-007 (sev-1 escalation routing).
**Downstream:** None.

---

## §8 — Example payload

`portal.workflow_submitted`:
```json
{
  "kind": "portal.workflow_submitted",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.submitter.456",
  "trace_id": "...",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "workflow_id": "0190...",
    "engagement_id": "0190...",
    "workflow_kind": "billing_inquiry",
    "title_sha256": "9c4e...",
    "chat_thread_id": "0190...",
    "sla_acknowledged_by": "2026-05-17T13:14:32Z",
    "sla_resolved_by": "2026-05-22T09:14:32Z"
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Per-workflow custom fields (slice 3).
- **Deferred:** Workflow templates (slice 3).
- **Deferred:** Multi-step workflows with approval gates (slice 3).
- **Deferred:** Workflow analytics dashboard for tenant_admin (slice 3).
- **Deferred:** Bulk client reply (multi-workflow) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unknown workflow_kind | enum check | 400 | Caller fixes |
| Engagement not in membership | scope check | 403 | Caller's engagements only |
| CHAT thread creation fails | TASK-CHAT-005 error | Workflow row rolled back; 500 | Operator investigates CHAT |
| Routing rule missing + no engagement_admin | fallback miss | Sev-1 alert; workflow remains 'submitted' unassigned | Tenant_admin assigns rule |
| Attachment upload fails | S3 error | 503; caller retries | Inherent |
| Attachment > 25 MiB | size check | 413 | Caller compresses |
| > 5 attachments | count check | 400 | Caller reduces |
| Auto-priority false-positive | manual review | Engagement_admin de-escalates via status_change | Inherent UX |
| SLA breach not detected (job failure) | daily monitor + watchdog | Sev-2 alert | Operator runs monitor manually |
| Reopen beyond 30d | window check | 400 | New submission |
| Rate limit hit | counter | 429 | Caller waits |
| Submitter tries to see internal CHAT messages | client_visible filter | Internal hidden | Inherent |
| Cross-tenant submission | RLS rejects | 403 | Inherent |
| Email send fails | TASK-EMAIL-001 error | Status change still committed; email retried by TASK-EMAIL-001 job | Inherent retry |
| Submitter deprovisioned mid-workflow (TASK-PORTAL-004) | session revoked | Subsequent actions 401; workflow row remains | Engagement team continues internally |
| Engagement_admin role lost mid-workflow | RLS at next read | Engagement_admin loses workflow visibility | Tenant_admin re-grants role |
| Status state machine invalid transition (e.g. resolved → submitted) | state guard | 400 + `invalid_status_transition` | Caller fixes flow |
| Reopen of escalated workflow | special handling | Status → awaiting_client; escalated_at cleared | Inherent |
| Concurrent status updates | optimistic lock via updated_at | Last writer wins; loser sees 409 | Caller refetches + retries |
| Workflow with no chat_thread_id | bridge failure during creation | Row exists in degraded state; sev-2 alert | Background job retries CHAT create |
| Attachment SHA256 mismatch | server verification | Attachment rejected | Caller re-uploads |

---

## §11 — Implementation notes

**§11.1** Routing rule lookup uses index on (tenant_id, workflow_kind, engagement_id, priority); typical query < 1ms.

**§11.2** Security keyword list maintained in `auto_priority.rs::SECURITY_KEYWORDS` const; review quarterly.

**§11.3** Status state machine: enforces valid transitions; e.g., submitted → acknowledged → in_progress → resolved → closed; escalated valid from any non-terminal.

**§11.4** SLA business-hours calculator uses tenant timezone + business calendar; defaults UTC weekdays.

**§11.5** CHAT thread channel mapping: per-tenant config; default `#workflow-<kind>`.

**§11.6** Client reply via portal UI: server creates CHAT message via TASK-CHAT-005 with type='client_reply' + author=submitter_subject_id.

**§11.7** Email templates per TASK-PORTAL-002 brand pack: per-tenant override fallback chain.

**§11.8** Rate limit Redis sliding-window per (caller, workflow_kind).

**§11.9** Attachments JSONB shape: `[{s3_key, sha256, size_bytes, filename, mime_type}]`.

**§11.10** Reopen audit reuses the same workflow_id (vs new row); preserves history continuity.

---

*End of TASK-PORTAL-006 spec.*
