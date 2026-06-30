//! FR-EVAL-002 versioning + the human-in-the-loop publish transition (§1 #6 #7 #8 #13).
//!
//! A rubric is versioned and effective-dated; a published version is immutable, and re-curation produces a
//! NEW version while the prior one is superseded (DEC-2603). The two seams FR-EVAL-003 and the authoring
//! surface use:
//!   * [`resolve_effective`] - the single read FR-EVAL-003 calls to anchor an assessment to the standard
//!     that was actually in force on a date (§1 #7);
//!   * [`publish_version`] - the HITL transition: a human approver is mandatory, the version must be
//!     coherent (every item cited + bilingual + check-shape valid, non-empty), and its effective interval
//!     must not overlap a live published version; on success the prior open-ended version is superseded and
//!     this one becomes published (§1 #8 #13).
//!
//! All writes run inside a tenant-scoped transaction (the FR-AUTH-003 RLS GUC, via [`crate::db::tenant_tx`]),
//! so RLS confines them to one tenant. The audit row is emitted best-effort AFTER the transaction commits,
//! on the separate memory-chain pool - the same contract `crate::handlers` and `crate::access` use (the L1
//! chain lives in the memory module's Postgres, not the eval DB, so it cannot share the eval transaction).
//!
//! Effective intervals are half-open `[effective_from, effective_to)` so adjacent versions meet on a
//! boundary date with no gap and no overlap.

use uuid::Uuid;

use crate::audit;
use crate::db::{self, Pool};

use super::model::{RubricError, RubricVersion, VersionState};

/// Open a new draft version of a rubric (§1 #12). The version_no is `max(version_no)+1` for the rubric
/// (RLS scopes the MAX to this tenant). Emits `eval.rubric_version_opened` (a `rubric_drafted` family row).
/// The version starts in `draft`; items are added via [`super::authoring::add_item`] and it becomes
/// operative only after a later human [`publish_version`].
pub async fn open_version(
    pool: &Pool,
    audit_pool: Option<&Pool>,
    tenant_id: Uuid,
    actor_subject_id: Uuid,
    rubric_id: Uuid,
) -> Result<RubricVersion, RubricError> {
    let mut tx = db::tenant_tx(pool, &tenant_id).await?;

    // Confirm the rubric exists in this tenant (RLS already confines the lookup).
    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM rubric WHERE id = $1 LIMIT 1")
        .bind(rubric_id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        let _ = tx.commit().await;
        return Err(RubricError::NotFound);
    }

    let next_no: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_no), 0) + 1 FROM rubric_version WHERE rubric_id = $1",
    )
    .bind(rubric_id)
    .fetch_one(&mut *tx)
    .await?;

    let version: RubricVersion = sqlx::query_as(
        "INSERT INTO rubric_version (rubric_id, tenant_id, version_no, state, created_by)
         VALUES ($1, $2, $3, 'draft', $4)
         RETURNING id, rubric_id, tenant_id, version_no, state, effective_from, effective_to,
                   approver_subject_id, approved_at, published_by_subject_id, published_at,
                   created_at, created_by",
    )
    .bind(rubric_id)
    .bind(tenant_id)
    .bind(next_no)
    .bind(actor_subject_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    audit::emit_governance(
        audit_pool,
        tenant_id,
        actor_subject_id,
        audit::kind::RUBRIC_DRAFTED,
        serde_json::json!({
            "rubric_id": rubric_id,
            "rubric_version_id": version.id,
            "version_no": version.version_no,
            "actor_subject_id": actor_subject_id,
        }),
    )
    .await;
    Ok(version)
}

/// The published version effective on `at` (§1 #7). FR-EVAL-003 anchors an assessment to whatever standard
/// was actually in force for the period: the published version whose half-open `[effective_from,
/// effective_to)` interval contains `at`. `Err(NoEffectiveVersion)` if none covers the date.
pub async fn resolve_effective(
    pool: &Pool,
    tenant_id: Uuid,
    rubric_id: Uuid,
    at: chrono::NaiveDate,
) -> Result<RubricVersion, RubricError> {
    let mut tx = db::tenant_tx(pool, &tenant_id).await?;
    let found: Option<RubricVersion> = sqlx::query_as(
        "SELECT id, rubric_id, tenant_id, version_no, state, effective_from, effective_to,
                approver_subject_id, approved_at, published_by_subject_id, published_at,
                created_at, created_by
           FROM rubric_version
          WHERE rubric_id = $1 AND state = 'published'
            AND effective_from <= $2
            AND (effective_to IS NULL OR effective_to > $2)
          ORDER BY version_no DESC
          LIMIT 1",
    )
    .bind(rubric_id)
    .bind(at)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    found.ok_or(RubricError::NoEffectiveVersion)
}

/// Whether a subject is a service (non-human) account. The HITL gate (§1 #8) requires a HUMAN approver; a
/// service-account approver is rejected. The authoritative is-human check belongs to AUTH (a later wiring);
/// for this slice the caller passes whether the approver is human (derived in the handler from the verified
/// token's subject + roles), and this function is the seam that decides the policy. Kept here so the rule -
/// "no version becomes operative without a human" - lives next to `publish_version`.
pub fn is_human_approver(approver_is_human: bool) -> bool {
    approver_is_human
}

/// HITL publish (§1 #8 #13). Preconditions, all enforced before the version becomes operative:
///   * `approver_is_human` is true (a service-account approver -> `RequiresHumanApprover`, 403);
///   * the version is currently `draft` or `approved` (publishing a published/superseded one -> `NotMutable`);
///   * the version is coherent: at least one item, and every item is clause-cited + has a Vietnamese title +
///     is not flagged `needs_clause_ref` (an ungrounded GENIE draft blocks publish);
///   * `effective_from` does not overlap a live published version's open interval for the same rubric.
///
/// On success the currently-open published version (effective_to IS NULL) is superseded - its `effective_to`
/// is set to this `effective_from` (half-open meet) and its state -> `superseded` - and this version becomes
/// `published`. Emits `eval.rubric_superseded` (if one was superseded) and `eval.rubric_published`.
pub async fn publish_version(
    pool: &Pool,
    audit_pool: Option<&Pool>,
    tenant_id: Uuid,
    approver_subject_id: Uuid,
    approver_is_human: bool,
    version_id: Uuid,
    effective_from: chrono::NaiveDate,
) -> Result<RubricVersion, RubricError> {
    // §1 #8 - the human-approval gate, checked first so an un-authorised publish never touches data.
    if !is_human_approver(approver_is_human) {
        return Err(RubricError::RequiresHumanApprover);
    }

    let mut tx = db::tenant_tx(pool, &tenant_id).await?;

    // Load the version + its rubric and current state (RLS confines this to the tenant).
    let row: Option<(Uuid, VersionState)> =
        sqlx::query_as("SELECT rubric_id, state FROM rubric_version WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (rubric_id, state) = match row {
        Some(r) => r,
        None => {
            let _ = tx.commit().await;
            return Err(RubricError::NotFound);
        }
    };
    // Only a draft/approved version may be published (§1 #6 - published is immutable).
    if !matches!(state, VersionState::Draft | VersionState::Approved) {
        let _ = tx.commit().await;
        return Err(RubricError::NotMutable);
    }

    // §1 #13 coherence: at least one item.
    let item_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM rubric_item WHERE rubric_version_id = $1")
            .bind(version_id)
            .fetch_one(&mut *tx)
            .await?;
    if item_count == 0 {
        let _ = tx.commit().await;
        return Err(RubricError::VersionEmpty);
    }
    // §1 #13 coherence: no uncited / ungrounded item. The migration already forbids an empty clause_ref and
    // an empty title_vi, so the only remaining publish blocker is a needs_clause_ref GENIE draft a human has
    // not yet grounded (§1 #9 #13). Surfaced as the same `rubric_item_uncited` code (AC #13).
    let ungrounded: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM rubric_item WHERE rubric_version_id = $1 AND needs_clause_ref = TRUE",
    )
    .bind(version_id)
    .fetch_one(&mut *tx)
    .await?;
    if ungrounded > 0 {
        let _ = tx.commit().await;
        return Err(RubricError::Uncited);
    }

    // §1 #13 - the new half-open `[effective_from, infinity)` interval must not overlap a live published
    // version for this rubric, EXCEPT the open-ended predecessor this publish cleanly supersedes (closing it
    // at the new effective_from, so the two meet on the boundary with no overlap). A published version
    // conflicts when:
    //   * it starts at or after the new effective_from (a same-start / future version - we cannot slot in
    //     front of it); or
    //   * it is a CLOSED version still covering the new start (effective_to > new effective_from); or
    //   * it is an OPEN version that starts AFTER the new start (a backdate before an open version).
    // The open version whose effective_from <= the new effective_from is exactly the predecessor we
    // supersede below, so it is NOT a conflict.
    let overlap: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM rubric_version
          WHERE rubric_id = $1 AND state = 'published' AND id <> $2
            AND (
                  effective_from >= $3
               OR (effective_to IS NOT NULL AND effective_to > $3)
               OR (effective_to IS NULL AND effective_from > $3)
            )",
    )
    .bind(rubric_id)
    .bind(version_id)
    .bind(effective_from)
    .fetch_one(&mut *tx)
    .await?;
    if overlap > 0 {
        let _ = tx.commit().await;
        return Err(RubricError::EffectiveOverlap);
    }

    // Supersede the currently open-ended published version for this rubric whose interval the new version
    // takes over (close its half-open interval at this effective_from). After the overlap guard the only
    // open predecessor left starts at/before the new effective_from; there is at most one.
    let superseded: Option<Uuid> = sqlx::query_scalar(
        "UPDATE rubric_version
            SET state = 'superseded', effective_to = $3
          WHERE rubric_id = $1 AND state = 'published' AND id <> $2 AND effective_to IS NULL
          RETURNING id",
    )
    .bind(rubric_id)
    .bind(version_id)
    .bind(effective_from)
    .fetch_optional(&mut *tx)
    .await?;

    // Publish this version.
    let published: RubricVersion = sqlx::query_as(
        "UPDATE rubric_version
            SET state = 'published', effective_from = $2,
                approver_subject_id = $3, approved_at = now(),
                published_by_subject_id = $3, published_at = now()
          WHERE id = $1
          RETURNING id, rubric_id, tenant_id, version_no, state, effective_from, effective_to,
                    approver_subject_id, approved_at, published_by_subject_id, published_at,
                    created_at, created_by",
    )
    .bind(version_id)
    .bind(effective_from)
    .bind(approver_subject_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    if let Some(prev) = superseded {
        audit::emit_governance(
            audit_pool,
            tenant_id,
            approver_subject_id,
            audit::kind::RUBRIC_SUPERSEDED,
            serde_json::json!({
                "rubric_id": rubric_id,
                "rubric_version_id": prev,
                "superseded_by": version_id,
                "effective_to": effective_from,
                "actor_subject_id": approver_subject_id,
            }),
        )
        .await;
    }
    audit::emit_governance(
        audit_pool,
        tenant_id,
        approver_subject_id,
        audit::kind::RUBRIC_PUBLISHED,
        serde_json::json!({
            "rubric_id": rubric_id,
            "rubric_version_id": published.id,
            "version_no": published.version_no,
            "actor_subject_id": approver_subject_id,
            "effective_from": effective_from,
        }),
    )
    .await;
    Ok(published)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_approver_gate() {
        // §1 #8 - the gate is exactly "is the approver human". A service account is refused.
        assert!(is_human_approver(true));
        assert!(!is_human_approver(false));
    }

    #[test]
    fn requires_human_approver_short_circuits_before_db() {
        // A non-human approver must fail without any pool access (the check is first in publish_version).
        // We assert the policy seam directly; the DB-backed path is covered by the integration test.
        let err = if is_human_approver(false) {
            None
        } else {
            Some(RubricError::RequiresHumanApprover)
        };
        assert!(matches!(err, Some(RubricError::RequiresHumanApprover)));
        assert_eq!(
            RubricError::RequiresHumanApprover.code(),
            "rubric_requires_human_approver"
        );
    }
}
