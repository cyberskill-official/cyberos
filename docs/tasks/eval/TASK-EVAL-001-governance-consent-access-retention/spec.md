---
id: TASK-EVAL-001
title: "governance, consent, access-control + retention layer — versioned monitoring notice + per-subject acknowledgment gate + data-category/purpose registry + per-category retention sweeper + manager/self access grants (tenant RLS + audit row per read) + data-subject rights — the Phase-0 gate every BRAIN/EVAL capture and evaluation depends on"
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-06-29T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: eval
priority: p0
# was "blocked" (not a valid status per STATUS-REFERENCE §1); deferred pending TASK-AUTH-003
status: on_hold
verify: T
phase: P0
milestone: BRAIN/EVAL · Phase 0 (governance first)
slice: 1
owner: Stephen Cheng
created: 2026-06-29
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-MEMORY-101, TASK-MEMORY-121, TASK-EVAL-002, TASK-EVAL-003, TASK-CUO-204]
depends_on: [TASK-AUTH-003]
blocks: [TASK-MEMORY-121, TASK-EVAL-002]

source_pages:
  - docs/strategy/cyberos-brain-evaluation-plan.md#doing-the-monitoring-responsibly
  - docs/strategy/cyberos-brain-evaluation-plan.md#the-plan-in-phases
source_decisions:
  - DEC-2520 (governance comes FIRST — capture + evaluation for a subject are GATED behind a recorded acknowledgment of the current per-tenant monitoring notice; wide day-1 capture is lawful only behind this gate)
  - DEC-2521 (ACCESS-RESTRICTED + CONTRACT-DISCLOSED — analysis/monitoring data is readable ONLY by the founder + designated managers (manager-of relationship) + the subject's own record; the processing basis is disclosed in the employment documents, not collected covertly)
  - DEC-2522 (scope = platform work-interactions ONLY — no keystroke logging, no screen capture, no private life; minimisation is a normative clause, not a guideline)
  - DEC-2523 (retention is bounded per data category by a sweeper job — nothing is kept forever; erasure is auditable)
  - DEC-2524 (every governance mutation AND every cross-subject read emits an l1_audit_log row — the governance layer is itself tamper-evident)
  - DEC-2525 (fully-covert / no-notice collection is OUT OF SCOPE and flagged as a legal risk for Vietnamese counsel; the disclosed notice is the boundary)
  - "2026-07-23 operator (IMP-139 Gate-1 EVAL-001 carve-out, individual confirmation): confirm ai_authorship: generated_then_reviewed and eu_ai_act_risk_class: high as-is — workplace monitoring/governance for BRAIN/EVAL is Annex-III-class high-risk AI Act work; authorship matches corpus production. Marker cleared after individual review (not bulk). Evidence: operator can ship all? + assets/unreviewed-fork-brief.md carve-out."

eu_ai_act_risk_class: high
language: rust 1.81
service: cyberos/services/eval/
new_files:
  - services/eval/migrations/0001_governance.sql
  - services/eval/src/lib.rs
  # versioned monitoring notice + acknowledgment gate
  - services/eval/src/notice/mod.rs
  # data-category + purpose + lawful-basis registry
  - services/eval/src/registry/mod.rs
  # access-grant resolution (founder + manager-of + self) + read audit
  - services/eval/src/access/mod.rs
  # per-category retention policy + erasure sweeper
  - services/eval/src/retention/mod.rs
  # data-subject self-read + rectification/objection requests
  - services/eval/src/rights/mod.rs
  # l1_audit_log emit (reuses cyberos-audit-chain::chain_anchor)
  - services/eval/src/audit.rs
  # REST surface
  - services/eval/src/handlers.rs
  - services/eval/tests/governance_gate_test.rs
  - services/eval/tests/access_control_test.rs
  - services/eval/tests/retention_sweep_test.rs
modified_files:
  # depend on shared cyberos-audit-chain
  - services/eval/Cargo.toml
  # new EVAL module index (created with this task)
  - docs/tasks/eval/README.md
allowed_tools:
  - file_read: services/eval/**
  - file_write: services/eval/{src,tests,migrations}/**
  - bash: cd services/eval && cargo test
disallowed_tools:
  - capture or evaluate a subject who has not acknowledged the current notice version (per DEC-2520 — the gate is the whole point)
  - read another subject's evaluation/monitoring data without an access grant AND without emitting a read-audit row (per DEC-2521, DEC-2524)
  - add a data category to the registry without a declared purpose + lawful_basis (per DEC-2521)
  - implement covert / no-notice collection (per DEC-2525 — out of scope, legal risk)

effort_hours: 14
subtasks:
  - "1.0h: 0001_governance.sql — eval_notice, eval_ack, eval_data_category, eval_retention_policy, eval_access_grant, eval_dsr_request tables + RLS policies (app.current_tenant_id) + REVOKE UPDATE/DELETE on append-only tables"
  - "2.0h: notice/mod.rs — publish_notice(version, body_hash) + acknowledge(subject, version) + is_gated(subject) -> bool (no current-version ack ⇒ gated)"
  - "1.5h: registry/mod.rs — register_category(category, purpose, lawful_basis, retain_days) with closed lawful_basis enum; reject category without purpose"
  - "2.0h: access/mod.rs — may_read(reader, target) = founder OR manager-of(reader,target) OR reader==target OR explicit grant; every cross-subject read emits eval.evaluation_read"
  - "2.0h: retention/mod.rs — sweeper (cron) deletes/erases L2 projections older than the category retain_days; emits eval.retention_swept + eval.subject_erased; never touches L1 chain"
  - "1.5h: rights/mod.rs — GET self record + POST rectification/objection DSR request (queued for human, not auto-applied)"
  - "1.0h: audit.rs — emit_governance_row reusing cyberos-audit-chain::chain_anchor; one row per governance mutation"
  - "1.0h: handlers.rs — REST surface (notice, ack, registry, grants, DSR, sweeper trigger) all RLS + RBAC gated"
  - "2.0h: tests — gate blocks un-acked subject; access denied without grant + read-audit emitted; sweep erases past-retention rows + leaves L1 intact; every mutation emits a chained row"
risk_if_skipped: "Without this layer, wide day-1 capture (TASK-MEMORY-121/122) and evaluation (TASK-EVAL-003) run with NO lawful basis, NO employee notice, NO access boundary, and NO retention limit — that is exactly the covert-surveillance posture DEC-2525 forbids and the single largest legal + trust risk in the whole BRAIN/EVAL workstream under Vietnam PDPD Decree 13/2023/ND-CP and Labor Code 45/2019/QH14. Without the acknowledgment gate, there is no defensible 'the employee was told' record. Without the access-grant table + read-audit, anyone with a DB connection could read a colleague's performance file unseen. Without the retention sweeper, monitoring data accumulates forever — unbounded liability. Governance is Phase 0 precisely because every later phase inherits its legality from this gate."
---

## §1 — Description (BCP-14 normative)

The governance layer **MUST** make wide, day-1 capture and downstream evaluation lawful, access-bounded, time-bounded, and tamper-evident BEFORE any interaction event is captured or any evaluation is produced. It is the precondition gate for the whole BRAIN/EVAL workstream. The contract:

1. **MUST** define a versioned per-tenant monitoring/data-processing notice. The `eval_notice` table carries `id UUID PK`, `tenant_id UUID`, `version INT`, `body_md TEXT` (the human-readable notice text), `body_sha256 TEXT` (hash of the exact text acknowledged), `lawful_basis_summary TEXT`, `published_at TIMESTAMPTZ`, `published_by UUID`, `is_current BOOL`. Exactly one row per tenant has `is_current = true`. Publishing a new version flips the prior current row to `false` in the same transaction. The notice table is APPEND-ONLY (no UPDATE/DELETE of a published version — a correction is a new version).

2. **MUST** record per-subject acknowledgment of a specific notice version. The `eval_ack` table carries `id UUID PK`, `tenant_id UUID`, `subject_id UUID`, `notice_version INT`, `notice_sha256 TEXT`, `acknowledged_at TIMESTAMPTZ`, `ack_method TEXT` (`console_click | sso_consent | hr_recorded | contract_addendum`), `UNIQUE (tenant_id, subject_id, notice_version)`. The `notice_sha256` MUST match the acknowledged notice's `body_sha256` (the subject acknowledged *that* text, not a moving target).

3. **MUST** expose `is_gated(subject_id) -> bool`: a subject is **gated** (capture and evaluation BLOCKED for that subject) whenever there is no `eval_ack` row matching the tenant's *current* notice version. This is the precondition that makes wide day-1 capture lawful: the moment a person logs into the OS, capture is armed, but it does not record that person until their acknowledgment of the current notice is on file. TASK-MEMORY-121/122 capture emitters and TASK-EVAL-003 evaluation **MUST** consult `is_gated` and skip a gated subject (emitting a `eval.capture_gated` audit row instead, so the skip is itself recorded).

4. **MUST** define a data-category + purpose + lawful-basis registry. The `eval_data_category` table carries `id UUID PK`, `tenant_id UUID`, `category TEXT` (e.g. `chat_message`, `module_usage`, `task_activity`, `signin_presence`, `document_activity`), `declared_purpose TEXT`, `lawful_basis TEXT` (closed enum: `legitimate_interest | contract_performance | legal_obligation | consent`), `retain_days INT`, `minimisation_note TEXT`, `created_at`, `created_by`. A category **MUST NOT** be registrable without a non-empty `declared_purpose` and a `lawful_basis` from the enum. Capture for a category not present in the registry is rejected (no silent collection of undeclared categories).

5. **MUST** enforce scope = platform work-interactions ONLY. The registry **MUST NOT** admit a category representing keystroke logging, screen capture, microphone/camera capture, location tracking, or private-life monitoring; such a category is rejected at `register_category` with `422 category_out_of_scope`. This encodes DEC-2522 minimisation as a normative invariant, not a guideline.

6. **MUST** define a per-category retention policy and a retention/erasure sweeper. The `eval_retention_policy` table carries `tenant_id`, `category`, `retain_days INT`, `basis TEXT`, `updated_at`, `updated_by`. A background `retention-sweep` job (cron, daily) **MUST** delete or irreversibly redact Layer-2 projections (the `l2_memory` / `l2_entity` rows and EVAL artefacts) for a category whose `ingested_at` is older than `retain_days`. Nothing is retained without a policy. The sweeper **MUST NOT** mutate or delete the Layer-1 `l1_audit_log` chain (the chain is the append-only system of record; erasure operates on the queryable projections + EVAL outputs, and an `eval.subject_erased` row is itself appended to the chain to record the erasure).

7. **MUST** enforce access control on evaluation/monitoring data. `may_read(reader_subject_id, target_subject_id) -> bool` resolves true iff ANY of: (a) the reader holds the founder/owner role (`cyberos_founder`); (b) the reader is a designated manager of the target via the `eval_access_grant` table's `manager_of` relationship; (c) `reader_subject_id == target_subject_id` (own record); (d) an explicit active grant row exists for the pair. All other reads resolve false. Enforcement is BOTH tenant RLS (`app.current_tenant_id`, per TASK-AUTH-003) AND the `may_read` check at the handler — defence in depth.

8. **MUST** define the `eval_access_grant` table: `id UUID PK`, `tenant_id UUID`, `reader_subject_id UUID`, `target_subject_id UUID`, `grant_kind TEXT` (`manager_of | hr_reviewer | explicit | self`), `granted_by UUID`, `granted_at TIMESTAMPTZ`, `revoked_at TIMESTAMPTZ NULL`, `reason TEXT`. A grant is active when `revoked_at IS NULL`. Grant and revoke are mutations that emit audit rows (clause 12).

9. **MUST** emit one `eval.evaluation_read` audit row on EVERY read of another subject's evaluation/monitoring data (i.e. every call where `reader_subject_id != target_subject_id`). The row payload carries `{reader_subject_id, target_subject_id, grant_kind_used, resource, trace_id}`. A subject reading their OWN record does not require a grant but the read is still counted (an `eval.self_read` row, lighter weight). This makes every cross-person access traceable after the fact.

10. **MUST** expose data-subject rights endpoints: (a) `GET /v1/eval/me` returns the requesting subject's own monitoring/evaluation record (governed by clause 7c, always permitted for self); (b) `POST /v1/eval/me/requests` files a rectification or objection request (`request_kind ∈ rectification | objection | erasure_request | access_export`), stored in `eval_dsr_request` and QUEUED for a human — it is NOT auto-applied. The `eval_dsr_request` table carries `id`, `tenant_id`, `subject_id`, `request_kind`, `detail TEXT`, `status TEXT` (`open | acknowledged | actioned | declined`), `created_at`, `resolved_by NULL`, `resolved_at NULL`, `resolution_note NULL`.

11. **MUST** keep a human in the loop for anything consequential. No governance endpoint, and no downstream EVAL engine consuming this layer, may auto-apply an erasure-request, an objection, or any change that affects pay, progression, or employment. Those are recorded, surfaced, and decided by a person (cross-reference TASK-EVAL-003 HITL). The sweeper's *scheduled retention* deletion (clause 6) is the only automatic erasure, and it acts only on the policy the operator configured.

12. **MUST** emit one `l1_audit_log` row for EVERY governance mutation, via `services/shared/cyberos-audit-chain`'s `chain_anchor` helper, with these kinds: `eval.notice_published`, `eval.notice_acknowledged`, `eval.category_registered`, `eval.retention_changed`, `eval.access_granted`, `eval.access_revoked`, `eval.dsr_filed`, `eval.dsr_resolved`, `eval.retention_swept`, `eval.subject_erased`. Each row is tenant-scoped and subject-attributed. The governance layer is therefore itself tamper-evident, exactly like the data it governs.

13. **MUST** state explicitly, in code comments and in this spec, that fully-covert / no-notice collection is OUT OF SCOPE (DEC-2525). The disclosed notice (clause 1) plus the acknowledgment gate (clause 3) are the boundary of what this system does. A deployment that captures without a published, acknowledged notice is a misconfiguration the gate is designed to prevent, and is a legal risk to be confirmed with Vietnamese counsel before any such mode is ever considered.

14. **MUST** RLS-enforce every table on `tenant_id = current_setting('app.current_tenant_id')::uuid` (per TASK-AUTH-003) so one tenant's notices, acknowledgments, grants, and DSR requests are invisible to another.

15. **MUST** make `eval_notice`, `eval_ack`, `eval_access_grant`, and `eval_dsr_request` APPEND-ONLY for the runtime SQL role (`cyberos_app`): `REVOKE UPDATE, DELETE`. A notice correction is a new version; an acknowledgment is never un-said; a grant is revoked by setting `revoked_at` (which is the one permitted column update, gated to the admin role) rather than deletion; a DSR resolution writes `resolved_*` columns via the admin role. Append-only is the audit-trail invariant.

16. **MUST** emit OTel metrics: `eval_capture_gated_total{reason}` (counter; reason ∈ `no_ack | stale_ack_version`), `eval_evaluation_reads_total{grant_kind}` (counter), `eval_retention_swept_rows_total{category}` (counter), `eval_dsr_requests_total{request_kind}` (counter).

17. **MUST** treat a notice-version bump as re-gating: when a tenant publishes notice version N+1, every subject whose latest ack is for version ≤ N becomes gated again until they acknowledge N+1. `eval_capture_gated_total{reason="stale_ack_version"}` increments for those subjects. This guarantees a materially changed monitoring scope is re-disclosed and re-acknowledged before capture resumes under it.

18. **MUST** provide a read-only `GET /v1/eval/governance/status` for the founder/managers that returns, per tenant: current notice version, count of acknowledged vs gated subjects, registered categories with their purpose + lawful_basis + retain_days, and open DSR request count. This is the operator's at-a-glance proof that the gate is healthy (everyone acknowledged, every category declared, retention set) before relying on any evaluation built on top.

---

## §2 — Why this design (rationale for humans)

**Why governance is Phase 0, not Phase 5.** Every later phase inherits its legality from this gate. If capture (Phase 1) ships first, the company is collecting employee work-interaction data with no notice, no lawful basis on file, no access boundary, and no retention limit — which is the covert-surveillance posture that is both the legal risk under Vietnam's PDPD (Decree 13/2023/ND-CP) and Labor Code (45/2019/QH14), and the trust risk that makes a team resent the system. Building the gate first is cheap and it unblocks everything safely. This is the plan's own sequencing decision (DEC-2520).

**Why a versioned notice with a hash (clause 1, 2).** "The employee was told" is only defensible if you can show *what* they were told and *that* they agreed to that exact text. Versioning plus `body_sha256` pins the acknowledged text: a later edit to the notice produces a new version, and the old acknowledgment still proves what was disclosed at the time. Without the hash, an operator could quietly edit the notice and the ack would silently re-point at new terms.

**Why the acknowledgment gate (clause 3) is what makes day-1 capture lawful.** Stephen's decision is wide, day-1 capture — monitoring works the moment a person logs into the OS. That breadth is only lawful behind disclosure-and-acknowledgment. So capture is *armed* immediately, but each subject is *gated* until their acknowledgment of the current notice is recorded. The gate flips per-subject, automatically, the instant the ack lands. Capture emitters skip a gated subject and record the skip (`eval.capture_gated`), so even the absence of data is auditable.

**Why a category/purpose/lawful-basis registry (clause 4, 5).** PDPD expects purpose limitation and a lawful basis. A registry forces every collected category to declare *why* it is collected and *under what basis*, and refuses undeclared categories. Clause 5 hard-codes minimisation (DEC-2522): keystroke logging, screen capture, location, and private-life categories are rejected at registration. Scope is enforced by the system, not left to good intentions.

**Why a retention sweeper that never touches Layer 1 (clause 6).** "Do not keep everything forever" is a PDPD expectation and a liability limit. But the audit chain is append-only and tamper-evident by design — deleting from it would break the very integrity property the brain relies on. The resolution: retention erases the *queryable projections* and EVAL outputs (the L2 lens), while the L1 chain stays intact, and the erasure event is itself appended to L1. You bound what is *recallable* without breaking what is *provable*. This mirrors the established Layer-1-is-source / Layer-2-is-projection split (memory's `0001_layer2.sql`).

**Why access control is both RLS and an explicit check (clause 7, 8).** Tenant RLS keeps tenants apart, but within one tenant every employee shares a tenant_id — RLS alone would let any colleague read any colleague's performance file. The `may_read` resolution (founder, manager-of, self, explicit grant) is the *intra-tenant* boundary that RLS does not provide. Doing both is defence in depth: a handler bug that forgets the check still can't cross tenants, and a connection that somehow crosses tenants still hits the `may_read` gate.

**Why a read-audit row per cross-person read (clause 9, 12).** The data being read is consequential (it can inform pay and progression). "Who looked at whose file" must be reconstructable. A subject reading their own record is normal and frequent, so it gets a lighter `eval.self_read`; a manager reading a report's file is the event that must always leave a trace.

**Why DSR requests are queued, not auto-applied (clause 10, 11).** A subject can always read their own record and can file a rectification or objection — those are data-subject rights. But auto-applying an erasure or objection could destroy evidence or silently change an evaluation; and anything affecting employment must be a human decision (the plan's human-in-the-loop principle). So rights requests are first-class and recorded, and a person resolves them.

**Why re-gate on a notice bump (clause 17).** If the monitoring scope materially changes (a new category, a new purpose), the old acknowledgment no longer covers it. Re-gating forces re-disclosure and re-acknowledgment before capture continues under the new terms. This is the difference between a living consent record and a one-time checkbox.

**Why a governance-status endpoint (clause 18).** Before anyone trusts an evaluation, they need a one-glance answer to "is the gate actually healthy?" — is everyone acknowledged, is every category declared with a basis, is retention set. It turns the governance posture into something an operator can verify, not assume.

**This is not legal advice.** The clauses encode a defensible *shape* (disclosure, purpose limitation, access control, retention, data-subject rights, human-in-the-loop), grounded in the named Vietnamese instruments, but the notice text and the precise lawful-basis selection per category MUST be confirmed by Vietnamese counsel before go-live. The system enforces the structure; counsel confirms the content.

---

## §3 — API contract

### Migration

```sql
-- services/eval/migrations/0001_governance.sql
-- EVAL Phase-0 governance: notice + ack gate, category registry, retention, access, DSR.
-- Reuses AUTH's per-tenant RLS GUC (app.current_tenant_id, TASK-AUTH-003) and the
-- L1 audit chain (services/shared/cyberos-audit-chain). L1 (l1_audit_log) is the
-- source of truth and is never mutated here; this module governs the L2 projections.

-- 1. Versioned per-tenant monitoring/data-processing notice (append-only).
CREATE TABLE eval_notice (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID NOT NULL,
    version               INT  NOT NULL,
    body_md               TEXT NOT NULL,
    body_sha256           TEXT NOT NULL,
    lawful_basis_summary  TEXT NOT NULL,
    published_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_by          UUID NOT NULL,
    is_current            BOOL NOT NULL DEFAULT TRUE,
    UNIQUE (tenant_id, version)
);
-- Exactly one current notice per tenant.
CREATE UNIQUE INDEX eval_notice_one_current
    ON eval_notice (tenant_id) WHERE is_current;

-- 2. Per-subject acknowledgment of a specific notice version (append-only).
CREATE TABLE eval_ack (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID NOT NULL,
    subject_id       UUID NOT NULL,
    notice_version   INT  NOT NULL,
    notice_sha256    TEXT NOT NULL,
    acknowledged_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ack_method       TEXT NOT NULL CHECK (ack_method IN
                     ('console_click','sso_consent','hr_recorded','contract_addendum')),
    UNIQUE (tenant_id, subject_id, notice_version)
);

-- 4. Data-category + purpose + lawful-basis registry.
CREATE TABLE eval_data_category (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID NOT NULL,
    category         TEXT NOT NULL,
    declared_purpose TEXT NOT NULL CHECK (length(declared_purpose) > 0),
    lawful_basis     TEXT NOT NULL CHECK (lawful_basis IN
                     ('legitimate_interest','contract_performance','legal_obligation','consent')),
    retain_days      INT  NOT NULL CHECK (retain_days > 0),
    minimisation_note TEXT NOT NULL DEFAULT '',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by       UUID NOT NULL,
    UNIQUE (tenant_id, category)
);

-- 6. Per-category retention policy.
CREATE TABLE eval_retention_policy (
    tenant_id   UUID NOT NULL,
    category    TEXT NOT NULL,
    retain_days INT  NOT NULL CHECK (retain_days > 0),
    basis       TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by  UUID NOT NULL,
    PRIMARY KEY (tenant_id, category)
);

-- 8. Access grants (append-only; revoke = set revoked_at via admin role).
CREATE TABLE eval_access_grant (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id          UUID NOT NULL,
    reader_subject_id  UUID NOT NULL,
    target_subject_id  UUID NOT NULL,
    grant_kind         TEXT NOT NULL CHECK (grant_kind IN
                       ('manager_of','hr_reviewer','explicit','self')),
    granted_by         UUID NOT NULL,
    granted_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at         TIMESTAMPTZ,
    reason             TEXT NOT NULL DEFAULT ''
);
CREATE INDEX eval_access_grant_lookup
    ON eval_access_grant (tenant_id, reader_subject_id, target_subject_id)
    WHERE revoked_at IS NULL;

-- 10. Data-subject rights requests (resolution columns written by admin role).
CREATE TABLE eval_dsr_request (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    subject_id      UUID NOT NULL,
    request_kind    TEXT NOT NULL CHECK (request_kind IN
                    ('rectification','objection','erasure_request','access_export')),
    detail          TEXT NOT NULL DEFAULT '',
    status          TEXT NOT NULL DEFAULT 'open' CHECK (status IN
                    ('open','acknowledged','actioned','declined')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_by     UUID,
    resolved_at     TIMESTAMPTZ,
    resolution_note TEXT
);

-- 14. Tenant RLS on every table (TASK-AUTH-003 GUC).
ALTER TABLE eval_notice          ENABLE ROW LEVEL SECURITY;
ALTER TABLE eval_ack             ENABLE ROW LEVEL SECURITY;
ALTER TABLE eval_data_category   ENABLE ROW LEVEL SECURITY;
ALTER TABLE eval_retention_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE eval_access_grant    ENABLE ROW LEVEL SECURITY;
ALTER TABLE eval_dsr_request     ENABLE ROW LEVEL SECURITY;

CREATE POLICY eval_notice_tenant ON eval_notice
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
CREATE POLICY eval_ack_tenant ON eval_ack
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
CREATE POLICY eval_category_tenant ON eval_data_category
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
CREATE POLICY eval_retention_tenant ON eval_retention_policy
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
CREATE POLICY eval_grant_tenant ON eval_access_grant
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
CREATE POLICY eval_dsr_tenant ON eval_dsr_request
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 15. Append-only for the runtime role. revoked_at / resolved_* updates go via the admin role.
REVOKE UPDATE, DELETE ON eval_notice        FROM cyberos_app;
REVOKE UPDATE, DELETE ON eval_ack           FROM cyberos_app;
REVOKE        DELETE ON eval_access_grant   FROM cyberos_app;  -- UPDATE allowed only for revoked_at (admin)
REVOKE        DELETE ON eval_dsr_request    FROM cyberos_app;  -- UPDATE allowed only for resolution (admin)
```

### Rust API — the acknowledgment gate (clause 3, 17)

```rust
// services/eval/src/notice/mod.rs
use uuid::Uuid;

/// Why a subject is gated (mirrors the OTel `reason` label, clause 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateReason {
    NoAck,            // never acknowledged any notice
    StaleAckVersion,  // acknowledged an older version; current notice bumped (clause 17)
}

/// A subject is GATED (capture + evaluation BLOCKED) unless they have acknowledged
/// the tenant's CURRENT notice version. Returns None when not gated.
pub async fn gate_reason(
    pool: &sqlx::PgPool,
    subject_id: Uuid,
) -> Result<Option<GateReason>, sqlx::Error> {
    // Current notice version for this tenant (RLS scopes to the connection's tenant).
    let current: Option<i32> = sqlx::query_scalar(
        "SELECT version FROM eval_notice WHERE is_current LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let Some(current_version) = current else {
        // No notice published yet ⇒ nobody can be captured ⇒ everyone gated.
        return Ok(Some(GateReason::NoAck));
    };

    let latest_ack: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(notice_version) FROM eval_ack WHERE subject_id = $1",
    )
    .bind(subject_id)
    .fetch_one(pool)
    .await?;

    Ok(match latest_ack {
        Some(v) if v >= current_version => None,                       // up to date ⇒ not gated
        Some(_) => Some(GateReason::StaleAckVersion),                  // acked, but stale
        None => Some(GateReason::NoAck),                               // never acked
    })
}

/// The hot-path predicate TASK-MEMORY-121/122 + TASK-EVAL-003 call before capturing/evaluating.
pub async fn is_gated(pool: &sqlx::PgPool, subject_id: Uuid) -> Result<bool, sqlx::Error> {
    Ok(gate_reason(pool, subject_id).await?.is_some())
}
```

### Rust API — access resolution + read audit (clause 7, 9, 12)

```rust
// services/eval/src/access/mod.rs
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantKind { Founder, ManagerOf, Self_, Explicit }

/// Resolve whether `reader` may read `target`'s evaluation/monitoring data.
/// Returns the grant kind used (for the audit row) or None if denied.
pub async fn may_read(
    pool: &sqlx::PgPool,
    reader: Uuid,
    target: Uuid,
    reader_is_founder: bool,  // from the JWT role claim (TASK-AUTH-101 cyberos_founder)
) -> Result<Option<GrantKind>, sqlx::Error> {
    if reader == target {
        return Ok(Some(GrantKind::Self_));            // own record — clause 7c
    }
    if reader_is_founder {
        return Ok(Some(GrantKind::Founder));          // clause 7a
    }
    // Active grant (manager_of / hr_reviewer / explicit) — clause 7b/7d.
    let kind: Option<String> = sqlx::query_scalar(
        "SELECT grant_kind FROM eval_access_grant
          WHERE reader_subject_id = $1 AND target_subject_id = $2 AND revoked_at IS NULL
          ORDER BY granted_at DESC LIMIT 1",
    )
    .bind(reader)
    .bind(target)
    .fetch_optional(pool)
    .await?;

    Ok(match kind.as_deref() {
        Some("manager_of") => Some(GrantKind::ManagerOf),
        Some(_)            => Some(GrantKind::Explicit),
        None               => None,                   // denied
    })
}

/// Guard a read of another subject's data: resolve access, then ALWAYS emit a read-audit
/// row when reader != target (clause 9). Denied ⇒ Err(Forbidden), no data returned.
pub async fn guard_evaluation_read(
    pool: &sqlx::PgPool,
    reader: Uuid,
    target: Uuid,
    reader_is_founder: bool,
    resource: &str,
) -> Result<GrantKind, AccessError> {
    match may_read(pool, reader, target, reader_is_founder).await? {
        Some(GrantKind::Self_) => {
            crate::audit::emit(pool, reader, "eval.self_read",
                &serde_json::json!({ "resource": resource }).to_string()).await?;
            Ok(GrantKind::Self_)
        }
        Some(kind) => {
            crate::audit::emit(pool, reader, "eval.evaluation_read", &serde_json::json!({
                "reader_subject_id": reader, "target_subject_id": target,
                "grant_kind_used": format!("{kind:?}"), "resource": resource,
                "trace_id": crate::trace_id(),
            }).to_string()).await?;
            Ok(kind)
        }
        None => Err(AccessError::Forbidden { reader, target }),
    }
}
```

### Rust API — governance-mutation audit (clause 12) reusing the shared chain

```rust
// services/eval/src/audit.rs
// Reuse the canonical L1 chain emit so EVAL rows verify under memory's reconcile,
// exactly like obs and auth do (services/shared/cyberos-audit-chain).
use uuid::Uuid;

/// Append one governance row to l1_audit_log under the connection's tenant.
/// `kind` is one of the clause-12 eval.* kinds; `body` is the canonical JSON payload.
pub async fn emit(
    pool: &sqlx::PgPool,
    subject_id: Uuid,
    kind: &str,
    body: &str,
) -> Result<i64, sqlx::Error> {
    let tenant_id: Uuid = sqlx::query_scalar(
        "SELECT current_setting('app.current_tenant_id', true)::uuid",
    )
    .fetch_one(pool)
    .await?;
    // path namespaces the row by kind; genesis row (independent governance event).
    let path = format!("eval/{}/{}", tenant_id, kind);
    cyberos_audit_chain::emit_genesis(pool, tenant_id, subject_id, &path, body).await
}
```

### Rust API — retention sweeper (clause 6)

```rust
// services/eval/src/retention/mod.rs
/// Daily sweep: for each (tenant, category) policy, erase L2 projections older than
/// retain_days. NEVER touches l1_audit_log. Emits eval.retention_swept per category and
/// eval.subject_erased per affected subject. Returns rows erased.
pub async fn run_retention_sweep(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
    let policies: Vec<(Uuid, String, i32)> = sqlx::query_as(
        "SELECT tenant_id, category, retain_days FROM eval_retention_policy",
    )
    .fetch_all(pool)
    .await?;

    let mut total: u64 = 0;
    for (tenant_id, category, retain_days) in policies {
        // L2 projection erasure only — the L1 chain is the immutable system of record.
        let erased = sqlx::query(
            "DELETE FROM l2_memory
              WHERE tenant_id = $1
                AND frontmatter->>'eval_category' = $2
                AND ingested_at < NOW() - ($3 || ' days')::interval",
        )
        .bind(tenant_id)
        .bind(&category)
        .bind(retain_days)
        .execute(pool)
        .await?
        .rows_affected();
        total += erased;
        if erased > 0 {
            metrics::counter!("eval_retention_swept_rows_total", "category" => category.clone())
                .increment(erased);
            // The erasure itself is appended to the L1 chain (auditable erasure).
            emit_system(pool, tenant_id, "eval.retention_swept", &serde_json::json!({
                "category": category, "retain_days": retain_days, "rows_erased": erased,
            }).to_string()).await?;
        }
    }
    Ok(total)
}
```

---

## §4 — Acceptance criteria

1. **Notice publish flips current** — publishing version 2 sets version 1 `is_current=false` and version 2 `is_current=true` in one transaction; the partial unique index permits exactly one current row (AC for §1 #1).
2. **Notice is append-only** — UPDATE/DELETE on `eval_notice` by `cyberos_app` → permission denied (AC for §1 #15).
3. **Ack pins the hash** — an `eval_ack` whose `notice_sha256` != the current notice's `body_sha256` is rejected (AC for §1 #2).
4. **Un-acked subject is gated** — `is_gated(subject)` returns true when the subject has no ack for the current version; TASK-MEMORY-121 capture skips them and emits `eval.capture_gated` (AC for §1 #3).
5. **Ack lifts the gate** — after the subject acknowledges the current version, `is_gated(subject)` returns false and capture proceeds (AC for §1 #3).
6. **No notice ⇒ everyone gated** — with zero published notices, `gate_reason` returns `NoAck` for any subject (AC for §1 #3, #13).
7. **Category needs purpose + basis** — `register_category` with empty `declared_purpose` → rejected; with a `lawful_basis` outside the enum → rejected (AC for §1 #4).
8. **Out-of-scope category rejected** — registering a `keystroke` / `screen_capture` / `location` category → `422 category_out_of_scope` (AC for §1 #5).
9. **Undeclared category not captured** — a capture for a category absent from `eval_data_category` is refused (AC for §1 #4).
10. **Retention sweep erases past-retention L2 rows** — an `l2_memory` row tagged with a category, older than `retain_days`, is deleted by the sweep; a fresher row is kept (AC for §1 #6).
11. **Sweep never touches L1** — after a sweep, `l1_audit_log` row count is unchanged (only grew, by the `eval.retention_swept` / `eval.subject_erased` rows); no L1 row was deleted (AC for §1 #6, #11).
12. **Founder may read any** — `may_read(founder, anyone)` → `Founder` (AC for §1 #7a).
13. **Manager-of may read report** — with an active `manager_of` grant, `may_read(mgr, report)` → `ManagerOf` (AC for §1 #7b).
14. **Self may read self** — `may_read(x, x)` → `Self_` with no grant needed (AC for §1 #7c).
15. **Stranger denied** — `may_read(a, b)` with no grant and a not founder → None; the handler returns 403 (AC for §1 #7).
16. **Cross-person read emits audit** — a permitted manager read of a report's record emits exactly one `eval.evaluation_read` row carrying `grant_kind_used` (AC for §1 #9, #12).
17. **Self read emits lighter audit** — a self-read emits `eval.self_read`, not `eval.evaluation_read` (AC for §1 #9).
18. **Revoke removes access** — after `revoked_at` is set, `may_read` no longer returns the grant; a subsequent read is denied; an `eval.access_revoked` row was emitted (AC for §1 #8, #12).
19. **DSR is queued, not applied** — `POST /v1/eval/me/requests` with `erasure_request` creates an `open` `eval_dsr_request` row and emits `eval.dsr_filed`; no data is erased automatically (AC for §1 #10, #11).
20. **Self record always readable** — `GET /v1/eval/me` returns the caller's own record regardless of grants (AC for §1 #10).
21. **Every governance mutation chains** — publish, ack, register, retention-change, grant, revoke, dsr-file, dsr-resolve each append one `l1_audit_log` row whose `chain_anchor_hex` verifies under the shared helper (AC for §1 #12).
22. **Tenant isolation** — tenant A's notices/acks/grants/DSR are invisible to tenant B under RLS (AC for §1 #14).
23. **Notice bump re-gates** — a subject acked v1; tenant publishes v2; `gate_reason(subject)` returns `StaleAckVersion` and `eval_capture_gated_total{reason="stale_ack_version"}` increments (AC for §1 #17).
24. **Governance status reflects truth** — `GET /v1/eval/governance/status` reports current notice version, acknowledged vs gated counts, registered categories, and open DSR count consistent with the tables (AC for §1 #18).
25. **OTel counters increment** — gated capture, cross-person read, swept rows, and DSR filing each increment their counter with the right label (AC for §1 #16).

---

## §5 — Verification

```rust
#[tokio::test]
async fn unacked_subject_is_gated_then_ack_lifts_it() {
    let env = TestEnv::new().await;                // seeds tenant + subject, sets app.current_tenant_id
    publish_notice(&env.pool, 1, "v1 monitoring notice", env.founder()).await.unwrap();

    assert!(is_gated(&env.pool, env.subject()).await.unwrap());      // no ack yet
    acknowledge(&env.pool, env.subject(), 1, sha256("v1 monitoring notice"),
                AckMethod::ConsoleClick).await.unwrap();
    assert!(!is_gated(&env.pool, env.subject()).await.unwrap());     // ack lifts the gate
}

#[tokio::test]
async fn notice_bump_re_gates_until_reack() {
    let env = TestEnv::new().await;
    publish_notice(&env.pool, 1, "v1", env.founder()).await.unwrap();
    acknowledge(&env.pool, env.subject(), 1, sha256("v1"), AckMethod::ConsoleClick).await.unwrap();
    assert!(!is_gated(&env.pool, env.subject()).await.unwrap());

    publish_notice(&env.pool, 2, "v2 wider scope", env.founder()).await.unwrap();
    assert_eq!(gate_reason(&env.pool, env.subject()).await.unwrap(),
               Some(GateReason::StaleAckVersion));                   // must re-acknowledge v2
}

#[tokio::test]
async fn stranger_read_denied_manager_read_audited() {
    let env = TestEnv::new().await;
    // No grant: stranger denied.
    assert!(may_read(&env.pool, env.alice(), env.bob(), false).await.unwrap().is_none());

    // Grant manager_of, then a permitted read emits exactly one eval.evaluation_read.
    grant_access(&env.pool, env.alice(), env.bob(), GrantKind::ManagerOf, env.founder()).await.unwrap();
    let before = env.count_audit("eval.evaluation_read").await;
    let kind = guard_evaluation_read(&env.pool, env.alice(), env.bob(), false, "record").await.unwrap();
    assert_eq!(kind, GrantKind::ManagerOf);
    assert_eq!(env.count_audit("eval.evaluation_read").await, before + 1);
}

#[tokio::test]
async fn retention_sweep_erases_l2_but_never_l1() {
    let env = TestEnv::new().await;
    set_retention(&env.pool, "chat_message", 30, env.founder()).await.unwrap();
    env.seed_l2_row("chat_message", days_ago(40)).await;            // past retention
    env.seed_l2_row("chat_message", days_ago(5)).await;             // fresh
    let l1_before = env.count_l1_rows().await;

    let erased = run_retention_sweep(&env.pool).await.unwrap();
    assert_eq!(erased, 1);                                          // only the 40-day row
    assert_eq!(env.count_l2_rows("chat_message").await, 1);        // fresh row kept
    assert!(env.count_l1_rows().await >= l1_before);               // L1 only grew (sweep audit), never shrank
}

#[tokio::test]
async fn dsr_request_is_queued_not_auto_applied() {
    let env = TestEnv::new().await;
    let id = file_dsr(&env.pool, env.subject(), DsrKind::ErasureRequest, "remove my Q1 data").await.unwrap();
    let row = env.get_dsr(id).await;
    assert_eq!(row.status, "open");                                // queued for a human
    assert_eq!(env.count_l2_rows_for(env.subject()).await,
               env.l2_baseline_for(env.subject()));                // nothing erased automatically
}

#[tokio::test]
async fn out_of_scope_category_rejected() {
    let env = TestEnv::new().await;
    let err = register_category(&env.pool, "keystroke_log", "perf", LawfulBasis::LegitimateInterest, 30, env.founder())
        .await.unwrap_err();
    assert!(matches!(err, RegistryError::OutOfScope));             // clause 5 minimisation invariant
}
```

---

## §6 — Implementation skeleton

(Migration + the four core modules — notice gate, access resolution, audit emit, retention sweep — sketched in §3. `registry/mod.rs`, `rights/mod.rs`, and `handlers.rs` follow the same shape: RLS-scoped queries, closed enums via CHECK constraints, one `audit::emit` per mutation, RBAC role check from the JWT at the handler edge.)

---

## §7 — Dependencies

- **TASK-AUTH-003** — per-tenant RLS via `app.current_tenant_id` GUC + `cyberos_app` runtime role; every EVAL table inherits this tenancy + append-only REVOKE pattern. (depends_on)
- **TASK-AUTH-101** — the `cyberos_founder` role claim that `may_read` reads for the founder-can-read-any path.
- **TASK-MEMORY-101** — the L1 audit-chain writer protocol; EVAL reuses `services/shared/cyberos-audit-chain::chain_anchor` / `emit_genesis` so its rows verify under memory's reconcile.
- **`l2_memory` / `l2_entity`** (memory `0001_layer2.sql`) — the Layer-2 projections the retention sweeper erases (Layer 1 stays immutable).
- **Consumed by TASK-MEMORY-121** (interaction-event schema) and **TASK-MEMORY-122** (capture emitters): both call `is_gated` before recording a subject. (blocks)
- **Consumed by TASK-EVAL-002** (rubric) and **TASK-EVAL-003** (evaluation engine): evaluation runs only for non-gated subjects, reads are `guard_evaluation_read`-gated, and HITL for consequential outcomes is inherited from clause 11. (blocks TASK-EVAL-002)
- **TASK-CUO-204** — the GENIE/dream loop's denylist already protects PII-recall and auth invariants; EVAL governance tables join that protected set (no autonomous modification of a consent gate).

---

## §8 — Example payloads

```json
{
  "kind": "eval.notice_published",
  "path": "eval/<tenant>/eval.notice_published",
  "body": {
    "version": 2,
    "body_sha256": "9b0e8c5...",
    "lawful_basis_summary": "Decree 13/2023/ND-CP Art.13 + Labor Code 45/2019/QH14",
    "published_by": "7e57c0de-...",
    "supersedes_version": 1
  }
}
```

```json
{
  "kind": "eval.evaluation_read",
  "path": "eval/<tenant>/eval.evaluation_read",
  "body": {
    "reader_subject_id": "mgr-...",
    "target_subject_id": "report-...",
    "grant_kind_used": "ManagerOf",
    "resource": "evaluation/2026-Q2",
    "trace_id": "0af..."
  }
}
```

```json
{
  "kind": "eval.capture_gated",
  "path": "eval/<tenant>/eval.capture_gated",
  "body": { "subject_id": "newhire-...", "reason": "no_ack", "would_capture": "chat_message" }
}
```

---

## §9 — Open questions

Resolved by Stephen's 2026-06-29 decisions (DEC-2520..2525). Deferred / for counsel:

- The exact notice text and the lawful-basis selection per category MUST be confirmed by Vietnamese counsel before go-live (clause 13, §2 closing note). This task ships the structure; the content is a legal sign-off, not an engineering decision.
- Whether `manager_of` is sourced from an HR org-chart (TASK-HR-001) or set explicitly via grants — Phase 0 supports explicit grants; the HR-derived auto-grant is a Phase-5 wiring (TASK-EVAL-004) once the org chart exists.
- Cross-tenant founder visibility (a holding-company founder over multiple tenant orgs) — out of scope here; single-tenant founder only. Revisit if CyberSkill operates multiple tenants under one owner.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Capture attempted for un-acked subject | `is_gated` returns true | Capture skipped; `eval.capture_gated` row | Subject acknowledges current notice |
| Notice published with no acks yet | `gate_reason` = NoAck for all | Whole tenant gated; nothing captured | Roll out acknowledgment to staff |
| Operator edits a published notice in place | REVOKE UPDATE on `eval_notice` | Permission denied | Publish a new version instead |
| Ack submitted for a stale/edited notice hash | `notice_sha256` mismatch check | Rejected | Re-fetch current notice + re-ack |
| Stranger reads a colleague's record | `may_read` returns None | 403; no data returned | Request a grant from founder/manager |
| Cross-person read with audit emit failing | audit emit errors before data returned | Read refused (fail-closed) | Investigate audit pool; retry |
| Manager grant left active after role change | grant has no auto-expiry | Stale access | Revoke grant (sets `revoked_at`) |
| Retention sweep deletes an L1 row | impossible — sweep targets `l2_*` only | None | N/A by design |
| Retention sweep runs with no policy for a category | category not in `eval_retention_policy` | Rows retained (no policy ⇒ no deletion) | Operator sets a policy |
| Out-of-scope category slips into registry | CHECK + `register_category` 422 | Rejected at write | Use an in-scope category |
| DSR erasure auto-applied | clause 11 forbids; request only sets `open` | No auto-erase | Human resolves the request |
| Covert mode requested (no notice) | gate blocks all capture; clause 13 flags | Nothing captured; legal-risk note | Counsel review before any such mode |
| Tenant A reads tenant B governance | RLS policy | 0 rows | None — by design |
| Notice bumped, subjects not re-acked | `StaleAckVersion`; capture pauses | Capture stops for stale subjects | Re-acknowledge v(N+1) |
| Two managers granted on same target | both grants valid | Both may read; both reads audited | None — by design |
| Revoke race (read in flight as revoke commits) | next read re-checks `revoked_at` | At most one extra read, audited | None — bounded |
| `app.current_tenant_id` unset on connection | `current_setting(..., true)` is NULL → RLS yields 0 rows | Fail-closed (no data) | Middleware must set the GUC (TASK-AUTH-003) |
| Sweeper job fails mid-run | per-category loop; logged | Partial sweep; resumes next cron | Operator inspects logs |
| OTel exporter down | counters buffered then dropped | Metrics gap only | Restore TASK-OBS-001 |
| L2 row missing `eval_category` frontmatter | sweep predicate misses it | Row not swept (retained) | Backfill category tag at ingest (TASK-MEMORY-122) |
| Founder role claim spoofed | JWT signature verify (TASK-AUTH-004) | Forged token rejected upstream | None — auth boundary |
| DSR backlog grows unattended | `eval_dsr_requests_total` + status endpoint open count | Visible to operator | Staff resolves the queue |

---

## §11 — Implementation notes

- The acknowledgment gate is the load-bearing clause: `is_gated` MUST be cheap because every capture emitter calls it per subject. Cache the `(subject, current_version)` → not-gated result for a short TTL (60s) per worker, invalidated on a notice bump; the un-cached path is two indexed lookups (current notice version, subject's max acked version).
- `gate_reason` distinguishes `NoAck` from `StaleAckVersion` only for the metric label and operator clarity; both are "gated". The hot path (`is_gated`) just needs the boolean.
- EVAL reuses `services/shared/cyberos-audit-chain` rather than re-implementing chain math, so every `eval.*` row is byte-compatible with memory's reconcile invariant (the same property obs and auth rely on). Each governance event is a genesis row (`prev_hash_hex = NULL`) — governance events are independent, like obs's.
- The retention sweeper operates on Layer-2 projections (`l2_memory`, `l2_entity`) and EVAL artefacts, never on `l1_audit_log`. This is the deliberate Layer-1-is-source / Layer-2-is-projection split (memory `0001_layer2.sql`): you bound recall without breaking provability. The erasure event is appended to L1 so the *fact* of erasure is permanent even though the *content* is gone.
- `eval_category` tagging on L2 rows is written by the TASK-MEMORY-122 capture emitters (the category each event belongs to). The sweeper reads `frontmatter->>'eval_category'`; if a future schema promotes it to a typed column, the predicate moves with it.
- Append-only is enforced by `REVOKE UPDATE, DELETE` from `cyberos_app`. The two legitimate updates — `eval_access_grant.revoked_at` and `eval_dsr_request.resolved_*` — are performed by the admin role (`cyberos_ops`), which bypasses the REVOKE; the runtime app role can never mutate a published notice, an ack, or delete a grant/DSR.
- `may_read` is intentionally a pure resolution returning the grant kind, separated from `guard_evaluation_read` which adds the side-effecting audit emit. This lets read-only callers (e.g. the status endpoint counting who-can-see-whom) resolve access without emitting a read row.
- The founder path is role-based (`cyberos_founder` from the JWT, TASK-AUTH-101), not a grant row, so the founder never needs a self-issued grant to every subject. Designated managers and HR reviewers are grant rows so they are explicit, revocable, and auditable.
- DSR requests are first-class rows precisely so the data-subject-rights obligation is demonstrable: a subject *can* object/rectify, the request *is* recorded, and a *human* resolves it. Auto-applying erasure was rejected — it could destroy evidence and it removes the human checkpoint the plan requires for anything consequential.
- The notice-bump re-gate (clause 17) is what keeps consent *living*. A one-time checkbox at hire does not cover a later expansion of monitoring scope; bumping the version pauses capture for everyone whose ack is stale until they acknowledge the new terms. Operators should bump the version only on a *material* scope change, not for typo fixes (a typo fix that does not change scope can reuse the version if the hash is unchanged; if the text changes at all the hash changes, so in practice any text edit is a new version — keep notice edits deliberate).
- This task is high-risk under the EU AI Act framing (employment-context monitoring feeding AI evaluation), so it carries the `## AI Risk Assessment` section below and its paired audit verifies the governance-first invariants explicitly. Stephen's sign-off and Vietnamese counsel's confirmation of the notice text are both preconditions to go-live.
- The whole layer is deliberately boring and conservative: it adds gates, registries, audit rows, and retention limits. It does not itself analyse anyone. Analysis is TASK-EVAL-003, and it runs only inside the fence this task builds.

---

## AI Risk Assessment

### Data sources

This layer governs (it does not itself produce) the AI-fed employment analysis. The data it gates is platform work-interactions only — chat messages, module usage, task/project activity, sign-in/presence, document activity — each of which must be a *declared category* in the registry with a purpose and a lawful basis before it can be collected (clause 4). Out-of-scope sources (keystroke logging, screen capture, microphone/camera, location, private life) are rejected at registration (clause 5). No data is collected for a subject until that subject has acknowledged the current monitoring notice (clause 3). The source of truth is the tamper-evident L1 chain; this layer reads the tenant's notice/ack/grant/registry/retention state to decide what may flow into evaluation.

### Human oversight

Human oversight is built into the layer (EU AI Act Article 14). Anything consequential — an erasure request, an objection, or any downstream change to pay, progression, or employment — is recorded and decided by a person, never auto-applied (clause 11). Reads of another person's evaluation data are restricted to the founder, designated managers, and the subject's own record (clause 7) and every cross-person read leaves an audit row (clause 9). The founder/managers have a governance-status view (clause 18) to confirm the gate is healthy before trusting any evaluation. The only automatic action is scheduled retention deletion, and it acts solely on the policy the operator configured (clause 6).

### Failure modes

The safe state everywhere is "do not capture, do not disclose" reached by gating, not by guessing. A subject without a current acknowledgment is gated and not recorded (clause 3); the skip is itself audited. A notice-scope change re-gates everyone with a stale acknowledgment until they re-acknowledge (clause 17). A read without authorisation is denied and returns no data (clause 7), and the read-audit emit is fail-closed (a read that cannot be audited does not return data). An un-set tenant GUC makes RLS yield zero rows (fail-closed). The retention sweeper can only erase Layer-2 projections; it can never touch the Layer-1 chain, so erasure bounds recall without destroying the integrity record (clause 6). Covert collection is structurally prevented by the gate and explicitly flagged for counsel (clause 13).

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this task from `docs/strategy/cyberos-brain-evaluation-plan.md` and Stephen's 2026-06-29 governance decisions (DEC-2520..2525), matching the repo's engineering-spec@1 house style.
- Scope: full draft of this specification — the normative clauses, the governance schema, the access-resolution and retention-sweep sketches, the acceptance criteria, and the AI risk assessment.
- Human review: Stephen reviews and approves before status moves past draft. Because this is the Phase-0 legal/trust gate for the whole BRAIN/EVAL workstream, the notice text and per-category lawful basis additionally require Vietnamese counsel sign-off, and the paired audit plus the CAF gate validate before any implementation merges.

---

## Operating mode (founder decision, 2026-06-30): quiet / in-product-silent

CyberSkill runs this in a quiet mode: the product shows employees NO monitoring or evaluation surface by default, and access to evaluation data is restricted to the founder and the responsible manager (plus the subject's own record on request). The disclosure that gives this its lawful basis is NOT an in-app banner or click — it is the signed monitoring-and-evaluation clause in the employment documents (see `docs/legal/data-monitoring-and-evaluation-notice.md`, EN + VN). This constrains the implementation: the per-subject acknowledgment that satisfies the §1 consent gate is recorded with `ack_source = 'signed_contract'` (the HR-recorded contract acceptance + the notice version), not an in-app acknowledgment, and the in-app notice surface is off by default. The signed-clause disclosure is the non-negotiable floor — there is no mode that captures or evaluates a subject with no acknowledgment row at all. Confirm the clause and lawful basis with Vietnamese counsel before go-live (PDPD Decree 13/2023/ND-CP + Labor Code 45/2019/QH14).

---

*End of TASK-EVAL-001.*
