//! Access control on evaluation / monitoring data (FR-EVAL-001 clause 7, 8, 9). Within one tenant every
//! employee shares a tenant_id, so RLS alone would let any colleague read any colleague's file. This is
//! the *intra-tenant* boundary RLS does not provide: a read resolves true iff the viewer is the founder,
//! is a designated manager of the target, is the target themselves, or holds an explicit active grant -
//! and is DENIED by default otherwise. Enforcement is defence in depth (tenant RLS AND this check).
//!
//! QUIET OPERATING MODE: access is founder + managers only. Employees see no monitoring / evaluation
//! surface by default; a subject reading their OWN record (clause 7c) is always permitted and is the only
//! self path. Every cross-subject read (viewer != target) emits an audit row (clause 9) so who-looked-at-
//! whose-file is reconstructable after the fact.

use uuid::Uuid;

use crate::db::{self, Pool};

/// The grant kind that authorised a read (for the audit row), or denial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantKind {
    /// The viewer holds an active `founder` grant (clause 7a).
    Founder,
    /// The viewer holds an active `manager_of` grant for this exact target (clause 7b).
    ManagerOf,
    /// The viewer is reading their own record (clause 7c).
    Self_,
}

impl GrantKind {
    /// Stable string for the audit payload / OTel label.
    pub fn as_str(self) -> &'static str {
        match self {
            GrantKind::Founder => "founder",
            GrantKind::ManagerOf => "manager_of",
            GrantKind::Self_ => "self",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("forbidden: {viewer} may not read {target}")]
    Forbidden { viewer: Uuid, target: Uuid },
}

/// Resolve whether `viewer_id` may read `target_subject_id`'s evaluation / monitoring data under
/// `tenant_id`. Returns the grant kind used (for the audit row) or `None` if denied. Deny by default.
///
/// Order of resolution:
///   1. own record (viewer == target) ⇒ always allowed (clause 7c);
///   2. a non-revoked `founder` grant held by the viewer ⇒ allowed (clause 7a);
///   3. a non-revoked `manager_of` grant for the exact (viewer, target) pair ⇒ allowed (clause 7b);
///   4. otherwise denied.
/// All grant lookups run inside a tenant-scoped transaction so RLS confines them to this tenant.
pub async fn may_read(
    pool: &Pool,
    tenant_id: Uuid,
    viewer_id: Uuid,
    target_subject_id: Uuid,
) -> Result<Option<GrantKind>, sqlx::Error> {
    // Own record - clause 7c. Always permitted, no grant row required.
    if viewer_id == target_subject_id {
        return Ok(Some(GrantKind::Self_));
    }

    let mut tx = db::tenant_tx(pool, &tenant_id).await?;

    // Founder: a non-revoked `founder`-scope grant held by this viewer (target column unconstrained -
    // founder may read anyone in the tenant) - clause 7a.
    let founder: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM access_grant
          WHERE viewer_subject_id = $1 AND scope = 'founder' AND revoked_at IS NULL
          LIMIT 1",
    )
    .bind(viewer_id)
    .fetch_optional(&mut *tx)
    .await?;
    if founder.is_some() {
        tx.commit().await?;
        return Ok(Some(GrantKind::Founder));
    }

    // Manager-of: a non-revoked `manager_of`-scope grant for this exact (viewer, target) pair - clause 7b.
    let manager: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM access_grant
          WHERE viewer_subject_id = $1 AND target_subject_id = $2
            AND scope = 'manager_of' AND revoked_at IS NULL
          LIMIT 1",
    )
    .bind(viewer_id)
    .bind(target_subject_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(if manager.is_some() {
        Some(GrantKind::ManagerOf)
    } else {
        None // denied by default
    })
}

/// The boolean predicate downstream code calls: `true` iff `viewer_id` may read
/// `target_subject_id`'s evaluation data under `tenant_id`. Deny by default.
pub async fn can_read_evaluation(
    pool: &Pool,
    tenant_id: Uuid,
    viewer_id: Uuid,
    target_subject_id: Uuid,
) -> Result<bool, sqlx::Error> {
    Ok(may_read(pool, tenant_id, viewer_id, target_subject_id)
        .await?
        .is_some())
}

/// Guard a read of another subject's data: resolve access, then ALWAYS emit a read-audit row
/// (clause 9) - `eval.self_read` for one's own record, `eval.evaluation_read` for a cross-subject read.
/// On denial returns `Forbidden` and emits nothing, so no data is leaked and no false read is recorded.
///
/// `audit_pool` is `AppState::audit_pool` (the memory module's Postgres). `resource` names what was read.
pub async fn guard_evaluation_read(
    pool: &Pool,
    audit_pool: Option<&Pool>,
    tenant_id: Uuid,
    viewer_id: Uuid,
    target_subject_id: Uuid,
    resource: &str,
) -> Result<GrantKind, AccessError> {
    match may_read(pool, tenant_id, viewer_id, target_subject_id).await? {
        Some(GrantKind::Self_) => {
            crate::audit::emit_governance(
                audit_pool,
                tenant_id,
                viewer_id,
                crate::audit::kind::SELF_READ,
                serde_json::json!({ "resource": resource }),
            )
            .await;
            Ok(GrantKind::Self_)
        }
        Some(kind) => {
            crate::audit::emit_governance(
                audit_pool,
                tenant_id,
                viewer_id,
                crate::audit::kind::EVALUATION_READ,
                serde_json::json!({
                    "viewer_subject_id": viewer_id,
                    "target_subject_id": target_subject_id,
                    "grant_kind_used": kind.as_str(),
                    "resource": resource,
                }),
            )
            .await;
            Ok(kind)
        }
        None => Err(AccessError::Forbidden {
            viewer: viewer_id,
            target: target_subject_id,
        }),
    }
}
