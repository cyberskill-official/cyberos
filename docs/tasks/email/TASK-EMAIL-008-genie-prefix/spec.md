---
id: TASK-EMAIL-008
title: "EMAIL Genie prefix — inbound subject prefix routes message to Genie (Branded AI) for automated drafting + action proposals"
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
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-EMAIL-001, TASK-PORTAL-005, TASK-AI-003, TASK-CUO-101, TASK-MEMORY-111]
depends_on: [TASK-EMAIL-001, TASK-EMAIL-005, TASK-PORTAL-005, TASK-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/email.html#genie-prefix

source_decisions:
  - DEC-1590 2026-05-17 — Tenant-configurable subject prefix (default "Genie:") routes inbound to Branded Genie chat flow (TASK-PORTAL-005)
  - DEC-1591 2026-05-17 — Genie processes message → proposes 0-N actions: draft_reply, create_issue, summarize_thread, fetch_data, escalate_human
  - DEC-1592 2026-05-17 — Closed enum `genie_action_kind` = {draft_reply, create_issue, summarize_thread, fetch_data, escalate_human, no_action}; cardinality 6
  - DEC-1593 2026-05-17 — Action proposals ALWAYS reviewed by user before execution — never auto-execute (matches TASK-INV-010 dunning pattern)
  - DEC-1594 2026-05-17 — Genie context: tenant brand pack (TASK-PORTAL-002), CRM context (TASK-CRM-001), recent thread context (last 10 msgs), tenant tools (per TASK-MCP-006 gating)
  - DEC-1595 2026-05-17 — memory audit kinds: email.genie_triggered, email.genie_action_proposed, email.genie_action_approved, email.genie_action_executed, email.genie_action_dismissed, email.genie_failed

language: rust 1.81
service: cyberos/services/email/
new_files:
  - services/email/migrations/0010_genie_sessions.sql
  - services/email/src/genie/mod.rs
  - services/email/src/genie/prefix_router.rs
  - services/email/src/genie/action_proposer.rs
  - services/email/src/genie/portal_bridge.rs
  - services/email/src/handlers/genie_routes.rs
  - services/email/src/audit/genie_events.rs
  - services/email/tests/genie_prefix_match_test.rs
  - services/email/tests/genie_action_proposed_test.rs
  - services/email/tests/genie_no_auto_execute_test.rs
  - services/email/tests/genie_action_kind_enum_cardinality_test.rs
  - services/email/tests/genie_context_loading_test.rs
  - services/email/tests/genie_audit_emission_test.rs

modified_files:
  - services/email/src/inbound_processor.rs

allowed_tools:
  - file_read: services/{email,portal,crm}/**
  - file_write: services/email/{src,tests,migrations}/**
  - bash: cd services/email && cargo test genie

disallowed_tools:
  - auto-execute action without user approval (per DEC-1593)
  - bypass tool gating from TASK-MCP-006

effort_hours: 8
subtasks:
  - "0.4h: 0010_genie_sessions.sql"
  - "0.3h: genie/mod.rs"
  - "0.5h: prefix_router.rs"
  - "1.0h: action_proposer.rs (Branded Genie integration)"
  - "0.6h: portal_bridge.rs (TASK-PORTAL-005 call)"
  - "0.5h: handlers/genie_routes.rs"
  - "0.4h: audit/genie_events.rs"
  - "0.5h: inbound_processor.rs hook"
  - "2.4h: tests — 6 test files"
  - "1.4h: CDO UI for action review queue"

risk_if_skipped: "Without Genie prefix routing, AM/CDO must manually use Branded Genie for each email — workflow friction. Without DEC-1593 manual approval, AI hallucination causes wrong customer replies. Without DEC-1594 brand-pack context, Genie sounds off-brand."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship Genie-prefix routing at `services/email/src/genie/` matching subject prefix, calling TASK-PORTAL-005 Branded Genie, proposing actions, queueing for user approval, 6 memory audit kinds.

1. **MUST** hook into `services/email/src/inbound_processor.rs` after message stored: call `genie::route(message)`.

2. **MUST** match prefix per DEC-1590 via `prefix_router.rs::matches(subject, tenant)`:
   - Load `tenant.genie_prefix` (default "Genie:").
   - Case-insensitive starts-with match after stripping `Re:`/`Fwd:`.

3. **MUST** load context per DEC-1594 — brand pack (TASK-PORTAL-002), CRM contact via TASK-EMAIL-006 link, thread last-10-msgs, tenant tool list from TASK-MCP-006.

4. **MUST** call Branded Genie at `portal_bridge.rs::propose(message, context)` — invokes TASK-PORTAL-005 chat with system prompt: "Read this email, propose 0-N actions. Output JSON [{kind, params, rationale}]".

5. **MUST** validate proposed `genie_action_kind` against closed enum per DEC-1592.

6. **MUST** queue actions for user review per DEC-1593 — NEVER auto-execute.

7. **MUST** define `genie_sessions` and `genie_actions` tables at migration `0010`:
   ```sql
   CREATE TABLE genie_sessions (
     session_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     message_id UUID NOT NULL,
     thread_id UUID NOT NULL,
     status TEXT NOT NULL DEFAULT 'proposing'
       CHECK (status IN ('proposing','awaiting_review','executing','completed','failed','dismissed')),
     started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ,
     trace_id CHAR(32)
   );
   ALTER TABLE genie_sessions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY genie_sessions_rls ON genie_sessions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON genie_sessions FROM cyberos_app;
   GRANT UPDATE (status, completed_at) ON genie_sessions TO cyberos_app;

   CREATE TABLE genie_actions (
     action_id UUID PRIMARY KEY,
     session_id UUID NOT NULL REFERENCES genie_sessions(session_id),
     tenant_id UUID NOT NULL,
     kind TEXT NOT NULL
       CHECK (kind IN ('draft_reply','create_issue','summarize_thread','fetch_data','escalate_human','no_action')),
     params JSONB NOT NULL,
     rationale TEXT,
     status TEXT NOT NULL DEFAULT 'proposed'
       CHECK (status IN ('proposed','approved','executed','dismissed','failed')),
     reviewed_by UUID,
     reviewed_at TIMESTAMPTZ,
     executed_at TIMESTAMPTZ,
     result JSONB,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE genie_actions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY genie_actions_rls ON genie_actions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON genie_actions FROM cyberos_app;
   GRANT UPDATE (status, reviewed_by, reviewed_at, executed_at, result) ON genie_actions TO cyberos_app;
   ```

8. **MUST** emit 6 memory audit kinds per DEC-1595. PII per TASK-MEMORY-111: message body + AI output SHA-256 hashed.

9. **MUST** thread trace_id from inbound → prefix match → Genie → action queue → user approve → execute.

10. **MUST** add tenant config column to extend TASK-EMAIL-001:
   ```sql
   ALTER TABLE email_tenant_config ADD COLUMN genie_prefix TEXT DEFAULT 'Genie:';
   ALTER TABLE email_tenant_config ADD COLUMN genie_enabled BOOLEAN DEFAULT false;
   ```

11. **MUST NOT** auto-execute action per DEC-1593.

12. **MUST NOT** bypass TASK-MCP-006 tool gating when executing fetch_data action — only allowlisted tools per tenant.

---

## §2 — Why this design

**Why prefix routing (DEC-1590)?** Lightweight opt-in per-message; doesn't intercept regular email flow.

**Why action proposals not direct execution (DEC-1593)?** AI hallucination risk on customer-facing sends; manual review is the gate.

**Why 6 action kinds (DEC-1591)?** Covers common Genie use cases — reply drafts, issue creation, thread summary, data lookup, human escalation, no-op (matches existing CDO workflows).

**Why brand pack context (DEC-1594)?** Branded Genie must speak in tenant voice; brand pack defines tone+language+disclaimers.

---

## §3 — API contract

```text
GET    /v1/email/genie/sessions                  (list user's pending Genie reviews)
GET    /v1/email/genie/sessions/{id}             (detail with proposed actions)
POST   /v1/email/genie/actions/{id}/approve      (execute)
POST   /v1/email/genie/actions/{id}/dismiss      (mark dismissed)
PUT    /v1/email/genie/config                    (tenant prefix + enabled toggle)
```

Sample session detail:
```json
{
  "session_id": "uuid",
  "message_id": "uuid",
  "status": "awaiting_review",
  "actions": [
    {
      "action_id": "uuid",
      "kind": "draft_reply",
      "params": {"body": "Dear John, thanks for reaching out..."},
      "rationale": "Customer asked about pricing; drafted standard response.",
      "status": "proposed"
    },
    {
      "action_id": "uuid",
      "kind": "create_issue",
      "params": {"project_id": "...", "title": "Schedule pricing demo for Acme"},
      "rationale": "Follow-up needed in 24h.",
      "status": "proposed"
    }
  ]
}
```

---

## §4 — Acceptance criteria
1. **Subject prefix triggers Genie**. 2. **Case-insensitive match**. 3. **Strips Re:/Fwd: before match**. 4. **Tenant prefix configurable**. 5. **Genie_enabled toggle respected (off → skip silently)**. 6. **Context loaded (brand + CRM + thread + tools)**. 7. **6 action kinds enum + cardinality test**. 8. **Actions queued, never auto-executed**. 9. **6 memory audit kinds emitted**. 10. **PII scrubbed (body/AI output SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **fetch_data respects TASK-MCP-006 gating**. 14. **Approve → execute (calls TASK-EMAIL-009 send / TASK-PROJ-001 create / etc.)**. 15. **Dismiss → status=dismissed (audit)**. 16. **AI failure → status=failed + sev-2**. 17. **Append-only sessions/actions tables**. 18. **Multiple actions per session executed in order**. 19. **Result of execution stored in actions.result**. 20. **Branded Genie call uses brand pack tone**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn prefix_triggers_genie() {
    let ctx = TestContext::with_genie_enabled().await;
    let msg = ctx.receive_inbound_with_subject("Genie: Draft a reply").await;
    let session = ctx.fetch_genie_session_for_msg(msg.id).await;
    assert!(session.is_some());
}

#[tokio::test]
async fn never_auto_executes() {
    let ctx = TestContext::with_genie_enabled().await;
    let msg = ctx.receive_inbound_with_subject("Genie: Help me").await;
    let session = ctx.wait_for_session_proposing(msg.id).await;
    let sent_emails = ctx.email_send_count().await;
    let issues = ctx.issue_create_count().await;
    assert_eq!(sent_emails, 0);
    assert_eq!(issues, 0);
    assert_eq!(session.status, "awaiting_review");
}

#[tokio::test]
async fn case_insensitive_prefix() {
    let ctx = TestContext::with_genie_prefix("Genie:").await;
    let cases = ["GENIE: help", "genie: help", "Genie: help", "Re: GENIE: reply", "Fwd: Genie: forward"];
    for subj in cases {
        let msg = ctx.receive_inbound_with_subject(subj).await;
        assert!(ctx.fetch_genie_session_for_msg(msg.id).await.is_some());
    }
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-EMAIL-001, TASK-PORTAL-005, TASK-CUO-101.
**Cross-module:** TASK-PORTAL-002 (brand pack), TASK-CRM-001 (contact), TASK-MCP-006 (tool gating), TASK-AI-003 (LLM), TASK-MEMORY-111 (PII).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Genie disabled (tenant) | flag check | skip silently | inherent |
| Branded Genie unreachable | HTTP timeout | status=failed; sev-2 | retry |
| AI returns invalid action kind | enum match | filter + sev-3 audit | inherent |
| User no email send permission | exec time | action fails, status=failed | request perm |
| Tool not in allowlist (fetch_data) | TASK-MCP-006 gate | rejected | inherent |
| Prefix not configured | use default "Genie:" | inherent | inherent |
| Concurrent prefix match on same msg | UNIQUE on session_id+message_id | first wins | inherent |
| Approve already-executed action | status check | 409 | inherent |
| Brand pack missing | TASK-PORTAL-002 fallback | use default tone | inherent |
| Genie context too large (>50k tokens) | truncate | last 10 msgs only | inherent |

## §11 — Implementation notes
- §11.1 Prefix match regex: `^(re:|fwd:)?\s*{escaped_prefix}\s*` case-insensitive.
- §11.2 Action proposer prompt includes JSON schema for output validation.
- §11.3 Result column stores execution outcome (e.g. sent_message_id, created_issue_id).
- §11.4 memory audit body: action kinds + counts; AI output SHA256.
- §11.5 fetch_data executes via TASK-MCP-006-gated MCP tools; tenant must have allowlisted them.

---

*End of TASK-EMAIL-008 spec.*
