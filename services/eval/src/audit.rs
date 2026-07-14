//! Governance-mutation audit (TASK-EVAL-001 clause 12). Reuse the canonical L1 chain emit so EVAL rows
//! verify under memory's reconcile, exactly like chat, obs-router, and auth do
//! (services/shared/cyberos-audit-chain::emit_genesis). We do NOT hand-roll the chain here.
//!
//! Every governance mutation appends one l1_audit_log row, tenant-scoped and subject-attributed. The
//! governance layer is therefore itself tamper-evident, exactly like the data it governs. When no audit
//! pool is configured (tests / local), the event is logged instead (best-effort), mirroring chat.

use serde_json::{json, Value};
use uuid::Uuid;

/// The clause-12 governance event kinds. Slice 1 emitted the gate / access set; slice 2 added the two
/// governance-mutation kinds its HTTP surface needs - category registration and a filed data-subject
/// request. The retention sweeper (this sub-task) adds `eval.retention_swept` (per category sweep) and
/// `eval.subject_erased` (per subject whose derived rows were erased); the consent gate adds the
/// gated-capture skip `eval.capture_gated`.
pub mod kind {
    pub const NOTICE_PUBLISHED: &str = "eval.notice_published";
    pub const ACK_RECORDED: &str = "eval.ack_recorded";
    pub const ACCESS_GRANTED: &str = "eval.access_granted";
    pub const ACCESS_REVOKED: &str = "eval.access_revoked";
    pub const RETENTION_CHANGED: &str = "eval.retention_changed";
    /// A data-category registered / updated in the registry (clause 4), per slice 2 POST /categories.
    pub const CATEGORY_REGISTERED: &str = "eval.category_registered";
    /// A subject filed a data-subject request about their own record (clause 10b), per POST /me/requests.
    pub const SUBJECT_REQUEST: &str = "eval.subject_request";
    /// A cross-subject read of evaluation data (reader != target), per clause 9.
    pub const EVALUATION_READ: &str = "eval.evaluation_read";
    /// A subject reading their OWN record (lighter weight), per clause 9.
    pub const SELF_READ: &str = "eval.self_read";
    /// One per-category retention sweep that erased >= 1 derived (L2 / brain) row (clause 6, 12). The
    /// erasure event is itself appended to the L1 chain so the *fact* of erasure is permanent even though
    /// the *content* is gone. The sweep NEVER touches `l1_audit_log` itself.
    pub const RETENTION_SWEPT: &str = "eval.retention_swept";
    /// One per subject whose derived rows were erased by a retention sweep (clause 6, 12). Records that a
    /// specific person's rebuildable projections were bounded, without ever deleting the immutable L1 record.
    pub const SUBJECT_ERASED: &str = "eval.subject_erased";
    /// A capture skipped because the subject is consent-gated (clause 3, 16). Emitted by the gate at the
    /// real skip site so even the *absence* of captured data is auditable. `reason` is `no_ack` or
    /// `stale_ack_version`.
    pub const CAPTURE_GATED: &str = "eval.capture_gated";

    // TASK-EVAL-002 rubric curation events (§1 #11, DEC-2604). Every rubric mutation chains one of these into
    // the same `l1_audit_log` as the rest of CyberOS, so the rubric's full curation history is tamper-
    // evident. `eval.rubric_edited` is reserved for the later GENIE/edit slice; this slice emits drafted,
    // published, and superseded.
    /// A rubric, version, or item drafted (framework created / version opened / item added), per
    /// `crate::rubric::authoring` + `crate::rubric::versioning::open_version`.
    pub const RUBRIC_DRAFTED: &str = "eval.rubric_drafted";
    /// A draft rubric item edited (reserved for the later edit/GENIE slice).
    pub const RUBRIC_EDITED: &str = "eval.rubric_edited";
    /// A draft version approved by a human (reserved; this slice publishes directly from draft).
    pub const RUBRIC_APPROVED: &str = "eval.rubric_approved";
    /// A version published by a human - it becomes the operative standard, per
    /// `crate::rubric::versioning::publish_version` (§1 #8 #11).
    pub const RUBRIC_PUBLISHED: &str = "eval.rubric_published";
    /// A previously-published version superseded by a newer one (its effective interval closed), per
    /// `crate::rubric::versioning::publish_version` (§1 #6 #11).
    pub const RUBRIC_SUPERSEDED: &str = "eval.rubric_superseded";
}

/// Append one governance row to `l1_audit_log` via the shared chain, scoped to `tenant`,
/// attributed to `actor`. `event_type` is one of the `kind::*` constants; `payload` is the canonical
/// JSON body. Best-effort: a failure is logged and swallowed (mirrors chat's audit contract) so the
/// governing operation still completes; the absence is itself visible in the logs.
///
/// `audit_pool` is the memory module's Postgres (`AppState::audit_pool`). When `None`, the event is
/// logged only - the convention used in tests and local single-DB runs.
pub async fn emit_governance(
    audit_pool: Option<&crate::db::Pool>,
    tenant: Uuid,
    actor: Uuid,
    event_type: &str,
    payload: Value,
) {
    let body = json!({ "event_type": event_type, "payload": payload }).to_string();
    if let Some(pool) = audit_pool {
        if let Err(e) =
            cyberos_audit_chain::emit_genesis(pool, tenant, actor, event_type, &body).await
        {
            tracing::warn!(
                target: "cyberos_eval::audit",
                event_type,
                error = %e,
                "governance audit emit failed (best-effort)"
            );
        }
        return;
    }
    tracing::info!(
        target: "cyberos_eval::audit",
        event_type = event_type,
        tenant_id = %tenant,
        actor = %actor,
        payload = %body,
        "eval governance audit event (no audit pool configured; logged only)"
    );
}

/// Convenience over [`emit_governance`] for handler code that already holds `AppState`. Mirrors
/// `cyberos_chat::audit::emit(state, ...)`.
pub async fn emit(
    state: &crate::AppState,
    tenant: Uuid,
    actor: Uuid,
    event_type: &str,
    payload: Value,
) {
    emit_governance(
        state.audit_pool.as_ref(),
        tenant,
        actor,
        event_type,
        payload,
    )
    .await
}
