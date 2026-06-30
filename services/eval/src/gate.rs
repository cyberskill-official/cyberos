//! The consent / acknowledgment gate (FR-EVAL-001 clause 3, 17). This is the precondition that makes
//! wide day-1 capture lawful: capture is *armed* the moment a person logs into the OS, but a subject is
//! *gated* (capture + evaluation BLOCKED) until their acknowledgment of the tenant's CURRENT monitoring
//! notice version is on file. FR-MEMORY-121/122 capture emitters and FR-EVAL-003 evaluation call
//! [`is_capture_allowed`] before recording or evaluating a subject and skip a gated subject.
//!
//! QUIET OPERATING MODE: the acknowledgment is normally the signed employment-document clause recorded by
//! HR (`subject_acknowledgment.ack_source = 'signed_contract'`), not an in-app click. The gate does not
//! distinguish the source - any acknowledgment row for the current version lifts the gate. What it will
//! NOT do is treat a subject with no acknowledgment row as capturable; that is the covert posture DEC-2525
//! forbids.

use uuid::Uuid;

use crate::db::{self, Pool};

/// The OTel metric name for a gated capture (clause 16: `eval_capture_gated_total{reason}`). Emitted as a
/// structured `tracing` event (this workspace's metrics path is OTel via the obs pipeline, matching
/// `cyberos_memory::interaction::emit` and `cyberos_capture::emitter` - NOT the `metrics` facade). The
/// `reason` label is [`GateReason::as_str`].
pub const METRIC_CAPTURE_GATED: &str = "eval_capture_gated_total";

/// Why a subject is gated (mirrors the OTel `reason` label, clause 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateReason {
    /// No published notice yet, or the subject has never acknowledged any notice.
    NoAck,
    /// The subject acknowledged an older version; the current notice was bumped (clause 17 re-gating).
    StaleAckVersion,
}

impl GateReason {
    /// The stable `reason` label for the `eval_capture_gated_total` counter (clause 16: `no_ack |
    /// stale_ack_version`).
    pub fn as_str(self) -> &'static str {
        match self {
            GateReason::NoAck => "no_ack",
            GateReason::StaleAckVersion => "stale_ack_version",
        }
    }
}

/// Resolve whether `subject_id` is gated under `tenant_id`. `Ok(None)` ⇒ not gated (capture allowed);
/// `Ok(Some(reason))` ⇒ gated. All queries run inside a tenant-scoped transaction so RLS confines them to
/// this tenant's notices and acknowledgments.
pub async fn gate_reason(
    pool: &Pool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<Option<GateReason>, sqlx::Error> {
    let mut tx = db::tenant_tx(pool, &tenant_id).await?;

    // The tenant's current published notice version.
    let current: Option<i32> =
        sqlx::query_scalar("SELECT version FROM monitoring_notice WHERE is_current LIMIT 1")
            .fetch_optional(&mut *tx)
            .await?;

    let Some(current_version) = current else {
        // No notice published yet ⇒ nobody can be captured ⇒ everyone gated.
        tx.commit().await?;
        return Ok(Some(GateReason::NoAck));
    };

    // The subject's latest acknowledged version (NULL if they never acknowledged any).
    let latest_ack: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(notice_version) FROM subject_acknowledgment WHERE subject_id = $1",
    )
    .bind(subject_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    let reason = match latest_ack {
        Some(v) if v >= current_version => None, // up to date ⇒ not gated
        Some(_) => Some(GateReason::StaleAckVersion), // acked, but stale (clause 17)
        None => Some(GateReason::NoAck),         // never acked
    };

    // clause 16 metric: eval_capture_gated_total{reason}. The gate firing here IS the deny - emit the OTel
    // counter (structured tracing event) on every gated resolution so a re-gating spike (a notice bump, an
    // un-onboarded cohort) is visible. Not gated ⇒ no metric (the hot allow-path stays quiet).
    if let Some(r) = reason {
        tracing::debug!(
            target: "cyberos_eval::gate",
            metric = METRIC_CAPTURE_GATED,
            reason = r.as_str(),
            tenant_id = %tenant_id,
            "capture gated: subject has not acknowledged the current monitoring notice"
        );
    }

    Ok(reason)
}

/// Record a gated-capture SKIP onto the L1 chain (clause 3: capture emitters skip a gated subject and emit
/// `eval.capture_gated` "so the skip is itself recorded"). The companion to the hot-path predicate: a caller
/// that finds [`is_capture_allowed`] false for a subject calls this to leave the auditable trace of *what*
/// would have been captured and *why* it was not. Best-effort (the same contract as `audit::emit`); the
/// underlying interaction is never blocked by it. `would_capture` names the category/event the gate blocked.
pub async fn record_capture_gated(
    audit_pool: Option<&Pool>,
    tenant_id: Uuid,
    subject_id: Uuid,
    reason: GateReason,
    would_capture: &str,
) {
    crate::audit::emit_governance(
        audit_pool,
        tenant_id,
        subject_id,
        crate::audit::kind::CAPTURE_GATED,
        serde_json::json!({
            "subject_id": subject_id,
            "reason": reason.as_str(),
            "would_capture": would_capture,
        }),
    )
    .await;
}

/// The hot-path predicate FR-MEMORY-121/122 + FR-EVAL-003 call before capturing / evaluating: `true` iff
/// the subject has acknowledged the tenant's CURRENT published notice version (i.e. is NOT gated).
pub async fn is_capture_allowed(
    pool: &Pool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<bool, sqlx::Error> {
    Ok(gate_reason(pool, tenant_id, subject_id).await?.is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_reason_labels_match_the_otel_reason_enum() {
        // clause 16: reason ∈ no_ack | stale_ack_version. The label is what the counter carries; pin it.
        assert_eq!(GateReason::NoAck.as_str(), "no_ack");
        assert_eq!(GateReason::StaleAckVersion.as_str(), "stale_ack_version");
    }
}
