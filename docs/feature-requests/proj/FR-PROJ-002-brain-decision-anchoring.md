---
id: FR-PROJ-002
title: "BRAIN-anchored proj.decision row per Issue state change — reason + prior_chain link + cross-module references + ACL + audit-before-action"
module: PROJ
priority: MUST
status: accepted
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CPO)
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-PROJ-001, FR-AI-003, FR-BRAIN-106, FR-BRAIN-108]
depends_on: [FR-PROJ-001, FR-AI-003]
blocks: [FR-PROJ-003, FR-PROJ-004, FR-PROJ-013, FR-PROJ-014, FR-PROJ-015, FR-PROJ-016, FR-TIME-004]

source_pages:
  - website/docs/modules/proj.html#brain-anchoring
source_decisions:
  - DEC-215 (audit-before-action; BRAIN failure rolls back DB mutation)
  - DEC-216 (per-issue chain via prior_decision_chain field; per-issue history queryable)
  - DEC-217 (sync_class: shareable default for engagement members; ACL allow-list for restricted)

language: rust 1.81
service: cyberos/services/proj/
new_files:
  - services/proj/src/decisions.rs
  - services/proj/src/decision_chain.rs
  - services/proj/src/decision_acl.rs
  - services/proj/tests/decisions_test.rs
  - services/proj/tests/decision_chain_test.rs
  - services/proj/tests/decision_acl_test.rs
modified_files:
  - services/proj/src/handlers/issues.rs                 # call decisions::emit on status change
  - services/proj/src/audit.rs                            # extend with decision builder
allowed_tools:
  - file_read: services/proj/**
  - file_write: services/proj/**
  - bash: cd services/proj && cargo test decision
disallowed_tools:
  - mutate issue without emitting decision row (per §1 #1)
  - bypass audit-before-action invariant (per §1 #6)
  - skip prior_decision_chain link (per §1 #5 — breaks per-issue history)
  - allow reason > 500 chars (per §1 #3)

effort_hours: 7
sub_tasks:
  - "0.5h: decisions.rs — DecisionRow struct + emit function"
  - "0.5h: decision_chain.rs — fetch prior decision's chain hash"
  - "0.5h: decision_acl.rs — ACL evaluation (engagement members default)"
  - "0.5h: handlers/issues.rs integration — call emit on status change"
  - "0.5h: Reason field validation (≤ 500 chars)"
  - "0.5h: Cross-module link extraction (chat://, email:// URLs in reason text)"
  - "0.5h: Audit-before-action transaction wrapper"
  - "0.5h: sync_class default + ACL configuration"
  - "0.5h: BRAIN row builder for proj.decision kind"
  - "0.5h: OTel metrics emission"
  - "1.5h: Tests — emit + reason + prior_chain + ACL + cross-module links + rollback on BRAIN failure"
  - "0.5h: BRAIN search integration (FR-BRAIN-108) — query decisions by issue_id"
risk_if_skipped: "Issue state changes are silent — chain has no per-decision rationale. Audit reviews can't answer 'why was this done done?'. Without prior_chain, per-issue history requires expensive join. Without audit-before-action, partial states (DB updated but BRAIN not) corrupt the chain."
---

## §1 — Description (BCP-14 normative)

Every PROJ Issue state change **MUST** be a BRAIN-anchored decision, not just a DB row update. The integration:

1. **MUST** emit BRAIN row `proj.decision` per Issue status transition (FR-PROJ-001 §1 #6's `proj.issue_status_changed` is REPLACED by this richer row). Payload: `issue_id`, `from_status`, `to_status`, `reason`, `decided_by_subject_id`, `prior_decision_chain`, `cross_module_links`, `tenant_id`, `request_id`.
2. **MUST** support optional `reason: string` (max 500 chars) on PATCH issue. Reason is the rationale for the transition; mandatory in some contexts (slice 3+ enforces for `done` transitions).
3. **MUST** validate `reason` length: > 500 chars → 400 with `reason_too_long`. Empty `reason` allowed (transition without explicit rationale).
4. **MUST** be referenceable from CHAT via `#issue/<id>` syntax — chat module's link expander (FR-CHAT-007) follows the link to the BRAIN decision row.
5. **MUST** include `prior_decision_chain` field referencing the chain hash of the previous `proj.decision` for the SAME issue. First transition → `null`. Subsequent transitions form a per-issue chain queryable as "show all decisions on issue X" via FR-BRAIN-108.
6. **MUST** apply audit-before-action invariant: the entire status change (DB UPDATE + BRAIN row emit) runs in a single Postgres transaction; BRAIN emit failure rolls back the DB UPDATE. Partial states forbidden.
7. **MUST** default `sync_class: shareable` with `acl: []` (engagement members access via tenant scope). Tenant-admin can override `acl` to restrict (e.g., HR-confidential decisions on `acl: ["@hr-team"]`).
8. **MUST** extract cross-module links from `reason` text: URLs matching `chat://thread/...`, `email://thread/...`, `meeting://...` patterns are parsed and stored as `cross_module_links` array. CHAT/email/meeting modules can resolve these links bidirectionally.
9. **MUST** be queryable via FR-BRAIN-108 — searching for `kind:proj.decision AND issue_id:<id>` returns the chronological per-issue chain.
10. **MUST** include `decided_by_subject_id` from JWT claims. Audit shows WHO made the decision, not just that it happened.
11. **MUST** complete decision emission within 100ms p95 (BRAIN row write + chain fetch + DB update).
12. **SHOULD** emit OTel metrics:
    - `proj_decisions_emitted_total{tenant_id, transition}` (counter; transition like `triage_to_todo`).
    - `proj_decisions_emit_latency_ms` (histogram).
    - `proj_decisions_with_reason_total` (counter; trend who provides rationale).
    - `proj_decisions_with_links_total{link_type}` (counter; cross-module integration health).
13. **MUST** redact PII from `reason` text before BRAIN emit using FR-BRAIN-111 ruleset (same redactor as FR-CHAT-005). The redacted form is stored in the audit row; original is NOT retained. This is mandatory not optional — operators cannot disable.
14. **MUST** include a `decision_id` (UUIDv7) as a top-level field in the payload, distinct from the BRAIN row's internal chain hash. Operators and APIs reference decisions by decision_id; chain hash is implementation detail.
15. **MUST** support decision retraction via separate `proj.decision_retracted` row kind (NOT mutation of the original). Retraction carries `{retracts_decision_id, retraction_reason, retracted_by_subject_id}`. The original decision row is immutable; the retraction is the supersession marker.
16. **MUST** treat a bulk-status-change (multiple issues PATCHed in one request) as ONE BRAIN row per issue, NOT one aggregate row. The audit trail must be per-issue queryable.
17. **MUST** normalise cross-module links: `chat://thread/abc` and `chat://Thread/ABC` resolve to the same canonical form (lowercase + trim). Stored in canonical form to enable bidirectional lookup.
18. **MUST** support an optional `decision_attributes` JSONB field on the payload for structured metadata: `{client_facing: true, requires_followup: false, deadline_iso: "2026-06-01"}`. Schema is open (per-tenant extension); reserved keys are `client_facing`, `requires_followup`, `deadline_iso`.
19. **MUST** index `proj.decision` rows by `(tenant_id, issue_id, ts_ns)` in the BRAIN search index (FR-BRAIN-108). The per-issue chain query MUST return in O(log n) time, not O(n) scan.
20. **MUST** enforce immutability: once emitted, a decision row's payload is frozen. Any attempt to PATCH or DELETE returns 405 Method Not Allowed. Modifications happen via retraction + new decision.
21. **MUST** support per-tenant required-reason policy: `cyberos_proj_tenant_settings.require_reason_for = ["done"]` (array of statuses). When the target status is in the list, missing/empty reason → 400 `reason_required`. Default = empty array (no requirement); slice 3+ defaults to `["done", "cancelled"]`.
22. **MUST** include `chain_anchor_in_payload`: the chain hash of THIS row, included in the payload itself so the row is self-verifying. Recipient SHA-256s the payload-minus-chain_anchor and compares.
23. **MUST** track `decision_session_id`: a single user PATCH that changes status + reason + assignee in one operation generates one BRAIN row for the status change but they share a session_id with other ops (e.g. comment add). Allows reconstructing "what happened in this PATCH request" across multiple audit rows.
24. **MUST** support per-engagement decision policy override: `cyberos_proj_engagement_settings.decision_acl_default` — engagements (e.g. legal-engagement) may default to `acl: ["@legal-team"]` instead of empty.

---

## §2 — Why this design (rationale for humans)

**Why decision-not-just-update (DEC-215)?** A status change in DB is silent — no rationale captured, no chain entry. As a "decision" in BRAIN, it's queryable, auditable, search-indexable, AND linked into the per-issue history. The decision IS the truth; the DB row is the projection.

**Why audit-before-action (DEC-215)?** Partial states (DB updated but BRAIN not) corrupt the audit chain — the issue shows status=done but no decision row explains it. Single transaction with BRAIN-emit-blocks-commit eliminates the partial state.

**Why prior_decision_chain (DEC-216)?** Per-issue history queries shouldn't require expensive joins. The prior_chain field forms a linked list per issue; following it gives the full transition history in chain order without joins.

**Why cross-module link extraction (§1 #8)?** Real decisions reference contextual conversations: "marked done after Stephen approved in chat" with `chat://thread/...` link. Extracting + storing as structured field lets CHAT/email/meeting modules resolve links bidirectionally — opening the chat thread shows "this thread led to issue X being marked done."

**Why default sync_class shareable (DEC-217)?** Engagement members typically all see decisions. ACL restriction is the exception (HR-only, finance-only). Default shareable optimises for the common case.

**Why max 500-char reason (§1 #3)?** Reasons are rationale, not essays. Long-form decision documents go in BRAIN as separate `decisions/` files. 500 chars enforces conciseness.

**Why mandatory reason for `done` (slice 3+, §1 #2)?** Marking work done without rationale loses retro context. Slice 1 ships optional; slice 3+ enforces for terminal transitions.

**Why decided_by_subject_id (§1 #10)?** Audit attribution. "Stephen marked it done" vs "Alice marked it done" matters for retro discussion + accountability.

**Why 100ms emit budget (§1 #11)?** Status changes are interactive (operator clicks "Done"). 100ms is invisible; longer feels sluggish. Budget covers DB UPDATE + BRAIN write + chain fetch.

**Why FR-BRAIN-108 query integration (§1 #9)?** Decisions become first-class searchable artefacts. "Show me all decisions for engagement X" or "decisions by subject Y last quarter" — answerable directly.

**Why mandatory PII redaction (§1 #13)?** Reason text is operator-typed; it may include client names, emails, phone numbers. BRAIN audit retention is years; storing unredacted = years of accumulated PII risk. Redact at write-time, not at read-time, so the persisted form is always safe.

**Why decision_id distinct from chain_hash (§1 #14)?** Chain hashes are 64 hex chars (unwieldy in URLs); UUIDv7 is human-friendly + time-sortable. Decision_id is the public identifier; chain_hash is the cryptographic identifier.

**Why retraction as separate row (§1 #15)?** Audit trail integrity: mutating the original would break the chain hash. Retraction-as-new-row preserves the original AND records the supersession. Operators answering "what was decided then; what was decided now" see both.

**Why one row per issue in bulk (§1 #16)?** Per-issue queryability: an operator searching "all decisions on issue X" expects to find the bulk change. An aggregate row would hide individual issues' transitions.

**Why normalised cross-module links (§1 #17)?** Bidirectional lookup: CHAT module asks "which decisions reference this thread?" — works only if links are stored in canonical form. Without normalisation, `chat://Thread/ABC` and `chat://thread/abc` would miss each other.

**Why open `decision_attributes` (§1 #18)?** Per-tenant workflows have custom metadata needs (e.g. CRM-engagement decisions need `crm_account_id`); reserved keys cover the common case while leaving room.

**Why O(log n) index (§1 #19)?** Per-issue history queries are interactive (operator clicks "see history"). Linear scan over thousands of decisions = perceptible delay. Index ordering enables binary-search-style lookup.

**Why immutability (§1 #20)?** Audit chain integrity. If anyone could edit a decision after the fact, the chain hash + recipient verify.sh would diverge. Immutability is the foundation of tamper-evidence.

**Why per-tenant required-reason (§1 #21)?** Compliance-heavy tenants (legal, financial services) require rationale on every status change for SOX-like audit. SMB tenants opt out for speed. Per-tenant respects context.

**Why chain_anchor_in_payload (§1 #22)?** Audit row self-verification: a recipient with the payload alone can recompute + compare. Without it, recipient needs access to BRAIN's chain index.

**Why decision_session_id (§1 #23)?** Operator workflows are often multi-mutation ("update status + add comment + reassign"); reconstructing "what did Stephen do in that PATCH" requires correlation across rows. session_id is the correlation key.

**Why per-engagement ACL default (§1 #24)?** Engagement-level policy override: a legal-engagement defaults to `["@legal-team"]`; the engagement-creator chose that scope at engagement creation. Per-decision override still possible but rare.

---

## §3 — API contract

```rust
// services/proj/src/decisions.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct DecisionPayload {
    pub issue_id: Uuid,
    pub tenant_id: Uuid,
    pub from_status: String,
    pub to_status: String,
    pub reason: Option<String>,
    pub decided_by_subject_id: Uuid,
    pub prior_decision_chain: Option<String>,        // hex 64
    pub cross_module_links: Vec<String>,
    pub request_id: String,
}

pub async fn emit_decision_in_tx(
    tx: &mut PgTransaction<'_>,
    issue_id: Uuid, tenant_id: Uuid,
    from: IssueStatus, to: IssueStatus,
    reason: Option<&str>, decided_by: Uuid, request_id: &str,
) -> Result<(), DecisionError> {
    // §1 #3 reason validation
    if let Some(r) = reason {
        if r.len() > 500 {
            return Err(DecisionError::ReasonTooLong { actual: r.len() });
        }
    }

    // §1 #5 fetch prior chain
    let prior_chain = decision_chain::fetch_prior_chain(tx, issue_id).await?;

    // §1 #8 extract cross-module links
    let cross_module_links = reason
        .map(|r| extract_links(r))
        .unwrap_or_default();

    let payload = DecisionPayload {
        issue_id, tenant_id,
        from_status: format!("{:?}", from).to_lowercase(),
        to_status: format!("{:?}", to).to_lowercase(),
        reason: reason.map(|s| s.to_string()),
        decided_by_subject_id: decided_by,
        prior_decision_chain: prior_chain,
        cross_module_links,
        request_id: request_id.into(),
    };

    let mut row = AuditRow {
        kind: "proj.decision".into(),
        payload: serde_json::to_value(&payload).unwrap(),
        meta: meta_with_acl_default(tenant_id),
        ..Default::default()
    };

    // §1 #6 audit-before-action — emit_in_tx blocks on BRAIN write
    brain_writer::emit_in_tx(tx, row).await
        .map_err(|e| DecisionError::BrainEmitFailed(e.to_string()))?;

    metrics::decision_emitted(tenant_id, &format!("{from:?}_to_{to:?}").to_lowercase(),
                              reason.is_some(), cross_module_links.len() as u32);
    Ok(())
}

fn extract_links(text: &str) -> Vec<String> {
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?:chat|email|meeting)://[a-zA-Z0-9_/\-\.]+").unwrap()
    });
    RE.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

fn meta_with_acl_default(tenant_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "sync_class": "shareable",
        "acl": [],
    })
}

#[derive(Debug, thiserror::Error)]
pub enum DecisionError {
    #[error("reason too long: {actual} chars (max 500)")]
    ReasonTooLong { actual: usize },
    #[error("brain emit failed: {0}")]
    BrainEmitFailed(String),
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
}
```

```rust
// services/proj/src/decision_chain.rs
pub async fn fetch_prior_chain(tx: &mut PgTransaction<'_>, issue_id: Uuid) -> Result<Option<String>, DecisionError> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as(
        "SELECT chain FROM brain_outbox
         WHERE kind = 'proj.decision'
           AND payload_json::jsonb->>'issue_id' = $1
         ORDER BY created_at DESC LIMIT 1",
    ).bind(issue_id.to_string()).fetch_optional(&mut **tx).await?;
    Ok(row.map(|(chain,)| hex::encode(chain)))
}
```

### decision_acl.rs — ACL evaluation + engagement override

```rust
// services/proj/src/decision_acl.rs
use uuid::Uuid;

pub struct DecisionAcl {
    pub sync_class: SyncClass,
    pub acl:        Vec<String>,
}

pub enum SyncClass { Private, Shareable }

pub async fn resolve_decision_acl(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    engagement_id: Option<Uuid>,
    override_acl: Option<Vec<String>>,
) -> Result<DecisionAcl, sqlx::Error> {
    // Per-decision override beats engagement default.
    if let Some(acl) = override_acl {
        return Ok(DecisionAcl {
            sync_class: if acl.is_empty() { SyncClass::Shareable } else { SyncClass::Shareable },
            acl,
        });
    }
    // Engagement-level default.
    if let Some(eng_id) = engagement_id {
        let default_acl: Option<Vec<String>> = sqlx::query_scalar(
            "SELECT decision_acl_default FROM cyberos_proj_engagement_settings WHERE engagement_id = $1"
        ).bind(eng_id).fetch_optional(pool).await?;
        if let Some(acl) = default_acl {
            return Ok(DecisionAcl { sync_class: SyncClass::Shareable, acl });
        }
    }
    // Tenant fallback.
    Ok(DecisionAcl { sync_class: SyncClass::Shareable, acl: vec![] })
}
```

### Required-reason policy + retraction + immutability

```rust
// services/proj/src/decisions.rs — additions

pub async fn check_required_reason(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    to_status: IssueStatus,
    reason: Option<&str>,
) -> Result<(), DecisionError> {
    let required_for: Vec<String> = sqlx::query_scalar(
        "SELECT COALESCE(require_reason_for, '{}') FROM cyberos_proj_tenant_settings WHERE tenant_id = $1"
    ).bind(tenant_id).fetch_optional(pool).await?.unwrap_or_default();

    let target = format!("{:?}", to_status).to_lowercase();
    if required_for.contains(&target) && reason.map(str::trim).map(|s| s.is_empty()).unwrap_or(true) {
        return Err(DecisionError::ReasonRequired { for_status: target });
    }
    Ok(())
}

pub async fn emit_retraction(
    tx: &mut PgTransaction<'_>,
    retracts_decision_id: Uuid,
    tenant_id: Uuid,
    retraction_reason: &str,
    retracted_by: Uuid,
    request_id: &str,
) -> Result<(), DecisionError> {
    // Verify the original decision exists + is not already retracted.
    let original = sqlx::query!(
        "SELECT payload, chain FROM brain_outbox
         WHERE kind = 'proj.decision'
           AND payload_json::jsonb->>'decision_id' = $1
           AND tenant_id = $2"
    , retracts_decision_id.to_string(), tenant_id).fetch_optional(&mut **tx).await?;
    let Some(original) = original else {
        return Err(DecisionError::DecisionNotFound(retracts_decision_id));
    };

    let already_retracted: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM brain_outbox
             WHERE kind = 'proj.decision_retracted'
               AND payload_json::jsonb->>'retracts_decision_id' = $1)"
    ).bind(retracts_decision_id.to_string()).fetch_one(&mut **tx).await?;
    if already_retracted {
        return Err(DecisionError::AlreadyRetracted(retracts_decision_id));
    }

    let redacted_reason = pii_redactor::redact(retraction_reason);
    let payload = serde_json::json!({
        "retracts_decision_id": retracts_decision_id,
        "retracts_chain": hex::encode(original.chain),
        "retraction_reason": redacted_reason,
        "retracted_by_subject_id": retracted_by,
        "tenant_id": tenant_id,
        "request_id": request_id,
    });
    let row = AuditRow {
        kind: "proj.decision_retracted".into(),
        payload,
        meta: meta_with_acl_default(tenant_id),
        ..Default::default()
    };
    brain_writer::emit_in_tx(tx, row).await
        .map_err(|e| DecisionError::BrainEmitFailed(e.to_string()))?;
    Ok(())
}

// Immutability: explicit handler that rejects mutation attempts.
pub async fn handle_decision_mutation_attempt() -> Result<axum::http::Response<axum::body::Body>, axum::http::StatusCode> {
    Err(axum::http::StatusCode::METHOD_NOT_ALLOWED)
}
```

### Cross-module link normalisation

```rust
// services/proj/src/decisions.rs — additions
pub fn normalise_link(raw: &str) -> String {
    // Split on first :// to keep scheme intact; lowercase scheme + path; trim.
    if let Some(idx) = raw.find("://") {
        let (scheme, rest) = raw.split_at(idx);
        let rest = &rest[3..];
        format!("{}://{}", scheme.to_lowercase(), rest.trim().to_lowercase())
    } else {
        raw.trim().to_lowercase()
    }
}

pub fn extract_links_normalised(text: &str) -> Vec<String> {
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?:chat|email|meeting)://[a-zA-Z0-9_/\-\.]+").unwrap()
    });
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for m in RE.find_iter(text) {
        let canon = normalise_link(m.as_str());
        if seen.insert(canon.clone()) { out.push(canon); }
    }
    out
}
```

### Self-verifying chain anchor + decision_id

```rust
// services/proj/src/decisions.rs — additions
pub fn build_decision_payload_with_anchor(
    payload: &mut DecisionPayload,
) -> serde_json::Value {
    let decision_id = uuid::Uuid::now_v7();
    let mut value = serde_json::to_value(&*payload).unwrap();
    value["decision_id"] = serde_json::json!(decision_id);
    let chain_anchor = {
        let canon = serde_json::to_string(&value).unwrap();
        let mut h = sha2::Sha256::new();
        h.update(canon.as_bytes());
        hex::encode(h.finalize())
    };
    value["chain_anchor"] = serde_json::json!(chain_anchor);
    value
}
```

### Schema additions

```sql
-- services/proj/sql/init-decision-policies.sql
CREATE TABLE IF NOT EXISTS cyberos_proj_tenant_settings (
    tenant_id            UUID PRIMARY KEY,
    require_reason_for   TEXT[] NOT NULL DEFAULT '{}',
    redact_reason        BOOLEAN NOT NULL DEFAULT true,
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cyberos_proj_engagement_settings (
    engagement_id        UUID PRIMARY KEY,
    tenant_id            UUID NOT NULL,
    decision_acl_default TEXT[] NOT NULL DEFAULT '{}',
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on the BRAIN outbox for (tenant, issue, ts) per §1 #19.
CREATE INDEX IF NOT EXISTS idx_brain_outbox_proj_decision
    ON brain_outbox ((payload_json::jsonb->>'tenant_id'),
                     (payload_json::jsonb->>'issue_id'),
                     created_at DESC)
    WHERE kind = 'proj.decision';
```

Integration in `handlers/issues.rs::patch_issue`:

```rust
// Replace the prior `audit::emit_status_changed` call with:
if let Some(new_status) = req.status {
    if new_status != current.status {
        status_fsm::validate_transition(current.status, new_status)?;
        decisions::emit_decision_in_tx(
            &mut tx, current.id, claims.tenant_id,
            current.status, new_status,
            req.reason.as_deref(), claims.subject_id, request_id,
        ).await?;
    }
}
```

---

## §4 — Acceptance criteria

1. PATCH issue status → BRAIN row `proj.decision` emitted with full payload.
2. Reason field stored verbatim in audit row.
3. Prior decision chain links to previous transition for same issue.
4. First transition → `prior_decision_chain: null`.
5. Reason > 500 chars → 400 `reason_too_long`.
6. Cross-module links extracted from reason text (`chat://`, `email://`, `meeting://`).
7. Audit-before-action: BRAIN emit failure → DB UPDATE rolled back; status NOT changed.
8. Default `sync_class: shareable`, `acl: []`.
9. Tenant-admin can override ACL to restrict (e.g., `acl: ["@hr-team"]`) per-decision.
10. CHAT @lumi can search decisions via FR-BRAIN-108: `kind:proj.decision AND issue_id:<id>`.
11. Cross-engagement search respects ACL — auditor sees only authorised decisions.
12. p95 decision emission < 100ms.
13. `decided_by_subject_id` from JWT claims.
14. Status-change without reason still emits decision (reason: null).
15. Same status PATCH (no transition) → no decision row.
16. Per-issue chain query returns transitions in chronological order.
17. **Reason redacted before emit** — fixture: reason with email `alice@x.com` → audit row `reason` contains `<EMAIL>`; original NOT stored anywhere (AC for §1 #13).
18. **decision_id present in payload** — every emitted row has `payload.decision_id` (UUIDv7) (AC for §1 #14).
19. **Retraction emits separate row** — POST /v1/proj/decisions/<id>/retract with reason → `proj.decision_retracted` row references original via `retracts_decision_id`; original row unchanged (AC for §1 #15).
20. **Retraction of already-retracted decision rejected** — second retract → 409 `already_retracted` (AC for §1 #15).
21. **Bulk PATCH = one row per issue** — PATCH `/v1/proj/issues?ids=a,b,c` with same status → 3 BRAIN rows, one per issue (AC for §1 #16).
22. **Cross-module links normalised** — fixture: reason `chat://Thread/ABC and chat://thread/abc` → `cross_module_links: ["chat://thread/abc"]` (single deduped canonical) (AC for §1 #17).
23. **decision_attributes round-trips** — POST with `decision_attributes: {client_facing: true}` → audit row preserves verbatim (AC for §1 #18).
24. **Index supports O(log n) per-issue lookup** — EXPLAIN ANALYZE of `SELECT ... WHERE issue_id = $1 ORDER BY ts_ns DESC LIMIT 50` shows Index Scan, not Seq Scan (AC for §1 #19).
25. **PATCH /v1/proj/decisions/<id> returns 405** — attempted decision row mutation rejected (AC for §1 #20).
26. **DELETE /v1/proj/decisions/<id> returns 405** — attempted decision row deletion rejected (AC for §1 #20).
27. **Required-reason policy enforces** — set `require_reason_for=['done']`; PATCH to done without reason → 400 `reason_required`; with reason → 200 (AC for §1 #21).
28. **chain_anchor in payload self-verifies** — sha256(payload-minus-chain_anchor) == payload.chain_anchor (AC for §1 #22).
29. **decision_session_id correlates ops** — multi-mutation PATCH produces multiple BRAIN rows sharing `session_id` (AC for §1 #23).
30. **Engagement ACL default applied** — set engagement `decision_acl_default=['@legal']`; PATCH issue in that engagement → audit row meta.acl=['@legal'] (AC for §1 #24).
31. **Per-decision override beats engagement default** — same engagement but PATCH with explicit `acl: ['@stephen']` → meta.acl=['@stephen'] (AC for §1 #24).

---

## §5 — Verification

```rust
#[tokio::test]
async fn status_change_emits_decision_row() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Todo), reason: Some("ready to start".into()),
        ..Default::default()
    }, None, &claims.claims, &pool, "req").await.unwrap();

    let row = brain_test_helper::find_latest("proj.decision").unwrap();
    assert_eq!(row.payload["issue_id"], issue.id.to_string());
    assert_eq!(row.payload["from_status"], "triage");
    assert_eq!(row.payload["to_status"], "todo");
    assert_eq!(row.payload["reason"], "ready to start");
}

#[tokio::test]
async fn prior_decision_chain_links() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;

    patch_status(&issue, IssueStatus::Todo, &claims, &pool).await.unwrap();
    let first = brain_test_helper::find_latest("proj.decision").unwrap();
    assert_eq!(first.payload["prior_decision_chain"], serde_json::Value::Null);

    patch_status(&issue, IssueStatus::Doing, &claims, &pool).await.unwrap();
    let second = brain_test_helper::find_latest("proj.decision").unwrap();
    let prior_chain = second.payload["prior_decision_chain"].as_str().unwrap();
    assert_eq!(prior_chain, hex::encode(first.chain));
}

#[tokio::test]
async fn reason_over_500_chars_returns_400() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    let big_reason = "x".repeat(501);
    let err = patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Todo), reason: Some(big_reason),
        ..Default::default()
    }, None, &claims.claims, &pool, "req").await.expect_err("expected ReasonTooLong");
    assert!(matches!(err, IssueError::Decision(DecisionError::ReasonTooLong { .. })));
}

#[tokio::test]
async fn cross_module_links_extracted() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Doing).await;
    let reason = "marked done after approval in chat://thread/abc and email://thread/xyz";
    patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Done), reason: Some(reason.into()), ..Default::default()
    }, None, &claims.claims, &pool, "req").await.unwrap();

    let row = brain_test_helper::find_latest("proj.decision").unwrap();
    let links = row.payload["cross_module_links"].as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert!(links.iter().any(|v| v.as_str().unwrap().starts_with("chat://")));
    assert!(links.iter().any(|v| v.as_str().unwrap().starts_with("email://")));
}

#[tokio::test]
async fn brain_emit_failure_rolls_back_status() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    brain_test_helper::inject_emit_failure();

    let err = patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Todo), ..Default::default()
    }, None, &claims.claims, &pool, "req").await.expect_err("expected BrainEmitFailed");
    assert!(matches!(err, IssueError::Decision(DecisionError::BrainEmitFailed(_))));

    // Status NOT changed
    let after: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1").bind(issue.id).fetch_one(&pool).await.unwrap();
    assert_eq!(after.status, IssueStatus::Triage);
    brain_test_helper::clear_emit_failure();
}

#[tokio::test]
async fn no_decision_on_no_status_change() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Todo).await;
    patch_issue(issue.id, PatchIssueRequest {
        title: Some("retitled".into()), ..Default::default()
    }, None, &claims.claims, &pool, "req").await.unwrap();
    let count = brain_test_helper::count_rows_since("proj.decision", recent()).await;
    assert_eq!(count, 0);   // no status change, no decision
}

#[tokio::test]
async fn default_sync_class_and_acl() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    patch_status(&issue, IssueStatus::Todo, &claims, &pool).await.unwrap();
    let row = brain_test_helper::find_latest("proj.decision").unwrap();
    assert_eq!(row.meta["sync_class"], "shareable");
    assert_eq!(row.meta["acl"], serde_json::json!([]));
}

#[tokio::test]
async fn search_decisions_by_issue() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    patch_status(&issue, IssueStatus::Todo, &claims, &pool).await.unwrap();
    patch_status(&issue, IssueStatus::Doing, &claims, &pool).await.unwrap();
    patch_status(&issue, IssueStatus::Done, &claims, &pool).await.unwrap();

    let search_results = brain_search::search(&format!("kind:proj.decision AND issue_id:{}", issue.id), claims.tenant_id).await.unwrap();
    assert_eq!(search_results.len(), 3);
    // Verify chain links in chronological order
    for window in search_results.windows(2) {
        let earlier = &window[0];
        let later = &window[1];
        assert_eq!(later.payload["prior_decision_chain"], hex::encode(&earlier.chain));
    }
}

#[tokio::test]
async fn ac17_reason_redacted() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Todo),
        reason: Some("Email alice@cyberskill.world about this".into()),
        ..Default::default()
    }, None, &claims.claims, &pool, "req").await.unwrap();
    let row = brain_test_helper::find_latest("proj.decision").unwrap();
    let r = row.payload["reason"].as_str().unwrap();
    assert!(!r.contains("alice@cyberskill.world"));
    assert!(r.contains("<EMAIL>"));
}

#[tokio::test]
async fn ac19_retraction_emits_separate_row() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    patch_status(&issue, IssueStatus::Todo, &claims, &pool).await.unwrap();
    let original = brain_test_helper::find_latest("proj.decision").unwrap();
    let decision_id = uuid::Uuid::parse_str(original.payload["decision_id"].as_str().unwrap()).unwrap();

    retract_decision(decision_id, "wrong call; reverting", &claims).await.unwrap();
    let retraction = brain_test_helper::find_latest("proj.decision_retracted").unwrap();
    assert_eq!(retraction.payload["retracts_decision_id"], decision_id.to_string());
    assert_eq!(retraction.payload["retraction_reason"], "wrong call; reverting");

    // Original row unchanged.
    let original_again = brain_test_helper::find_by_decision_id(decision_id).unwrap();
    assert_eq!(original_again.chain, original.chain);
}

#[tokio::test]
async fn ac20_already_retracted_rejected() {
    let env = test_env_with_decision().await;
    let decision_id = env.last_decision_id();
    retract_decision(decision_id, "first retract", &env.claims).await.unwrap();
    let err = retract_decision(decision_id, "second retract", &env.claims).await.expect_err("must fail");
    assert!(matches!(err, DecisionError::AlreadyRetracted(_)));
}

#[tokio::test]
async fn ac21_bulk_patch_one_row_per_issue() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issues: Vec<_> = (0..3).map(|_| create_test_issue(&pool, &claims, IssueStatus::Todo))
        .collect::<futures::future::JoinAll<_>>().await;
    let ids: Vec<_> = issues.iter().map(|i| i.id).collect();
    bulk_patch_status(&ids, IssueStatus::Doing, "starting", &claims, &pool).await.unwrap();
    let rows = brain_test_helper::find_all_since("proj.decision", recent()).await;
    assert_eq!(rows.len(), 3);
    let mut emitted_ids: Vec<_> = rows.iter()
        .map(|r| uuid::Uuid::parse_str(r.payload["issue_id"].as_str().unwrap()).unwrap()).collect();
    emitted_ids.sort();
    let mut expected_ids = ids.clone(); expected_ids.sort();
    assert_eq!(emitted_ids, expected_ids);
}

#[rstest]
#[case("chat://Thread/ABC and chat://thread/abc",  vec!["chat://thread/abc"])]
#[case("email://thread/XYZ",                       vec!["email://thread/xyz"])]
#[case("meeting://2026-05-16-standup",             vec!["meeting://2026-05-16-standup"])]
#[case("nothing here",                             vec![])]
fn ac22_cross_module_link_normalisation(#[case] reason: &str, #[case] expected: Vec<&str>) {
    let links = extract_links_normalised(reason);
    assert_eq!(links, expected);
}

#[tokio::test]
async fn ac27_required_reason_enforced() {
    let (pool, claims) = test_setup_with_engagement().await;
    set_tenant_require_reason(&pool, claims.tenant_id, &["done"]).await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Doing).await;

    let err = patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Done), reason: None, ..Default::default()
    }, None, &claims.claims, &pool, "req").await.expect_err("expected ReasonRequired");
    assert!(matches!(err, IssueError::Decision(DecisionError::ReasonRequired { .. })));

    // With reason → succeeds.
    patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Done), reason: Some("client approved".into()),
        ..Default::default()
    }, None, &claims.claims, &pool, "req").await.unwrap();
}

#[tokio::test]
async fn ac28_chain_anchor_self_verifies() {
    let env = test_env_with_decision().await;
    let row = env.last_decision_row();
    let chain_anchor = row.payload["chain_anchor"].as_str().unwrap();
    let mut payload_minus_anchor = row.payload.clone();
    payload_minus_anchor.as_object_mut().unwrap().remove("chain_anchor");
    let canon = serde_json::to_string(&payload_minus_anchor).unwrap();
    let computed = hex::encode(sha2::Sha256::digest(canon.as_bytes()));
    assert_eq!(chain_anchor, computed);
}

#[tokio::test]
async fn ac30_engagement_acl_default_applied() {
    let env = test_env().await;
    let engagement = env.create_engagement_with_acl_default(vec!["@legal".into()]).await;
    let issue = env.create_issue_in_engagement(engagement.id, IssueStatus::Triage).await;
    patch_status(&issue, IssueStatus::Todo, &env.claims, &env.pool).await.unwrap();
    let row = brain_test_helper::find_latest("proj.decision").unwrap();
    assert_eq!(row.meta["acl"], serde_json::json!(["@legal"]));
}

#[tokio::test]
async fn ac31_per_decision_override_beats_engagement() {
    let env = test_env().await;
    let engagement = env.create_engagement_with_acl_default(vec!["@legal".into()]).await;
    let issue = env.create_issue_in_engagement(engagement.id, IssueStatus::Triage).await;
    patch_issue(issue.id, PatchIssueRequest {
        status: Some(IssueStatus::Todo),
        acl_override: Some(vec!["@stephen".into()]),
        ..Default::default()
    }, None, &env.claims.claims, &env.pool, "req").await.unwrap();
    let row = brain_test_helper::find_latest("proj.decision").unwrap();
    assert_eq!(row.meta["acl"], serde_json::json!(["@stephen"]));
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **FR-PROJ-001** — Issue schema + status FSM.
- **FR-AI-003** — brain_writer for audit row.
- **FR-BRAIN-106** — sync_class enforcement at sync boundary.
- **FR-BRAIN-108** — search-by-kind queryable.
- Crates: `regex@1`, `sqlx`, `tokio`, `serde`, `hex`.

---

## §8 — Example payloads

### Decision row

```json
{
  "kind": "proj.decision",
  "ts_ns": 1747526400000000000,
  "meta": {
    "sync_class": "shareable",
    "acl": []
  },
  "payload": {
    "issue_id": "issue-...",
    "tenant_id": "550e...",
    "from_status": "doing",
    "to_status": "review",
    "reason": "Implementation complete; awaiting Stephen review",
    "decided_by_subject_id": "subject-stephen-...",
    "prior_decision_chain": "a3f9c8d7e6b5a4f3...",
    "cross_module_links": ["chat://thread/abc-123"],
    "request_id": "req_..."
  }
}
```

### PATCH with reason

```http
PATCH /v1/proj/issues/issue-... HTTP/1.1
Content-Type: application/json
{
  "status": "review",
  "reason": "Implementation complete; see chat://thread/abc-123 for details"
}

→ 200 OK + decision row emitted
```

### Search per-issue history

```http
GET /v1/brain/search?q=kind:proj.decision%20AND%20issue_id:issue-... HTTP/1.1
→ 200 OK
{
  "items": [
    { "ts": "...", "from_status": "triage", "to_status": "todo", ... },
    { "ts": "...", "from_status": "todo", "to_status": "doing", ... },
    { "ts": "...", "from_status": "doing", "to_status": "review", ... }
  ]
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Mandatory reason for terminal transitions (slice 3+).
- Decision-quality scoring (e.g., "this decision was reverted within 7 days") — slice 5+.
- Per-decision attachments (file references) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| BRAIN unreachable | brain_writer error | tx rollback; 503 audit-before-action | Operator investigates BRAIN |
| Prior decision missing (first transition) | empty query | prior_decision_chain: null | By design |
| Reason exceeds 500 chars | length check | 400 reason_too_long | Caller shortens |
| Cross-module link malformed (e.g., `chat:/`) | regex doesn't match | Not extracted; OK | By design |
| Decision emit succeeded but DB UPDATE failed | tx rollback | Both reverted | By design |
| Same status PATCH (no transition) | check before emit | No decision row | By design |
| Tenant-admin ACL change before emit | meta override applied | Restricted decision | By design |
| Concurrent PATCH races | optimistic lock at FR-PROJ-001 | First wins; second 412 | By design |
| Decision search by issue returns out of order | brain search not chain-aware | Caller sorts by ts_ns | By design |
| Reason contains PII | not currently scrubbed | Stored verbatim | Slice 3+ adds Presidio scrub |
| Cross-tenant link reference | resolution out of scope | Caller discovers broken link | By design |
| BRAIN slow (>100ms emit) | OTel histogram | Sev-3 alarm | Investigate brain_writer |
| Concurrent decisions for same issue | independent emit | Both succeed; chain may interleave | By design |
| sync_class not respected at sync | FR-BRAIN-106 enforces | Filtered at sync boundary | By design |
| ACL too restrictive | engagement member can't see own decision | Operator review | Manual fix |
| reason field deleted after emit | row immutable | N/A | By design |
| PII redaction crash (rare) | catch_unwind | emit fails; tx rolls back; SEV-2 | Operator fixes redactor |
| Redacted reason loses semantic meaning | over-redaction | observable in audits | Operator tunes redactor |
| decision_id UUIDv7 collision (theoretical) | unique constraint | second attempt retries | None |
| Retraction of non-existent decision_id | lookup miss | 404 decision_not_found | None |
| Bulk PATCH partial failure (issue B fails midway) | tx rolls back ALL | none of A/B/C committed | Caller retries |
| Cross-module link with non-allowed scheme (e.g. ftp://) | regex doesn't match | not extracted | By design |
| Cross-module link to deleted resource (e.g. archived chat thread) | not validated at extract | dangling link | Resolver handles at read time |
| decision_attributes contains reserved key with wrong type | passed through | downstream may mishandle | Slice 3+ adds JSON schema validation |
| Index not used (planner regression) | EXPLAIN ANALYZE catches | linear scan; slow | Operator runs ANALYZE |
| Immutability bypass via direct DB write | RLS+REVOKE prevents | rejected | None |
| Required-reason policy changed mid-PATCH | check uses snapshot at start | next PATCH uses new policy | None |
| Empty reason ("") with required-reason policy | trim().is_empty() == true | rejected | None |
| chain_anchor mismatch (corruption) | self-verify fails | recipient sees verification fail | Operator investigates |
| decision_session_id missing (legacy clients) | optional field; auto-generated | rows have unique session_ids | None |
| Engagement ACL default with deleted engagement | resolve_decision_acl returns tenant fallback | shareable + [] | None |
| Per-decision ACL override with unknown subject_id | stored verbatim | resolver may not find match | Operator |
| Bulk PATCH > 1000 issues | bounded at 1000 per request | 413 | Caller batches |
| Cross-module link extraction regex catastrophic backtracking | bounded character class | None | None |
| Concurrent retraction of same decision (race) | second sees `already_retracted` | first wins | None |
| Decision row size > 64KB (huge attributes) | bounded at 64KB | 400 | Caller |

---

## §11 — Notes

- Decisions are first-class chain entries; DB row is the projection.
- Audit-before-action prevents partial state corruption.
- Per-issue chain via prior_decision_chain enables history queries without joins.
- Cross-module link extraction creates structured references usable by other modules' UIs.
- Default sync_class shareable matches engagement-member workflows.
- 500-char reason limit enforces conciseness; long-form rationales go in BRAIN as separate decisions/ files.
- Slice 3+ enforces reason for terminal transitions (`done` requires reason) via the per-tenant `require_reason_for` policy (default empty → slice 3+ defaults to `['done','cancelled']`).
- decided_by_subject_id from JWT enables retro discussion + accountability.
- 100ms emit budget is interactive-UX-compatible.
- The PII redactor for reason text reuses the FR-BRAIN-111 ruleset; updates to the ruleset apply to PROJ without code changes.
- We chose UUIDv7 for decision_id (not v4) so per-issue history queries can sort by decision_id directly (v7 is time-ordered).
- Retraction as a separate row was chosen over mutation because: (a) audit-chain integrity is non-negotiable; (b) operators expect "the original decision exists forever; supersession is a new fact"; (c) it parallels FR-CHAT-006's `chat.message_edited` pattern.
- Bulk-PATCH emits one row per issue because per-issue queryability is the primary use case; aggregate rows would force consumers to deconstruct.
- Cross-module link normalisation lower-cases the entire URL including the path. We considered case-preserving the path (some systems are case-sensitive) but the lookup-symmetry benefit outweighs the rare case-sensitive system.
- The dedup-on-extract (HashSet) prevents `chat://x and chat://x and chat://x` producing three entries.
- decision_attributes is an open-key JSONB to allow per-tenant extension without code changes; reserved keys (client_facing, requires_followup, deadline_iso) cover the common case.
- The O(log n) per-issue index requires the per-tenant partition prefix; without it, the index would degenerate to O(n).
- Immutability is enforced via Postgres GRANT/REVOKE (cyberos_app has SELECT + INSERT only on brain_outbox), not just handler code. Defense in depth.
- The required-reason policy is per-status, not per-tenant boolean, so different statuses can have different requirements.
- chain_anchor_in_payload lets recipients verify a single decision without crawling the whole chain — useful for embedding decisions in DSAR exports (FR-CHAT-012 sibling pattern).
- decision_session_id is generated client-side (or by the orchestrator FR-CUO-101) and propagated; the proj handler trusts it.
- Per-engagement ACL default eliminates per-issue boilerplate for engagement-level policies; per-decision override remains for exceptions.
- We don't store the JWT itself in the audit row (only `decided_by_subject_id`) because JWTs may contain claims that shouldn't be in long-term audit trails.
- The cross-module link extraction is best-effort: malformed links (`chat:/` with single slash) are ignored, not stored as invalid.
- The 100ms p95 emit budget covers DB UPDATE + prior_chain fetch + PII redact + BRAIN row write + chain commit. Each is ~20ms typical.
- We considered streaming WAL changes to BRAIN (like FR-CHAT-005's bridge pattern) for decisions but rejected: decisions are interactive (operator clicks); synchronous emit gives immediate feedback.

---

*End of FR-PROJ-002. Status: draft (10/10 target).*
