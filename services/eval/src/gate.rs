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

/// Why a subject is gated (mirrors the planned OTel `reason` label, clause 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateReason {
    /// No published notice yet, or the subject has never acknowledged any notice.
    NoAck,
    /// The subject acknowledged an older version; the current notice was bumped (clause 17 re-gating).
    StaleAckVersion,
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

    Ok(match latest_ack {
        Some(v) if v >= current_version => None, // up to date ⇒ not gated
        Some(_) => Some(GateReason::StaleAckVersion), // acked, but stale (clause 17)
        None => Some(GateReason::NoAck),         // never acked
    })
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
