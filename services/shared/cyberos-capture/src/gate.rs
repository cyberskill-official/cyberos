//! FR-MEMORY-122 §1 #6 — the REAL consent gate.
//!
//! FR-MEMORY-121 defined the [`ConsentGate`] trait and shipped a default-deny stub (`DenyAll`); this is
//! where FR-MEMORY-122 wires the production gate that consults FR-EVAL-001's acknowledgment ledger. A
//! subject is capturable iff they have acknowledged the tenant's CURRENT published monitoring notice
//! version (DEC-2712). No acknowledgment row -> not capturable; an acknowledgment of an older version
//! (after the notice is re-published) -> not capturable until they re-acknowledge (FR-EVAL-001 clause 17).
//!
//! ## Why this lives here and not in `services/eval`
//!
//! The FR is explicit (DEC-2714 + §1 #16, and the FR-MEMORY-121 consent_gate doc): AUTH and CHAT must NOT
//! depend on the whole eval binary just to ask "may I capture this subject?". So this gate queries the two
//! governance TABLES directly — `monitoring_notice` and `subject_acknowledgment` (owned by
//! `services/eval/migrations/0001_governance.sql`) — over whatever Postgres pool the caller passes. It is
//! byte-for-byte the same predicate `services/eval/src/gate.rs::is_capture_allowed` computes; replicating
//! the two-query shape here (rather than calling eval) keeps the capture path decoupled from eval's
//! liveness while giving the identical verdict.
//!
//! ## RLS
//!
//! `monitoring_notice` and `subject_acknowledgment` are per-tenant RLS tables keyed on the AUTH GUC
//! `app.current_tenant_id` (FR-AUTH-003). So the read runs inside a transaction that first sets that GUC,
//! exactly like chat's `db::tenant_tx` and eval's gate — otherwise the RLS predicate hides every row and
//! the gate would wrongly deny an acknowledged subject. The nil-tenant bypass the policy allows is not
//! used here; capture always carries a real tenant.
//!
//! ## Caching
//!
//! This is the *inner* gate. FR-MEMORY-121's [`CachingGate`] decorator wraps it so a signed-in person's
//! burst of interactions issues at most one ledger read per `(tenant, subject)` per TTL window (default
//! 60 s). [`build_default`] returns the wrapped, ready-to-use gate the modules install.

use async_trait::async_trait;
use cyberos_memory::interaction::{CachingGate, ConsentGate};
use sqlx::PgPool;
use uuid::Uuid;

/// The production consent gate: a SQL read of the FR-EVAL-001 governance ledger. Holds the pool it queries
/// (the brain/governance Postgres — the same deployment that holds `l1_audit_log`, `monitoring_notice`,
/// and `subject_acknowledgment`).
#[derive(Clone)]
pub struct SqlConsentGate {
    pool: PgPool,
}

impl SqlConsentGate {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConsentGate for SqlConsentGate {
    /// `true` iff `subject_id` has acknowledged `tenant_id`'s current published notice version. Mirrors
    /// `services/eval/src/gate.rs` exactly: read the current notice version, read the subject's max
    /// acknowledged version, allow only when the latter is >= the former. A gate-DB error propagates as
    /// `sqlx::Error` (NEVER silently "allowed") — `emit` surfaces it and the caller swallows it best-effort.
    async fn is_capture_allowed(
        &self,
        tenant_id: Uuid,
        subject_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // RLS-scoped transaction: set the tenant GUC so the governance-table policies expose this tenant's
        // rows (identical to chat::db::tenant_tx / eval::db::tenant_tx).
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&mut *tx)
            .await?;

        // The tenant's current published notice version (None => no notice published => deny everyone).
        let current: Option<i32> =
            sqlx::query_scalar("SELECT version FROM monitoring_notice WHERE is_current LIMIT 1")
                .fetch_optional(&mut *tx)
                .await?;
        let Some(current_version) = current else {
            tx.commit().await?;
            return Ok(false);
        };

        // The subject's latest acknowledged version (NULL => never acknowledged any version).
        let latest_ack: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(notice_version) FROM subject_acknowledgment WHERE subject_id = $1",
        )
        .bind(subject_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // Allowed only when the subject has acknowledged the current (or a newer) version.
        Ok(matches!(latest_ack, Some(v) if v >= current_version))
    }
}

/// Build the production gate the modules install: the SQL ledger read [`SqlConsentGate`] wrapped in the
/// FR-MEMORY-121 [`CachingGate`] (default 60 s TTL). A revocation or a fresh acknowledgment takes effect
/// within that TTL — the documented bound. This is what `Capturer::new` is handed.
pub fn build_default(pool: PgPool) -> CachingGate<SqlConsentGate> {
    CachingGate::new(SqlConsentGate::new(pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The gate's SQL is exercised live by services/auth/tests/capture_signin_test.rs and
    // services/chat/tests/smoke_capture.py against a real governance DB (an in-process unit test cannot
    // stand up the RLS tables). Here we only assert the type wiring: SqlConsentGate is a ConsentGate, and
    // build_default yields a CachingGate over it that is itself a ConsentGate (so it coerces to
    // &dyn ConsentGate for emit()).
    fn _assert_is_consent_gate<G: ConsentGate>() {}

    #[test]
    fn sql_gate_and_caching_wrapper_are_consent_gates() {
        _assert_is_consent_gate::<SqlConsentGate>();
        _assert_is_consent_gate::<CachingGate<SqlConsentGate>>();
    }
}
