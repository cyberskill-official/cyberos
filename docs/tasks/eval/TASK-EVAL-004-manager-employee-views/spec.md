---
id: TASK-EVAL-004
title: "manager + employee evaluation views — access-restricted console panel over the eval service; founder/manager-of-report/self-only reads, the auto-score shown as a clearly-marked DRAFT requiring human approval before it is final, and the employee's right to see + respond to their own record; every cross-person read audited"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-06-29T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: EVAL
priority: p0
status: draft
verify: T
phase: P5
milestone: P5 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-06-29
shipped: null
memory_chain_hash: null
related_tasks: [TASK-EVAL-001, TASK-EVAL-002, TASK-EVAL-003, TASK-APP-001, TASK-AUTH-003, TASK-AUTH-004, TASK-APP-005, TASK-MEMORY-123]
depends_on: [TASK-EVAL-003, TASK-APP-001]
blocks: []

source_pages:
  - docs/strategy/cyberos-brain-evaluation-plan.md#phase-5-surfacing
source_decisions:
  - DEC-2620 (access-restricted + contract-disclosed — the evaluation views are visible ONLY to the founder, a designated manager for their own reports, and the employee's own self-view; enforced by tenant RLS + the TASK-EVAL-001 access grants; the access basis is disclosed in the signed documents per TASK-EVAL-001, never covert)
  - DEC-2621 (both auto-score AND HITL — the panel MUST show the TASK-EVAL-003 auto-score as a clearly-marked DRAFT, MUST require a human reviewer's approval before any assessment is final, and the auto-score alone MUST NEVER read as a final/decided result)
  - DEC-2622 (the employee has the right to see their own assessment and to record a response; the response is captured, audited, and shown alongside the assessment, and an assessment is not 'closed' until the employee has had the opportunity to respond)
  - DEC-2623 (every cross-person read is audited — when the founder or a manager opens an employee's record, a hash-chained audit row is written naming who read whose record; self-views are audited too; reads are first-class events, not silent)
  - DEC-2624 (the panel is one more panel in the TASK-APP-001 console, adds no backend beyond the eval service's own read/approve/respond endpoints, reuses the CDS design language, and is access-gated by TASK-EVAL-001 — it does not invent its own access rule)

eu_ai_act_risk_class: high

language: rust 1.81 + static JS (apps/console)
service: cyberos/services/eval/ + apps/console
new_files:
  - services/eval/src/views/mod.rs
  - services/eval/src/views/access.rs
  - services/eval/src/views/manager.rs
  - services/eval/src/views/employee.rs
  - services/eval/src/views/response.rs
  - services/eval/tests/views_access_test.rs
  - services/eval/tests/views_approval_test.rs
  - services/eval/tests/views_response_audit_test.rs
  - apps/console/src/api/eval.ts
  - apps/console/src/screens/eval_manager.ts
  - apps/console/src/screens/eval_self.ts
  - apps/console/src/render/assessment_card.ts
  - apps/console/tests/eval_api_client.test.ts
  - apps/console/tests/eval_draft_badge_render.test.ts
modified_files:
  # mount the views routes
  - services/eval/src/lib.rs
  # add eval.assessment_read / _approved / _overridden / _response_recorded kinds
  - services/eval/src/audit.rs
  # register the eval panel in the TASK-APP-001 shell nav (access-gated)
  - apps/console/src/main.ts
allowed_tools:
  - file_read: services/eval/**
  - file_read: apps/console/**
  - file_write: services/eval/{src,tests}/**
  - file_write: apps/console/{src,tests}/**
  - bash: cd services/eval && cargo test views
disallowed_tools:
  - return an assessment to any caller who is not the founder, the subject's manager-of, or the subject themselves (per DEC-2620 — access-restricted)
  - render the auto-score without the DRAFT state, or present a draft as final/approved (per DEC-2621 — auto-score is always a marked draft until a human approves)
  - close an assessment before the employee has had the opportunity to respond (per DEC-2622)
  - serve any cross-person assessment read without emitting an eval.assessment_read audit row (per DEC-2623)
  - add a backend endpoint outside the eval service, or invent an access rule instead of using the TASK-EVAL-001 grants (per DEC-2624)

effort_hours: 9
subtasks:
  - "1.0h: views/access.rs — the founder / manager-of-report / self access check over AUTH's manager-of relationship + TASK-EVAL-001 grants; deny-by-default"
  - "1.0h: views/manager.rs — GET a report's assessment (rubric mapping, evidence, the DRAFT auto-score, the approve/override control state)"
  - "0.5h: views/employee.rs — GET own assessment (self-view), same card minus other people"
  - "1.0h: views/response.rs — POST the employee response + the manager approve/override transitions; state machine draft -> approved | overridden, closed only after response opportunity"
  - "0.5h: wire eval.assessment_{read,approved,overridden,response_recorded} audit rows through the memory chain (read is audited)"
  - "1.0h: REST endpoints, tenant-scoped + TASK-EVAL-001-gated, every cross-person read audited"
  - "1.5h: apps/console eval_manager.ts + eval_self.ts + assessment_card.ts (DRAFT badge, approve/override control, response box) inside the TASK-APP-001 shell + CDS"
  - "1.0h: api/eval.ts client + render unit tests (DRAFT-vs-APPROVED badge, access-denied state) against fixtures"
  - "1.5h: views_access_test + views_approval_test + views_response_audit_test (access matrix, draft-not-final, response-before-close, read audited)"
risk_if_skipped: "Without the surfacing layer, the assessments TASK-EVAL-003 produces live only in the database and are never seen — the evaluation has no defensible, transparent outcome, defeating the purpose. Without the access restriction, a person's evaluation could be read by someone with no business seeing it, breaching the disclosed access basis and Vietnam's PDPD. Without the DRAFT-vs-APPROVED distinction, an auto-generated score reads as a decided judgement and a model effectively decides pay/progression — the exact thing the plan forbids. Without the employee response and the read audit, the system is opaque surveillance rather than the transparent, contestable record the governance plan requires; transparency here is a feature, not an afterthought."
---

## §1 — Description (BCP-14 normative)

The evaluation-views panel **MUST** surface TASK-EVAL-003 assessments to a strictly-bounded audience, always show the auto-score as a clearly-marked draft a human must approve, and give the employee the right to see and respond to their own record, with every cross-person read audited. The contract:

1. **MUST** be one panel in the same static single-page app as TASK-APP-001, under `apps/console/`, mounted inside the TASK-APP-001 shell and behind its auth gate, using CDS tokens and components (DEC-2624). It **MUST NOT** define its own shell, sign-in flow, or design language; it reuses what TASK-APP-001 established (the TASK-APP-005 pattern).

2. **MUST** be access-restricted to exactly three reader relationships (DEC-2620), enforced server-side in `services/eval/src/views/access.rs`, deny-by-default:
- the **founder** (the tenant owner role);
- a **manager** viewing only an employee who is their own report, established through AUTH's manager-of relationship (TASK-AUTH-003);
- the **employee themselves**, viewing only their own record (the self-view). Any other caller — including a manager requesting a non-report, or any peer — **MUST** receive `403 eval_view_forbidden` and **MUST NOT** see any assessment content, not even existence/metadata.

3. **MUST** treat the access decision as the TASK-EVAL-001 governance grant plus the AUTH relationship, not a rule this panel invents (DEC-2624). The check **MUST** consult the TASK-EVAL-001 access grants (who may view evaluations at all) and then the manager-of relationship (which specific reports a manager may view). A caller lacking the TASK-EVAL-001 grant is denied even for their own self-view path until governance has provisioned it.

4. **MUST** present a manager view (`apps/console/src/screens/eval_manager.ts` over `services/eval/src/views/manager.rs`) for an authorised manager/founder that shows, for one report: the assessment's rubric mapping (each TASK-EVAL-002 rubric item, with its `source_doc`/`clause_ref`), the evidence TASK-EVAL-003 linked per item, the **auto-score draft**, and the approve/override control. It **MUST NOT** show a report's assessment to a manager who is not that report's manager-of.

5. **MUST** render the TASK-EVAL-003 auto-score as a clearly-marked DRAFT and **MUST NEVER** let it read as final (DEC-2621):
- the assessment carries an explicit `state` of `draft | approved | overridden` (sourced from TASK-EVAL-003 / the response flow), and the panel **MUST** show a prominent DRAFT badge whenever `state='draft'`.
- a `draft` assessment **MUST** be visually and textually distinguished from an `approved`/`overridden` one (badge, label, and a "not yet reviewed by a human" note); the auto-score **MUST NOT** be displayed as a decided result, a final grade, or anything that reads as an employment decision.
- the panel **MUST NOT** offer any "final" or "decided" affordance for a `draft` assessment other than the human approve/override control.

6. **MUST** require a human reviewer's approval before an assessment is final (DEC-2621, EU AI Act Article 14):
- the approve/override control (`services/eval/src/views/response.rs`) lets the authorised manager/founder either **approve** the draft (state → `approved`, recording `reviewer_subject_id`) or **override** it (state → `overridden`, recording the reviewer's adjusted outcome + a required `override_reason`).
- the transition out of `draft` **MUST** record a human `reviewer_subject_id`; a transition with a null or service-account reviewer **MUST** be rejected (`403 review_requires_human`).
- the model's auto-score is never the final word: `approved` means a human endorsed it; `overridden` means a human replaced it; neither happens without a human acting.

7. **MUST** provide an employee self-view (`apps/console/src/screens/eval_self.ts` over `services/eval/src/views/employee.rs`) that shows the signed-in employee their own assessment — the same rubric mapping, evidence, score, and state — and **MUST NOT** expose any other person's record, any manager's private notes not meant for the employee, or any other employee's data through it.

8. **MUST** capture the employee's right to respond (DEC-2622):
- the self-view **MUST** let the employee record a free-text response to their own assessment (`POST .../response`), stored against the assessment with the author and timestamp.
- the response **MUST** be shown alongside the assessment in both the self-view and the manager view (the manager sees the employee's response).
- an assessment **MUST NOT** be marked `closed` until the employee has had the opportunity to respond — either the employee has recorded a response, or an explicit, audited "response window elapsed / waived" event has occurred. A close attempt before that **MUST** be rejected (`409 response_opportunity_pending`).

9. **MUST** audit every cross-person read (DEC-2623): when the founder or a manager opens an employee's assessment, the panel's backing endpoint **MUST** emit a hash-chained `eval.assessment_read` memory audit row naming `{reader_subject_id, subject_employee_id, assessment_id, relationship, trace_id}`. Self-views **MUST** also emit an `eval.assessment_read` row (reader == subject). Reads are first-class audited events, never silent.

10. **MUST** emit a hash-chained audit row for every state-affecting action through `services/eval/src/audit.rs`: `eval.assessment_approved`, `eval.assessment_overridden` (carrying `override_reason`), and `eval.assessment_response_recorded`. Each chains into the same `l1_audit_log` the rest of CyberOS uses (the TASK-PROJ-008 / TASK-MEMORY-123 pattern).

11. **MUST** be tenant-scoped through the session: every eval read/write **MUST** carry the TASK-AUTH-004 session token the TASK-APP-001 shell holds, and results **MUST** be the tenant scope the eval service's RLS applies to that token. The panel **MUST NOT** attempt to widen scope or pass a tenant other than the session's.

12. **MUST** add no backend beyond the eval service's own read/approve/respond endpoints (DEC-2624). The console is a front-end over those endpoints; a screen that appears to need a new endpoint is a signal to extend the eval service's task, not to add a backend in `app`. The access check, the audit emission, and the state machine live in the eval service, not in the browser.

13. **MUST** fail visibly and safely on error: on a 401 the panel **MUST** defer to the TASK-APP-001 auth gate and return to sign-in; on a 403 it **MUST** show an explicit access-denied state (not an empty panel that reads as "no assessments"); on any other non-2xx the affected screen **MUST** show a clear error and **MUST NOT** fabricate an assessment, a score, or a state. An empty-but-successful read (the employee has no assessment yet) **MUST** be shown as "no assessment yet", distinct from denied and from error.

14. **MUST NOT** let the browser compute or alter the score, the state, or the access decision. The panel renders the eval service's verdict — the score, the `draft`/`approved`/`overridden` state, and whether the caller is allowed — exactly as the service returns them; it never recomputes a score client-side, never up-grades a `draft` to look `approved`, and never makes its own allow/deny call.

15. **MUST** keep the API-client and render functions pure where possible and unit-tested without a live backend, against fixtures for the assessment-card shape, the DRAFT-vs-APPROVED badge, and the access-denied state. A live render against running eval + auth is an owner-run check, not a unit-test dependency.

16. **MUST** emit OTel metrics: `eval_assessment_reads_total{relationship}` (counter; relationship ∈ founder | manager | self), `eval_assessment_reviews_total{action}` (counter; action ∈ approved | overridden), and `eval_assessment_responses_total` (counter). The read counter by relationship makes "who is reading evaluations" observable, supporting the governance posture.

---

## §2 — Why this design (rationale for humans)

**Why server-side, deny-by-default access (DEC-2620, §1 #2 #3)?** An evaluation record is the most sensitive data about a person on the platform. The access rule cannot live in the browser, where it is advisory and bypassable; it is enforced in the eval service, and the default is deny. Exactly three relationships can read an assessment — the founder, the report's own manager, and the employee themselves — and everyone else gets a 403 with no content, not even a hint that the record exists.

**Why the access decision is TASK-EVAL-001 grant + AUTH relationship, not a new rule (DEC-2624, §1 #3)?** The governance layer already decides who may view evaluations and discloses that basis in the signed documents; AUTH already knows the manager-of relationship. Inventing a second access rule here would create two sources of truth that could drift, and would let the panel grant access the governance layer never sanctioned. The panel composes the two existing authorities; it does not add a third.

**Why the auto-score is always a marked DRAFT (DEC-2621, §1 #5)?** A number a model produced, shown plainly, reads as a verdict — and a verdict that affects pay or progression must come from a human, not a model (the brain-evaluation plan's core rule; EU AI Act Article 14). The persistent DRAFT badge and the "not yet reviewed" note make it impossible to mistake the auto-score for a decision. The draft is an input to a human's judgement, displayed as such.

**Why a human must approve or override before final (DEC-2621, §1 #6)?** Showing a draft is not enough; the system must make the human's act the thing that finalises an assessment. `approved` records that a human endorsed the draft; `overridden` records that a human replaced it with a reason. Either way a named human acted, and a service account cannot stand in for them. The model assists the decision; it never makes it.

**Why the employee response and the "not closed until they can respond" rule (DEC-2622, §1 #8)?** Transparency here is a feature, not an afterthought. An evaluation the subject cannot see or contest is surveillance; an evaluation they can read and respond to, on the record, is a fair process. Blocking `closed` until the employee has had the opportunity to respond turns "right to respond" from a slogan into an enforced state transition.

**Why audit every read, including self-views (DEC-2623, §1 #9)?** Who looks at whose evaluation is itself sensitive. Auditing every cross-person read — naming the reader, the subject, and the relationship — means a manager browsing a report's record leaves a trace, and the employee (and the founder) can later see who accessed their record. Auditing self-views too keeps the ledger complete and makes "the employee read their own assessment" a provable part of the response process.

**Why no new backend, panel-in-the-console (DEC-2624, §1 #1 #12)?** The founder's decision is one unified console with a panel per module (the TASK-APP-001 / TASK-APP-005 pattern). A separate evaluation app would duplicate the shell, the auth gate, and the CDS wiring, and split a sensitive surface across two front-ends. The access check, audit, and state machine belong in the eval service, where they are enforceable; the panel renders.

**Why the browser must not compute the score, state, or access (§1 #14)?** Each of these is security- or fairness-load-bearing. A client-side score could diverge from the service's; a client-side "approved" badge could lie about a draft; a client-side allow/deny could be bypassed by editing the page. The panel is a faithful renderer of the service's verdict on all three, never a second authority.

**Why a 403 must look different from an empty result (§1 #13)?** "You are not allowed to see this" and "there is no assessment yet" are completely different facts, and conflating them either leaks (an empty panel implying no record when access was actually denied) or confuses (a denied panel implying the person has no evaluation). The panel distinguishes denied, empty-but-allowed, and error explicitly.

**Why the read-by-relationship metric (§1 #16)?** The governance posture promises bounded access; making "who reads evaluations, in what relationship" observable lets the founder verify that promise is being kept and notice anomalies (e.g. a spike in cross-person reads) without reading the audit log row by row.

---

## §3 — API contract

### Access check (server-side, deny-by-default)

```rust
// services/eval/src/views/access.rs

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Relationship { Founder, Manager, SelfView }

/// The ONLY gate to an assessment (§1 #2 #3). Composes the TASK-EVAL-001 grant with
/// AUTH's manager-of relationship. Deny-by-default: anything not explicitly allowed
/// returns Err(Forbidden) and the caller sees no content.
pub async fn authorize_read(
    pool: &sqlx::PgPool,
    reader: &Subject,                  // from the TASK-AUTH-004 session
    subject_employee_id: uuid::Uuid,   // whose assessment is being read
) -> Result<Relationship, ViewError> {
    // 1. governance gate: may this caller view evaluations at all? (TASK-EVAL-001)
    if !eval_governance::has_view_grant(pool, reader).await? {
        return Err(ViewError::Forbidden);            // 403 eval_view_forbidden
    }
    // 2. self-view
    if reader.subject_id == subject_employee_id {
        return Ok(Relationship::SelfView);
    }
    // 3. founder (tenant owner)
    if reader.is_founder {
        return Ok(Relationship::Founder);
    }
    // 4. manager-of: only this reader's own reports (TASK-AUTH-003)
    if auth::is_manager_of(pool, reader.subject_id, subject_employee_id).await? {
        return Ok(Relationship::Manager);
    }
    Err(ViewError::Forbidden)                          // everyone else: denied, no content
}
```

### Manager read (audits the cross-person read)

```rust
// services/eval/src/views/manager.rs

pub async fn get_report_assessment(
    pool: &sqlx::PgPool,
    reader: &Subject,
    subject_employee_id: uuid::Uuid,
) -> Result<AssessmentView, ViewError> {
    let rel = access::authorize_read(pool, reader, subject_employee_id).await?;  // 403 if not allowed

    let view = load_assessment_view(pool, subject_employee_id).await?;  // rubric items + evidence + score + state

    // §1 #9: every cross-person read (and self-view) is a first-class audited event
    audit::emit(pool, "eval.assessment_read", json!({
        "reader_subject_id":   reader.subject_id,
        "subject_employee_id": subject_employee_id,
        "assessment_id":       view.assessment_id,
        "relationship":        rel,
        "trace_id":            current_trace_id(),
    })).await?;
    metrics::counter!("eval_assessment_reads_total", "relationship" => rel.as_str()).increment(1);

    Ok(view)   // carries state = draft | approved | overridden; the client renders the DRAFT badge on draft
}
```

### Human approve / override + employee response (the HITL state machine)

```rust
// services/eval/src/views/response.rs

/// Human review (§1 #6, EU AI Act Art. 14). A human reviewer endorses (approve)
/// or replaces (override, with a reason) the draft. A service account cannot review.
pub async fn review_assessment(
    tx: &mut sqlx::PgTransaction<'_>,
    reviewer: &Subject,
    assessment_id: uuid::Uuid,
    decision: ReviewDecision,          // Approve | Override { outcome, reason }
) -> Result<(), ViewError> {
    // reviewer must be the founder or the subject's manager-of, and must be human
    let subj = subject_of(tx, assessment_id).await?;
    let rel = access::authorize_read(&tx_pool(tx), reviewer, subj).await?;
    if rel == Relationship::SelfView { return Err(ViewError::Forbidden); }   // you cannot approve your own
    if is_service_account(reviewer.subject_id).await? {
        return Err(ViewError::ReviewRequiresHuman);     // 403 review_requires_human
    }
    match decision {
        ReviewDecision::Approve => {
            set_state(tx, assessment_id, "approved", reviewer.subject_id).await?;
            audit::emit_tx(tx, "eval.assessment_approved", json!({
                "assessment_id": assessment_id, "reviewer_subject_id": reviewer.subject_id })).await?;
        }
        ReviewDecision::Override { outcome, reason } => {
            require_nonempty(&reason)?;                  // override demands a reason
            set_overridden(tx, assessment_id, outcome, &reason, reviewer.subject_id).await?;
            audit::emit_tx(tx, "eval.assessment_overridden", json!({
                "assessment_id": assessment_id, "reviewer_subject_id": reviewer.subject_id,
                "override_reason": reason })).await?;
        }
    }
    Ok(())
}

/// Employee response (§1 #8). The subject — and only the subject — records a response
/// to their own assessment. An assessment is not closeable until this opportunity is met.
pub async fn record_response(
    tx: &mut sqlx::PgTransaction<'_>,
    author: &Subject,
    assessment_id: uuid::Uuid,
    body: &str,
) -> Result<(), ViewError> {
    let subj = subject_of(tx, assessment_id).await?;
    if author.subject_id != subj { return Err(ViewError::Forbidden); }   // only your own
    insert_response(tx, assessment_id, author.subject_id, body).await?;
    audit::emit_tx(tx, "eval.assessment_response_recorded", json!({
        "assessment_id": assessment_id, "author_subject_id": author.subject_id })).await?;
    Ok(())
}

/// §1 #8: close is blocked until the employee has had the opportunity to respond.
pub async fn close_assessment(
    tx: &mut sqlx::PgTransaction<'_>, closer: &Subject, assessment_id: uuid::Uuid,
) -> Result<(), ViewError> {
    if !response_opportunity_met(tx, assessment_id).await? {
        return Err(ViewError::ResponseOpportunityPending);   // 409 response_opportunity_pending
    }
    set_state(tx, assessment_id, "closed", closer.subject_id).await?;
    Ok(())
}
```

### Console panel (renders the verdict; never computes it)

```typescript
// apps/console/src/render/assessment_card.ts
// Pure render: the DRAFT badge is driven by the service's state, never by the client.
export function renderAssessmentCard(a: AssessmentView): HTMLElement {
  const card = el("div", "cds-card eval-assessment");
  if (a.state === "draft") {
    card.append(badge("DRAFT — auto-score, not yet reviewed by a human", "cds-badge-warning"));
  } else if (a.state === "approved") {
    card.append(badge("APPROVED by " + a.reviewer_display, "cds-badge-success"));
  } else if (a.state === "overridden") {
    card.append(badge("OVERRIDDEN by " + a.reviewer_display, "cds-badge-info"));
  }
  card.append(rubricMapping(a.items));        // each item: source_doc / clause_ref + evidence + per-item result
  card.append(scoreBlock(a.score, a.state));  // score is shown; on draft it is labelled provisional
  if (a.response) card.append(responseBlock(a.response));   // the employee's response, shown to both sides
  return card;
}
```

---

## §4 — Acceptance criteria

1. **Panel mounts in the TASK-APP-001 shell** — the eval panel appears in the console nav only when the session holds the TASK-EVAL-001 view grant; it reuses the shell, auth gate, and CDS (AC for §1 #1 #3).
2. **Founder can read any report** — founder requests employee X's assessment → 200 with the assessment view (AC for §1 #2).
3. **Manager can read only their own report** — manager M (manager-of X) reads X → 200; manager M reads Y (not their report) → `403 eval_view_forbidden`, no content (AC for §1 #2 #4).
4. **Peer is denied** — a peer with no relationship reads X → 403, no existence leak (AC for §1 #2).
5. **No grant → denied even for self** — a caller without the TASK-EVAL-001 grant reads their own record → 403 until governance provisions the grant (AC for §1 #3).
6. **DRAFT badge on auto-score** — a `state='draft'` assessment renders the DRAFT badge and the "not yet reviewed" note; it does not render an approved/final label (AC for §1 #5).
7. **Draft is not offered as final** — the only state-changing affordance on a draft is the human approve/override control; no "mark final/decided" action exists for a draft (AC for §1 #5).
8. **Human approval finalises** — manager approves X's draft → state `approved`, `reviewer_subject_id` recorded; badge flips to APPROVED (AC for §1 #6).
9. **Override requires a reason** — override with empty reason → rejected; with a reason → state `overridden`, reason stored + audited (AC for §1 #6).
10. **Service account cannot review** — `review_assessment` with a service-account reviewer → `403 review_requires_human` (AC for §1 #6).
11. **Self cannot approve own** — the subject calling `review_assessment` on their own assessment → 403 (AC for §1 #6).
12. **Employee self-view shows own record only** — employee X's self-view returns X's assessment; it exposes no other person's data (AC for §1 #7).
13. **Employee can respond** — X posts a response → stored against the assessment, shown in both the self-view and the manager view (AC for §1 #8).
14. **Close blocked before response opportunity** — close before X has responded (and no waiver event) → `409 response_opportunity_pending`; after a response or an audited waiver → close succeeds (AC for §1 #8).
15. **Cross-person read audited** — founder/manager opening X's assessment emits `eval.assessment_read` naming reader, subject, relationship; chained into `l1_audit_log` (AC for §1 #9).
16. **Self-view audited** — X opening own assessment emits `eval.assessment_read` with reader == subject (AC for §1 #9).
17. **Approve/override/response audited** — each emits its matching `eval.assessment_*` row; override row carries `override_reason` (AC for §1 #10).
18. **Tenant isolation** — tenant A's assessments are unreachable from a tenant B session (AC for §1 #11).
19. **No new backend** — the panel's `api/eval.ts` calls only eval service routes; review of the client against the eval route list finds no call to a route that does not exist (AC for §1 #12).
20. **403 distinct from empty distinct from error** — denied shows access-denied; an allowed read of someone with no assessment shows "no assessment yet"; a 500 shows an error; none is fabricated (AC for §1 #13).
21. **Browser does not recompute** — the rendered score and state come verbatim from the service; a client-tampered "approved" label cannot exist for a `draft` because the badge is driven by `state` (AC for §1 #14).
22. **Render units pass without backend** — the DRAFT-vs-APPROVED badge and the access-denied state render correctly against fixtures (AC for §1 #15).
23. **OTel read-by-relationship metric** — a founder read increments `eval_assessment_reads_total{relationship="founder"}`; a self read increments `{relationship="self"}` (AC for §1 #16).

---

## §5 — Verification

```rust
#[tokio::test]
async fn manager_reads_own_report_but_not_others() {
    let env = EvalTestEnv::new().await;
    let (m, x, y) = env.manager_with_report_and_a_stranger().await;  // m manages x, not y
    assert!(env.as_subject(m).get_assessment(x).await.is_ok());

    let denied = env.as_subject(m).get_assessment(y).await;
    assert_eq!(denied.status_code(), 403);
    assert_eq!(denied.error_code(), "eval_view_forbidden");
    assert!(denied.body_is_empty_of_content());     // no existence leak
}

#[tokio::test]
async fn no_eval_grant_denies_even_self_view() {
    let env = EvalTestEnv::new().await;
    let x = env.employee_without_view_grant().await;
    let res = env.as_subject(x).get_assessment(x).await;
    assert_eq!(res.status_code(), 403);             // governance grant is a precondition
}

#[tokio::test]
async fn auto_score_is_marked_draft_until_human_approves() {
    let env = EvalTestEnv::new().await;
    let (m, x) = env.manager_with_drafted_assessment().await;  // TASK-EVAL-003 produced a draft
    let v = env.as_subject(m).get_assessment(x).await.unwrap();
    assert_eq!(v.state, "draft");                   // not final

    env.as_subject(m).approve(v.assessment_id).await.unwrap();
    let v2 = env.as_subject(m).get_assessment(x).await.unwrap();
    assert_eq!(v2.state, "approved");
    assert_eq!(v2.reviewer_subject_id, Some(m));
}

#[tokio::test]
async fn service_account_cannot_review() {
    let env = EvalTestEnv::new().await;
    let (_, x) = env.manager_with_drafted_assessment().await;
    let svc = env.service_account_subject();
    let err = env.as_subject(svc).approve(env.assessment_of(x).await).await.unwrap_err();
    assert_eq!(err.error_code(), "review_requires_human");
}

#[tokio::test]
async fn assessment_not_closeable_until_employee_can_respond() {
    let env = EvalTestEnv::new().await;
    let (m, x) = env.manager_with_approved_assessment().await;
    let aid = env.assessment_of(x).await;
    // no response yet, no waiver -> close blocked
    let blocked = env.as_subject(m).close(aid).await;
    assert_eq!(blocked.error_code(), "response_opportunity_pending");

    env.as_subject(x).respond(aid, "I disagree with item 3; here is context.").await.unwrap();
    assert!(env.as_subject(m).close(aid).await.is_ok());
}

#[tokio::test]
async fn every_cross_person_read_is_audited() {
    let env = EvalTestEnv::new().await;
    let (m, x) = env.manager_with_drafted_assessment().await;
    env.as_subject(m).get_assessment(x).await.unwrap();
    let row = env.audit.last_of_kind("eval.assessment_read").await.unwrap();
    assert_eq!(row["payload"]["reader_subject_id"], m.to_string());
    assert_eq!(row["payload"]["subject_employee_id"], x.to_string());
    assert_eq!(row["payload"]["relationship"], "manager");
    assert!(!row["chain_anchor"].as_str().unwrap().is_empty());   // chained
}
```

```typescript
// apps/console/tests/eval_draft_badge_render.test.ts
test("draft assessment renders the DRAFT badge and no approved label", () => {
  const card = renderAssessmentCard(fixtureAssessment({ state: "draft" }));
  expect(card.textContent).toContain("DRAFT");
  expect(card.textContent).not.toContain("APPROVED");
});
test("approved assessment renders APPROVED, not DRAFT", () => {
  const card = renderAssessmentCard(fixtureAssessment({ state: "approved", reviewer_display: "Stephen" }));
  expect(card.textContent).toContain("APPROVED");
  expect(card.textContent).not.toContain("DRAFT");
});
test("access-denied renders the denied state, not an empty panel", () => {
  const panel = renderEvalPanel(fixtureDenied());
  expect(panel.textContent).toContain("do not have access");
  expect(panel.textContent).not.toContain("no assessment");
});
```

---

## §6 — Implementation skeleton

(Access check, manager read with read-audit, the approve/override + response state machine, and the pure render above. `lib.rs` mounts the views routes behind the TASK-EVAL-001 guard; `main.ts` registers the panel in the TASK-APP-001 nav only when the view grant is present.)

---

## §7 — Dependencies

- **TASK-EVAL-003** — the evaluation engine that produces the assessments (auto-score draft, rubric mapping, linked evidence, the `draft` state) this panel surfaces and a human approves/overrides. Hard `depends_on`.
- **TASK-APP-001** — the CDS console shell, the auth gate, the CDS tokens, and the Caddy static-serving path this panel mounts into (the TASK-APP-005 pattern). Hard `depends_on`.
- **TASK-EVAL-001** — the governance/consent/access/retention layer; the view grant the access check consults and the disclosed basis that makes these reads lawful and non-covert.
- **TASK-EVAL-002** — the rubric the assessment maps to; the panel shows each rubric item's `source_doc`/`clause_ref` so the manager and employee see what was measured and against which clause.
- **TASK-AUTH-003 / TASK-AUTH-004** — RLS + the manager-of relationship the access check uses, and the session token the panel carries on every call.
- **TASK-APP-005** — the precedent for "one more read-only panel in the console, no new backend, CDS, tenant-scoped through the session".
- **TASK-MEMORY-123 / TASK-PROJ-008** — the hash-chained audit-row pattern the `eval.assessment_*` rows (including the read audit) follow.

---

## §8 — Example payloads

```json
{
  "kind": "eval.assessment_read",
  "payload": {
    "reader_subject_id":   "manager-...",
    "subject_employee_id": "employee-...",
    "assessment_id":       "asmt-...",
    "relationship":        "manager",
    "trace_id":            "0af..."
  }
}
```

```json
{
  "kind": "eval.assessment_overridden",
  "payload": {
    "assessment_id":      "asmt-...",
    "reviewer_subject_id":"stephen-...",
    "override_reason":    "Q2 evidence for KPI-2 was logged late; adjusting up after manual review.",
    "trace_id":           "0af..."
  }
}
```

---

## §9 — Open questions

Resolved by Stephen's 2026-06-29 decisions (access-restricted + contract-disclosed; both auto-score AND HITL with the draft clearly marked; employee right to respond; every cross-person read audited; panel in the console, no new backend). Deferred:
- A formal response window (a duration after which "opportunity to respond" is auto-satisfied with an audited waiver) versus a purely event-driven close — this task requires the opportunity but leaves the window-vs-event policy to TASK-EVAL-001 governance config; the close gate honours whichever it sets.
- An HR role distinct from the founder (the brain plan mentions HR) — folded into the founder/manager grants for now; a dedicated HR grant is an additive TASK-EVAL-001 grant, not a change to this panel's access shape.
- A diff/history view of how an assessment moved draft → approved/overridden over time, and a rubric-version diff — surfacing-only additive screens, later.
- Notifications to the employee that a draft assessment exists — additive; the audit row and the self-view exist now, a push/email nudge is a later task.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Peer reads someone's assessment | `authorize_read` deny-by-default | 403 eval_view_forbidden, no content | None — by design |
| Manager reads a non-report | `is_manager_of` false | 403, no existence leak | None — by design |
| Caller without TASK-EVAL-001 grant | `has_view_grant` false | 403 even for self-view | Governance provisions the grant |
| Auto-score shown as final | client badge driven by `state` | DRAFT badge forced on draft | None — structural |
| Service account approves | `is_service_account` guard | 403 review_requires_human | A human reviews |
| Subject approves own assessment | self-view relationship rejected | 403 | A manager/founder reviews |
| Override with no reason | `require_nonempty` | rejected | Reviewer supplies a reason |
| Close before employee can respond | `response_opportunity_met` false | 409 response_opportunity_pending | Employee responds or an audited waiver elapses |
| Cross-person read not audited | endpoint emits before returning | read blocked if audit emit fails (tx) | Retry; read is gated on the audit row |
| Self-view leaks another's data | self-view loads own subject only | only own record returned | None — by design |
| Browser recomputes the score | client renders service value only | no client score path exists | None — structural |
| Client forges an "approved" badge | badge bound to `state` from service | a `draft` cannot render approved | None — structural |
| 403 rendered as empty panel | client distinguishes denied/empty/error | access-denied state shown | None — by design |
| Empty-but-allowed read | HTTP 200, no assessment | "no assessment yet" shown | None |
| 401 expired session | TASK-APP-001 auth gate | return to sign-in | Re-authenticate |
| Upstream eval unreachable | non-2xx | error state, no fabrication | Restore eval service |
| Cross-tenant read | RLS + session scope | 0 rows / 403 | None — by design |
| New endpoint appears needed | API-layer review (guardrail) | extend eval task, not add backend | Add to the eval service's task |
| Audit emit fails mid-review | sqlx tx rollback | state change not committed | Caller retries |
| OTel exporter down | buffered then dropped | logged | Restore TASK-OBS-001 |
| Manager loses manager-of mid-session | next read re-checks relationship | 403 on next read | None — re-checked per call |
| Override outcome out of rubric range | validated against TASK-EVAL-002 scale | 422 | Reviewer picks a valid outcome |

---

## §11 — Implementation notes

- The access check (`authorize_read`) is the single gate and is deny-by-default: it returns one of three relationships or `Forbidden`, and every read path calls it first. There is no second access path and no client-side allow/deny, so the rule cannot be bypassed by editing the page.
- The access decision composes two existing authorities — the TASK-EVAL-001 view grant (may this caller see evaluations at all) and AUTH's manager-of (which specific reports a manager may see) — rather than inventing a third. This keeps a single source of truth for "who may see what" and prevents the panel from granting access governance never sanctioned.
- The DRAFT badge is bound to the service-supplied `state`, never to a client flag. A `draft` assessment cannot be made to render as `approved` from the browser, because the only thing that flips the badge is the service returning `state='approved'` after a human acted. This is the structural enforcement of "the model never decides".
- Approve vs override are both human acts that finalise; the difference is endorse vs replace. Override demands a reason and records the reviewer's adjusted outcome, so a human disagreeing with the auto-score leaves a defensible, audited record of why. A service account cannot stand in for the human reviewer.
- The "not closed until the employee can respond" rule is enforced as a state-machine precondition (`response_opportunity_met`), not a UI nicety. Either the employee recorded a response, or an explicit audited waiver/window event occurred; absent both, `close` returns 409. This turns the right to respond into an enforced gate.
- Every read — cross-person and self — emits `eval.assessment_read` before the content is returned, and the emit is on the request path (tx-gated for the review/response writes). Auditing self-views too keeps the ledger complete and makes "the employee saw their assessment" provable for the response process.
- The panel adds no backend: the access check, the audit, and the state machine live in `services/eval/`, and the console is a faithful renderer (the TASK-APP-005 discipline). A screen that seems to need a new endpoint is the signal to extend TASK-EVAL-003, not to put logic in the browser.
- The three render distinctions — denied (403), empty-but-allowed (200, no assessment), and error (other non-2xx) — are kept separate because conflating them either leaks or misleads. The audit row for a denied read is not written by this panel (the deny happens before content load), but the 403 itself is observable via the access metric.
- `eval_assessment_reads_total{relationship}` makes the governance promise auditable in aggregate: the founder can watch the balance of self vs manager vs founder reads and spot an anomaly without walking the audit log.
- This is a high-risk task under the EU AI Act framing because it surfaces an employment evaluation and is where human oversight is operationalised; see the AI Risk Assessment section. The human approve/override gate (Article 14) is the core of the task, not an add-on.
- The override-outcome validity is checked against the TASK-EVAL-002 rubric version's scale, so a reviewer cannot record an outcome the rubric does not define; this keeps the human's adjustment inside the same standard the draft was scored against.
- HR-as-a-distinct-role is deferred: the brain plan mentions HR, but for the first release the founder/manager grants cover it, and a dedicated HR grant is an additive TASK-EVAL-001 grant rather than a change to this panel's three-relationship access shape.

---

## AI Risk Assessment

### Risk classification

High-risk under the EU AI Act framing: the panel surfaces an automated employment-evaluation output that can inform pay and progression, and it is the surface where human oversight of that output is exercised. Human oversight (Article 14) is the core of this task, not a peripheral control.

### Data sources

The panel reads only TASK-EVAL-003 assessments for the people the caller is authorised to see — the founder for any report, a manager for their own reports, and the employee for their own record. It reads through the TASK-AUTH-004 session, tenant-scoped by RLS, and it stores only the employee's response and the reviewer's approve/override decision. It computes nothing about a person itself; the score and state come from the eval service.

### Human oversight

The TASK-EVAL-003 auto-score is always shown as a clearly-marked DRAFT and can never read as final (clause 5). A human reviewer — the founder or the subject's manager, never a service account and never the subject for their own — must approve or override before an assessment is final (clause 6, EU AI Act Article 14); override demands a written reason. The model assists the decision and never makes it. The employee can see their own record and record a response, and the assessment is not closed until they have had the opportunity to (clause 8).

### Traceability

Every read of an assessment, cross-person and self, emits a hash-chained `eval.assessment_read` audit row naming who read whose record and in what relationship (clause 9); approve, override (with reason), and response each emit their own chained row (clause 10), into the same tamper-evident `l1_audit_log` as the rest of CyberOS. Who accessed an evaluation, who finalised it and how, and what the employee said are all reconstructable and provable. A read-by-relationship metric (clause 16) makes the bounded-access promise auditable in aggregate.

### Failure modes

Unauthorised access is denied by default with no content or existence leak (clause 2). A model score reading as a decision is prevented structurally — the DRAFT badge is bound to the service state and a draft has no "final" affordance (clause 5). A model or service account finalising an assessment is prevented by the human-reviewer requirement (clause 6). An evaluation the subject cannot contest is prevented by the response right and the close gate (clause 8). The browser cannot recompute or up-grade the score, state, or access (clause 14). In every case the safe state is "no one sees what they may not, and nothing is final without a named human".

---

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this task from the brain-evaluation plan (Phase 5 surfacing), Stephen's 2026-06-29 decisions, and the existing TASK-APP-001 / TASK-APP-005 console pattern and the TASK-EVAL-001..003 chain.
- Scope: full draft of this specification — the normative clauses, the access model, the DRAFT-vs-approved state machine, the employee-response gate, the read-audit requirement, and the paired AI Risk Assessment. No console or service code is written by this task; the panel and the views endpoints are built in a later session.
- Human review: Stephen reviews and approves before status moves past draft; this is a high-risk task, so the access restriction, the human-approval gate, and the read-audit need his explicit sign-off, and the paired audit plus the CAF gate validate before any implementation merges.

---

## Operating mode (founder decision, 2026-06-30): quiet / in-product-silent

In quiet mode the employee self-view is OFF by default (config `eval_employee_self_view = off`): the product surfaces evaluation data only to the founder and the responsible manager. An employee still has the right to see and respond to their own record, but that is served on request through HR and the access path in TASK-EVAL-001, not as a default in-product screen. The auto-score is never shown to the employee by default, and where it is shown to a reviewer it is a clearly-marked DRAFT requiring human approval before it is final. Access stays founder + manager-of-report (+ self on request), deny-by-default, with every cross-person read audited. The lawful basis is the signed clause in `docs/legal/data-monitoring-and-evaluation-notice.md`.

---

*End of TASK-EVAL-004.*
