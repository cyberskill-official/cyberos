//! FR-EVAL-001 slice 2 - the governance HTTP surface. Each handler verifies the caller (RS256 token ->
//! [`crate::auth::Caller`]), opens a tenant-scoped transaction (the FR-AUTH-003 RLS GUC), does the access
//! / role check, performs the mutation or read, and emits the clause-12 audit row. JSON in, JSON out, the
//! `(StatusCode, String)` error shape - all mirroring `cyberos_chat`'s handlers.
//!
//! The checks REUSE slice 1's logic, never re-implement it:
//!   * the consent gate via [`crate::gate::is_capture_allowed`] / [`crate::gate::gate_reason`];
//!   * the access resolution via [`crate::access::can_read_evaluation`] / [`crate::access::may_read`] /
//!     [`crate::access::guard_evaluation_read`].
//!
//! QUIET OPERATING MODE (founder decision 2026-06-30): the employee self-view (`GET /v1/eval/me`) returns
//! the caller's OWN record only - never another subject's - enforced server-side, deny-by-default; the
//! acknowledgment recorded here defaults to `ack_source = 'signed_contract'` (the HR action when an
//! employee signs the clause); there is no path that captures or evaluates a subject with no
//! acknowledgment row. The in-app notice surface stays off by default.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rubric::{self, model::RubricError};
use crate::{audit, auth, db, gate, AppState};

/// 403 helper, mirroring the `(StatusCode, String)` error contract used across the handlers.
fn forbidden(msg: &str) -> (StatusCode, String) {
    (StatusCode::FORBIDDEN, msg.to_string())
}

// ===========================================================================
// Notice (clause 1) - POST publishes a new version (founder only); GET returns the current one.
// ===========================================================================

#[derive(Debug, Deserialize)]
pub struct PublishNotice {
    /// The human-readable notice text, English.
    pub lang_en: String,
    /// The human-readable notice text, Vietnamese.
    pub lang_vi: String,
    /// A short lawful-basis summary (e.g. "Decree 13/2023/ND-CP + Labor Code 45/2019/QH14").
    pub lawful_basis: String,
}

#[derive(Debug, Serialize)]
pub struct Notice {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub version: i32,
    pub lang_en: String,
    pub lang_vi: String,
    pub lawful_basis: String,
    pub is_current: bool,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub published_by: Uuid,
}

type NoticeRow = (
    Uuid,
    Uuid,
    i32,
    String,
    String,
    String,
    bool,
    chrono::DateTime<chrono::Utc>,
    Uuid,
);

fn to_notice(r: NoticeRow) -> Notice {
    Notice {
        id: r.0,
        tenant_id: r.1,
        version: r.2,
        lang_en: r.3,
        lang_vi: r.4,
        lawful_basis: r.5,
        is_current: r.6,
        published_at: r.7,
        published_by: r.8,
    }
}

/// `POST /v1/eval/notice` - publish a new monitoring-notice version (founder only). The new version is
/// `max(version)+1` for the tenant; the prior current row is flipped to `is_current=false` in the same
/// transaction (clause 1). Emits `eval.notice_published`.
pub async fn publish_notice(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PublishNotice>,
) -> Result<(StatusCode, Json<Notice>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.is_founder {
        return Err(forbidden(
            "only the founder may publish a monitoring notice",
        ));
    }
    let en = body.lang_en.trim();
    let vi = body.lang_vi.trim();
    let basis = body.lawful_basis.trim();
    if en.is_empty() || vi.is_empty() || basis.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "lang_en, lang_vi, and lawful_basis are required".to_string(),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    // Next version for this tenant (RLS scopes the MAX to this tenant).
    let next_version: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM monitoring_notice")
            .fetch_one(&mut *tx)
            .await
            .map_err(crate::internal)?;
    // Flip the prior current notice to non-current, in the same transaction (clause 1).
    sqlx::query("UPDATE monitoring_notice SET is_current = FALSE WHERE is_current")
        .execute(&mut *tx)
        .await
        .map_err(crate::internal)?;
    let row: NoticeRow = sqlx::query_as(
        "INSERT INTO monitoring_notice
            (tenant_id, version, lang_en, lang_vi, lawful_basis, is_current, published_by)
         VALUES ($1, $2, $3, $4, $5, TRUE, $6)
         RETURNING id, tenant_id, version, lang_en, lang_vi, lawful_basis, is_current,
                   published_at, published_by",
    )
    .bind(caller.tenant_id)
    .bind(next_version)
    .bind(en)
    .bind(vi)
    .bind(basis)
    .bind(caller.subject_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let notice = to_notice(row);
    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::NOTICE_PUBLISHED,
        serde_json::json!({
            "notice_id": notice.id,
            "version": notice.version,
            "lawful_basis": notice.lawful_basis,
            "published_by": caller.subject_id,
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(notice)))
}

/// `GET /v1/eval/notice` - the tenant's current published notice (founder or manager). 404 if none yet.
pub async fn get_notice(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Notice>, (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    // Founder or manager: a manager is any caller holding an active manager_of grant (they can see the
    // notice they are accountable for). Resolve "is this caller a manager of anyone" cheaply below.
    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    if !caller.is_founder {
        let is_manager: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM access_grant
              WHERE viewer_subject_id = $1 AND scope = 'manager_of' AND revoked_at IS NULL
              LIMIT 1",
        )
        .bind(caller.subject_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
        if is_manager.is_none() {
            let _ = tx.commit().await;
            return Err(forbidden(
                "only the founder or a manager may read the notice",
            ));
        }
    }
    let row: Option<NoticeRow> = sqlx::query_as(
        "SELECT id, tenant_id, version, lang_en, lang_vi, lawful_basis, is_current,
                published_at, published_by
           FROM monitoring_notice WHERE is_current LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;
    match row {
        Some(r) => Ok(Json(to_notice(r))),
        None => Err((StatusCode::NOT_FOUND, "no notice published yet".to_string())),
    }
}

// ===========================================================================
// Category registry (clause 4, 5) - POST registers/updates a data category (founder only).
// ===========================================================================

/// Out-of-scope category-name fragments (clause 5 minimisation, DEC-2522). A category whose name matches
/// any of these is rejected at registration - scope is platform work-interactions ONLY. The match is
/// substring + case-insensitive so `keystroke_log`, `screen_capture`, `webcam`, etc. are all caught.
const OUT_OF_SCOPE_FRAGMENTS: &[&str] = &[
    "keystroke",
    "keylog",
    "screen",
    "screenshot",
    "camera",
    "webcam",
    "microphone",
    "mic_",
    "audio",
    "location",
    "gps",
    "geolocat",
    "private",
];

fn is_out_of_scope(name: &str) -> bool {
    let n = name.to_lowercase();
    OUT_OF_SCOPE_FRAGMENTS.iter().any(|f| n.contains(f))
}

#[derive(Debug, Deserialize)]
pub struct RegisterCategory {
    pub name: String,
    pub purpose: String,
    /// Closed enum: legitimate_interest | contract_performance | legal_obligation | consent.
    pub lawful_basis: String,
}

#[derive(Debug, Serialize)]
pub struct Category {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub purpose: String,
    pub lawful_basis: String,
    pub created_by: Uuid,
}

const LAWFUL_BASES: &[&str] = &[
    "legitimate_interest",
    "contract_performance",
    "legal_obligation",
    "consent",
];

/// `POST /v1/eval/categories` - register or update a data category (founder only). Rejects an empty
/// purpose, a lawful_basis outside the closed enum, and an out-of-scope category name (422). Emits
/// `eval.category_registered`.
pub async fn register_category(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RegisterCategory>,
) -> Result<(StatusCode, Json<Category>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.is_founder {
        return Err(forbidden("only the founder may register a data category"));
    }
    let name = body.name.trim();
    let purpose = body.purpose.trim();
    let basis = body.lawful_basis.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".to_string()));
    }
    if purpose.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "declared purpose is required".to_string(),
        ));
    }
    if !LAWFUL_BASES.contains(&basis) {
        return Err((
            StatusCode::BAD_REQUEST,
            "lawful_basis must be one of legitimate_interest, contract_performance, legal_obligation, consent".to_string(),
        ));
    }
    // Clause 5 minimisation: scope is platform work-interactions ONLY.
    if is_out_of_scope(name) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "category_out_of_scope".to_string(),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    let row: (Uuid, Uuid, String, String, String, Uuid) = sqlx::query_as(
        "INSERT INTO data_category (tenant_id, name, purpose, lawful_basis, created_by)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (tenant_id, name)
         DO UPDATE SET purpose = EXCLUDED.purpose, lawful_basis = EXCLUDED.lawful_basis
         RETURNING id, tenant_id, name, purpose, lawful_basis, created_by",
    )
    .bind(caller.tenant_id)
    .bind(name)
    .bind(purpose)
    .bind(basis)
    .bind(caller.subject_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let category = Category {
        id: row.0,
        tenant_id: row.1,
        name: row.2,
        purpose: row.3,
        lawful_basis: row.4,
        created_by: row.5,
    };
    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::CATEGORY_REGISTERED,
        serde_json::json!({
            "category": category.name,
            "lawful_basis": category.lawful_basis,
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(category)))
}

// ===========================================================================
// Acknowledgment (clause 2, 3) - the HR action that flips the consent gate for a subject.
// ===========================================================================

#[derive(Debug, Deserialize)]
pub struct RecordAck {
    /// The subject whose acknowledgment is being recorded (the employee who signed the clause).
    pub subject_id: Uuid,
    /// Source of the acknowledgment. Defaults to 'signed_contract' (the quiet-operating-mode HR action).
    #[serde(default = "default_ack_source")]
    pub ack_source: String,
}

fn default_ack_source() -> String {
    "signed_contract".to_string()
}

#[derive(Debug, Serialize)]
pub struct AckRecorded {
    pub subject_id: Uuid,
    pub notice_version: i32,
    pub ack_source: String,
    pub recorded_by: Uuid,
    /// Whether capture is now allowed for this subject (the gate consequence of recording the ack).
    pub capture_allowed: bool,
}

/// `POST /v1/eval/ack` - record a subject's acknowledgment of the CURRENT notice version (founder /
/// admin / HR only). `ack_source` defaults to 'signed_contract' (the HR-recorded clause acceptance);
/// `recorded_by` is the caller. This is the action that flips the consent gate for that subject. Emits
/// `eval.ack_recorded`.
pub async fn record_ack(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RecordAck>,
) -> Result<(StatusCode, Json<AckRecorded>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.may_record_ack() {
        return Err(forbidden(
            "only the founder, an admin, or HR may record an acknowledgment",
        ));
    }
    let ack_source = body.ack_source.trim();
    if !matches!(ack_source, "signed_contract" | "in_app") {
        return Err((
            StatusCode::BAD_REQUEST,
            "ack_source must be signed_contract or in_app".to_string(),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    // Resolve the current notice (id + version) - the ack is always against the current version.
    let current: Option<(Uuid, i32)> =
        sqlx::query_as("SELECT id, version FROM monitoring_notice WHERE is_current LIMIT 1")
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;
    let (notice_id, version) = match current {
        Some(c) => c,
        None => {
            let _ = tx.commit().await;
            return Err((
                StatusCode::CONFLICT,
                "no current notice to acknowledge; publish one first".to_string(),
            ));
        }
    };
    // Append-only ledger: a re-ack of the same version is idempotent (UNIQUE tenant+subject+version).
    sqlx::query(
        "INSERT INTO subject_acknowledgment
            (tenant_id, subject_id, notice_id, notice_version, ack_source, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (tenant_id, subject_id, notice_version) DO NOTHING",
    )
    .bind(caller.tenant_id)
    .bind(body.subject_id)
    .bind(notice_id)
    .bind(version)
    .bind(ack_source)
    .bind(caller.subject_id)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    // The gate consequence: capture is now allowed for this subject (reuse slice 1's gate).
    let capture_allowed = gate::is_capture_allowed(&st.pool, caller.tenant_id, body.subject_id)
        .await
        .map_err(crate::internal)?;

    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::ACK_RECORDED,
        serde_json::json!({
            "subject_id": body.subject_id,
            "notice_version": version,
            "ack_source": ack_source,
            "recorded_by": caller.subject_id,
        }),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(AckRecorded {
            subject_id: body.subject_id,
            notice_version: version,
            ack_source: ack_source.to_string(),
            recorded_by: caller.subject_id,
            capture_allowed,
        }),
    ))
}

// ===========================================================================
// Access grants (clause 7, 8) - grant (founder only) and revoke (founder only).
// ===========================================================================

#[derive(Debug, Deserialize)]
pub struct GrantAccess {
    pub viewer_subject_id: Uuid,
    /// For a manager_of / self grant, the subject being read. For a founder grant the target column is a
    /// formality (founder reads anyone); pass the same viewer id, or any tenant subject.
    pub target_subject_id: Uuid,
    /// founder | manager_of | self.
    pub scope: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Grant {
    pub id: Uuid,
    pub viewer_subject_id: Uuid,
    pub target_subject_id: Uuid,
    pub scope: String,
    pub granted_by: Uuid,
}

/// `POST /v1/eval/access` - grant access (founder only). Scope in founder|manager_of|self. Emits
/// `eval.access_granted`.
pub async fn grant_access(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GrantAccess>,
) -> Result<(StatusCode, Json<Grant>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.is_founder {
        return Err(forbidden("only the founder may grant access"));
    }
    if !matches!(body.scope.as_str(), "founder" | "manager_of" | "self") {
        return Err((
            StatusCode::BAD_REQUEST,
            "scope must be founder, manager_of, or self".to_string(),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    let row: (Uuid, Uuid, Uuid, String) = sqlx::query_as(
        "INSERT INTO access_grant
            (tenant_id, viewer_subject_id, target_subject_id, scope, granted_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, viewer_subject_id, target_subject_id, scope",
    )
    .bind(caller.tenant_id)
    .bind(body.viewer_subject_id)
    .bind(body.target_subject_id)
    .bind(&body.scope)
    .bind(caller.subject_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let grant = Grant {
        id: row.0,
        viewer_subject_id: row.1,
        target_subject_id: row.2,
        scope: row.3,
        granted_by: caller.subject_id,
    };
    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::ACCESS_GRANTED,
        serde_json::json!({
            "grant_id": grant.id,
            "viewer_subject_id": grant.viewer_subject_id,
            "target_subject_id": grant.target_subject_id,
            "scope": grant.scope,
            "reason": body.reason,
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(grant)))
}

#[derive(Debug, Deserialize)]
pub struct RevokeAccess {
    pub grant_id: Uuid,
}

/// `POST /v1/eval/access/revoke` - revoke a grant by setting `revoked_at` (founder only). In production
/// this UPDATE runs as the admin role (`cyberos_ops`); the runtime app role is REVOKE'd UPDATE on
/// `access_grant` (clause 15). Emits `eval.access_revoked`.
pub async fn revoke_access(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RevokeAccess>,
) -> Result<StatusCode, (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.is_founder {
        return Err(forbidden("only the founder may revoke access"));
    }

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    let res = sqlx::query(
        "UPDATE access_grant SET revoked_at = now()
          WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(body.grant_id)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;
    if res.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "grant not found or already revoked".to_string(),
        ));
    }

    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::ACCESS_REVOKED,
        serde_json::json!({ "grant_id": body.grant_id }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

// ===========================================================================
// Retention (clause 6) - set/replace a per-category retention policy (founder only).
// ===========================================================================

#[derive(Debug, Deserialize)]
pub struct SetRetention {
    /// The data-category name the policy applies to (must already be registered).
    pub category: String,
    pub retain_days: i32,
    #[serde(default)]
    pub basis: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Retention {
    pub category: String,
    pub data_category_id: Uuid,
    pub retain_days: i32,
    pub basis: String,
}

/// `POST /v1/eval/retention` - set or replace the retention policy for a registered data category
/// (founder only). Emits `eval.retention_changed`.
pub async fn set_retention(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SetRetention>,
) -> Result<(StatusCode, Json<Retention>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.is_founder {
        return Err(forbidden("only the founder may set a retention policy"));
    }
    let category = body.category.trim();
    if category.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "category is required".to_string()));
    }
    if body.retain_days <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "retain_days must be > 0".to_string(),
        ));
    }
    let basis = body.basis.as_deref().unwrap_or("legitimate_interest");

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    // The policy is keyed by the registered category - resolve its id first.
    let cat: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM data_category WHERE name = $1 LIMIT 1")
            .bind(category)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;
    let data_category_id = match cat {
        Some((id,)) => id,
        None => {
            let _ = tx.commit().await;
            return Err((
                StatusCode::NOT_FOUND,
                "category not registered; register it first".to_string(),
            ));
        }
    };
    sqlx::query(
        "INSERT INTO retention_policy
            (tenant_id, data_category_id, retain_days, basis, updated_by)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (tenant_id, data_category_id)
         DO UPDATE SET retain_days = EXCLUDED.retain_days, basis = EXCLUDED.basis,
                       updated_at = now(), updated_by = EXCLUDED.updated_by",
    )
    .bind(caller.tenant_id)
    .bind(data_category_id)
    .bind(body.retain_days)
    .bind(basis)
    .bind(caller.subject_id)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::RETENTION_CHANGED,
        serde_json::json!({
            "category": category,
            "retain_days": body.retain_days,
            "basis": basis,
        }),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(Retention {
            category: category.to_string(),
            data_category_id,
            retain_days: body.retain_days,
            basis: basis.to_string(),
        }),
    ))
}

// ===========================================================================
// Governance status (clause 18) - the founder/manager at-a-glance proof the gate is healthy. Read-only,
// tenant-scoped, founder-or-manager gated (mirrors get_notice).
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct CategoryStatus {
    pub name: String,
    pub purpose: String,
    pub lawful_basis: String,
    /// The retention policy in force for this category, if one is set (clause 6). `None` ⇒ no policy ⇒ the
    /// sweeper retains the category's data indefinitely (the operator must set a policy to bound it).
    pub retain_days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct GovernanceStatus {
    /// The tenant's current published monitoring-notice version, or `None` if none has been published yet
    /// (in which case every subject is gated, clause 3 #6).
    pub current_notice_version: Option<i32>,
    /// Count of distinct subjects whose latest acknowledgment IS the current notice version (the gate is
    /// lifted for them).
    pub acknowledged_current: i64,
    /// Count of distinct subjects who acknowledged an OLDER version and are now re-gated by a notice bump
    /// (clause 17 `StaleAckVersion`). Subjects who never acknowledged any version are NOT counted here: the
    /// eval DB holds no subject roster (that lives in AUTH), and the gate treats a no-ack subject as gated at
    /// capture time regardless.
    pub stale_ack_subjects: i64,
    /// The registered data categories with their declared purpose, lawful basis, and retention (clause 4, 6).
    pub categories: Vec<CategoryStatus>,
    /// Count of active (non-revoked) access grants in the tenant (clause 8).
    pub active_grants: i64,
    /// Count of open (unresolved) data-subject requests queued for a human (clause 10).
    pub open_requests: i64,
}

/// `GET /v1/eval/governance/status` - the operator's at-a-glance governance posture (clause 18): the current
/// notice version, acknowledged vs stale-ack subject counts, the registered categories with their purpose +
/// lawful_basis + retention, the active-grant count, and the open-DSR count. Founder or an active manager
/// (mirrors `get_notice`). Read-only, tenant-scoped via `tenant_tx`; emits no audit row (a read of aggregate
/// posture, not of any subject's record).
pub async fn governance_status(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<GovernanceStatus>, (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    // Founder or an active manager_of grant may read the posture (same gate as get_notice).
    if !caller.is_founder {
        let is_manager: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM access_grant
              WHERE viewer_subject_id = $1 AND scope = 'manager_of' AND revoked_at IS NULL
              LIMIT 1",
        )
        .bind(caller.subject_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
        if is_manager.is_none() {
            let _ = tx.commit().await;
            return Err(forbidden(
                "only the founder or a manager may read governance status",
            ));
        }
    }

    // The current notice version (None ⇒ nothing published yet ⇒ everyone gated).
    let current_notice_version: Option<i32> =
        sqlx::query_scalar("SELECT version FROM monitoring_notice WHERE is_current LIMIT 1")
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;

    // Acknowledged-vs-stale subject counts, computed against the current version. A subject's latest acked
    // version >= current ⇒ acknowledged; < current ⇒ stale (re-gated, clause 17). Both derive from the
    // per-subject MAX(notice_version), so a subject is counted once.
    let (acknowledged_current, stale_ack_subjects): (i64, i64) = match current_notice_version {
        Some(v) => sqlx::query_as(
            "SELECT
                COUNT(*) FILTER (WHERE latest >= $1) AS acknowledged,
                COUNT(*) FILTER (WHERE latest <  $1) AS stale
             FROM (
                SELECT subject_id, MAX(notice_version) AS latest
                  FROM subject_acknowledgment
                 GROUP BY subject_id
             ) per_subject",
        )
        .bind(v)
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::internal)?,
        None => (0, 0), // no current notice ⇒ no one is "acknowledged-current".
    };

    // Registered categories LEFT JOINed to their retention policy (the policy may be unset ⇒ retain_days
    // None). Ordered by name for a stable view.
    let cat_rows: Vec<(String, String, String, Option<i32>)> = sqlx::query_as(
        "SELECT dc.name, dc.purpose, dc.lawful_basis, rp.retain_days
           FROM data_category dc
           LEFT JOIN retention_policy rp ON rp.data_category_id = dc.id
          ORDER BY dc.name",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;

    let active_grants: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM access_grant WHERE revoked_at IS NULL")
            .fetch_one(&mut *tx)
            .await
            .map_err(crate::internal)?;

    // Open DSR count. subject_request is the slice-2 table (migration 0002); it may be absent in a partial
    // deployment, so tolerate a missing-relation error as zero rather than 500-ing the whole status.
    let open_requests: i64 = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM subject_request WHERE status = 'open'",
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(n) => n,
        Err(sqlx::Error::Database(_)) => 0,
        Err(e) => {
            let _ = tx.commit().await;
            return Err(crate::internal(e));
        }
    };
    let _ = tx.commit().await;

    let categories = cat_rows
        .into_iter()
        .map(
            |(name, purpose, lawful_basis, retain_days)| CategoryStatus {
                name,
                purpose,
                lawful_basis,
                retain_days,
            },
        )
        .collect();

    Ok(Json(GovernanceStatus {
        current_notice_version,
        acknowledged_current,
        stale_ack_subjects,
        categories,
        active_grants,
        open_requests,
    }))
}

// ===========================================================================
// Data-subject self surface (clause 10) - GET /me (own record), POST /me/requests (file a request).
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct MyAck {
    pub notice_version: i32,
    pub ack_source: String,
    pub acknowledged_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct MyGrant {
    pub id: Uuid,
    pub viewer_subject_id: Uuid,
    pub scope: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MyRecord {
    pub subject_id: Uuid,
    pub capture_allowed: bool,
    /// The caller's own acknowledgments.
    pub acknowledgments: Vec<MyAck>,
    /// The access grants ABOUT the caller (who may read the caller's record).
    pub access_grants_about_me: Vec<MyGrant>,
}

/// `GET /v1/eval/me` - the caller's OWN record: their acknowledgments and the access grants about them.
/// Self-access, always allowed for the caller's own subject_id, and it returns ONLY the caller's own
/// data (the quiet-operating-mode deny-by-default self-view). The self read is audited via slice 1's
/// `guard_evaluation_read` (`eval.self_read`).
pub async fn get_me(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MyRecord>, (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;

    // Count + audit the self read via slice 1's guard (viewer == target => Self_ => eval.self_read).
    crate::access::guard_evaluation_read(
        &st.pool,
        st.audit_pool.as_ref(),
        caller.tenant_id,
        caller.subject_id,
        caller.subject_id,
        "eval/me",
    )
    .await
    .map_err(|e| match e {
        crate::access::AccessError::Forbidden { .. } => forbidden("forbidden"),
        crate::access::AccessError::Db(e) => crate::internal(e),
    })?;

    let capture_allowed = gate::is_capture_allowed(&st.pool, caller.tenant_id, caller.subject_id)
        .await
        .map_err(crate::internal)?;

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    // The caller's own acknowledgments (RLS + explicit subject_id = the caller; never another subject).
    let acks: Vec<(i32, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT notice_version, ack_source, acknowledged_at
           FROM subject_acknowledgment WHERE subject_id = $1 ORDER BY notice_version",
    )
    .bind(caller.subject_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    // The grants ABOUT the caller (target = the caller): who may read the caller's record.
    type GrantRow = (
        Uuid,
        Uuid,
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    );
    let grants: Vec<GrantRow> = sqlx::query_as(
        "SELECT id, viewer_subject_id, scope, granted_at, revoked_at
           FROM access_grant WHERE target_subject_id = $1 ORDER BY granted_at",
    )
    .bind(caller.subject_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;

    Ok(Json(MyRecord {
        subject_id: caller.subject_id,
        capture_allowed,
        acknowledgments: acks
            .into_iter()
            .map(|(notice_version, ack_source, acknowledged_at)| MyAck {
                notice_version,
                ack_source,
                acknowledged_at,
            })
            .collect(),
        access_grants_about_me: grants
            .into_iter()
            .map(
                |(id, viewer_subject_id, scope, granted_at, revoked_at)| MyGrant {
                    id,
                    viewer_subject_id,
                    scope,
                    granted_at,
                    revoked_at,
                },
            )
            .collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct FileRequest {
    /// access | rectification | objection.
    pub kind: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubjectRequest {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub kind: String,
    pub status: String,
}

/// `POST /v1/eval/me/requests` - the caller files a data-subject request about their OWN record. Stored
/// `open` and QUEUED for a human; NEVER auto-applied (clause 11). `subject_id` is always the caller's own
/// (a subject cannot file about another). Emits `eval.subject_request`.
pub async fn file_request(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FileRequest>,
) -> Result<(StatusCode, Json<SubjectRequest>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    let kind = body.kind.trim();
    if !matches!(kind, "access" | "rectification" | "objection") {
        return Err((
            StatusCode::BAD_REQUEST,
            "kind must be access, rectification, or objection".to_string(),
        ));
    }
    let note = body.note.as_deref().unwrap_or("").trim().to_string();

    let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
        .await
        .map_err(crate::internal)?;
    // subject_id is ALWAYS the caller's own - a subject can only file about themselves.
    let row: (Uuid, Uuid, String, String) = sqlx::query_as(
        "INSERT INTO subject_request (tenant_id, subject_id, kind, note)
         VALUES ($1, $2, $3, $4)
         RETURNING id, subject_id, kind, status",
    )
    .bind(caller.tenant_id)
    .bind(caller.subject_id)
    .bind(kind)
    .bind(&note)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let req = SubjectRequest {
        id: row.0,
        subject_id: row.1,
        kind: row.2,
        status: row.3,
    };
    audit::emit(
        &st,
        caller.tenant_id,
        caller.subject_id,
        audit::kind::SUBJECT_REQUEST,
        serde_json::json!({
            "request_id": req.id,
            "kind": req.kind,
            "status": req.status,
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(req)))
}

// ===========================================================================
// Rubric (FR-EVAL-002) - the human-curated, clause-cited evaluation rubric. Authoring (create / open
// version / add item / publish) requires the rubric-admin grant (founder + designated rubric admins,
// §1 #10); reading the effective version requires founder or manager, mirroring GET /v1/eval/notice. Every
// mutation emits a hash-chained `eval.rubric_*` audit row inside the rubric module functions (§1 #11). No
// path here scores a person or calls a model - the GENIE draft path is a deferred slice.
// ===========================================================================

/// Map a `RubricError` to the `(StatusCode, String)` HTTP shape with the §1 / §4 stable error code as the
/// body. The status codes follow the FR's failure-modes table: uncited / missing-vi / bad-shape / empty ->
/// 422; effective overlap -> 409; human-approver-required -> 403; not-found -> 404; everything else a 422
/// authoring error except a DB fault which is a 500.
fn rubric_err(e: RubricError) -> (StatusCode, String) {
    let status = match e {
        RubricError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        RubricError::RequiresHumanApprover => StatusCode::FORBIDDEN,
        RubricError::EffectiveOverlap => StatusCode::CONFLICT,
        RubricError::NotFound | RubricError::NoEffectiveVersion => StatusCode::NOT_FOUND,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    (status, e.code().to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateRubric {
    pub name: String,
}

/// `POST /v1/eval/rubrics` - create a named rubric framework (rubric-admin only). A duplicate name in the
/// tenant is a 409. Emits `eval.rubric_drafted`.
pub async fn create_rubric(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateRubric>,
) -> Result<(StatusCode, Json<rubric::Rubric>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.may_administer_rubric() {
        return Err(forbidden("only a rubric admin may create a rubric"));
    }
    let name = body.name.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".to_string()));
    }
    let res = rubric::authoring::create_rubric(
        &st.pool,
        st.audit_pool.as_ref(),
        caller.tenant_id,
        caller.subject_id,
        name,
    )
    .await;
    match res {
        Ok(r) => Ok((StatusCode::CREATED, Json(r))),
        // A unique-violation on (tenant_id, name) is a 409, distinct from the authoring 422s.
        Err(RubricError::Db(sqlx::Error::Database(db_err))) if db_err.is_unique_violation() => {
            Err((
                StatusCode::CONFLICT,
                "a rubric with that name already exists".to_string(),
            ))
        }
        Err(e) => Err(rubric_err(e)),
    }
}

/// `POST /v1/eval/rubrics/{id}/versions` - open a new draft version of a rubric (rubric-admin only). Emits
/// `eval.rubric_drafted`.
pub async fn open_rubric_version(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(rubric_id): Path<Uuid>,
) -> Result<(StatusCode, Json<rubric::RubricVersion>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.may_administer_rubric() {
        return Err(forbidden("only a rubric admin may open a rubric version"));
    }
    let version = rubric::versioning::open_version(
        &st.pool,
        st.audit_pool.as_ref(),
        caller.tenant_id,
        caller.subject_id,
        rubric_id,
    )
    .await
    .map_err(rubric_err)?;
    Ok((StatusCode::CREATED, Json(version)))
}

/// `POST /v1/eval/rubrics/{id}/versions/{vid}/items` - add a clause-cited item to a draft version
/// (rubric-admin only). The body is a [`rubric::RubricItemDraft`]; an uncited / missing-vi / bad-shape item
/// is rejected 422 with the matching code. Emits `eval.rubric_drafted`.
pub async fn add_rubric_item(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path((_rubric_id, version_id)): Path<(Uuid, Uuid)>,
    Json(draft): Json<rubric::RubricItemDraft>,
) -> Result<(StatusCode, Json<rubric::RubricItem>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.may_administer_rubric() {
        return Err(forbidden("only a rubric admin may add a rubric item"));
    }
    let item = rubric::authoring::add_item(
        &st.pool,
        st.audit_pool.as_ref(),
        caller.tenant_id,
        caller.subject_id,
        version_id,
        &draft,
    )
    .await
    .map_err(rubric_err)?;
    Ok((StatusCode::CREATED, Json(item)))
}

#[derive(Debug, Deserialize)]
pub struct PublishRubricVersion {
    /// The date the version becomes effective (half-open interval start). Defaults to today if omitted.
    #[serde(default)]
    pub effective_from: Option<chrono::NaiveDate>,
}

/// `POST /v1/eval/rubrics/{id}/versions/{vid}/publish` - the HITL publish transition (rubric-admin only,
/// human approver). The caller is the human approver; a service-account token is rejected 403
/// (`rubric_requires_human_approver`). Publish is blocked 422 if the version is empty or has an ungrounded
/// item, and 409 if its effective_from overlaps a live published version. Emits `eval.rubric_superseded`
/// (if one was superseded) and `eval.rubric_published`.
pub async fn publish_rubric_version(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path((_rubric_id, version_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<PublishRubricVersion>,
) -> Result<(StatusCode, Json<rubric::RubricVersion>), (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    if !caller.may_administer_rubric() {
        return Err(forbidden(
            "only a rubric admin may publish a rubric version",
        ));
    }
    // The caller's verified token IS the human approver (§1 #8). A real service account would carry a
    // service-account role; the founder / rubric-admin path here is a human by construction. The
    // `approver_is_human` seam stays explicit so a future service-account caller is refused.
    let approver_is_human = !caller.has_any_role(&["service-account", "service_account"]);
    let effective_from = body
        .effective_from
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let version = rubric::versioning::publish_version(
        &st.pool,
        st.audit_pool.as_ref(),
        caller.tenant_id,
        caller.subject_id,
        approver_is_human,
        version_id,
        effective_from,
    )
    .await
    .map_err(rubric_err)?;
    Ok((StatusCode::CREATED, Json(version)))
}

#[derive(Debug, Deserialize)]
pub struct EffectiveQuery {
    /// The date to resolve the effective version for. Defaults to today.
    #[serde(default)]
    pub at: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct EffectiveRubric {
    pub version: rubric::RubricVersion,
    pub items: Vec<rubric::RubricItem>,
}

/// `GET /v1/eval/rubrics/{id}?at=<date>` - the published version effective on a date (defaults to today),
/// with all its items and their citations (§1 #7 #12). Read access requires founder or a manager, mirroring
/// `get_notice`. 404 if no version is in force on the date.
pub async fn get_effective_rubric(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(rubric_id): Path<Uuid>,
    Query(q): Query<EffectiveQuery>,
) -> Result<Json<EffectiveRubric>, (StatusCode, String)> {
    let caller = auth::caller(&st, &headers)?;
    // Founder or a manager may read the rubric (the standard), mirroring the notice read gate.
    if !caller.is_founder {
        let mut tx = db::tenant_tx(&st.pool, &caller.tenant_id)
            .await
            .map_err(crate::internal)?;
        let is_manager: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM access_grant
              WHERE viewer_subject_id = $1 AND scope = 'manager_of' AND revoked_at IS NULL
              LIMIT 1",
        )
        .bind(caller.subject_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
        let _ = tx.commit().await;
        if is_manager.is_none() {
            return Err(forbidden(
                "only the founder or a manager may read the rubric",
            ));
        }
    }
    let at = q.at.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let version = rubric::versioning::resolve_effective(&st.pool, caller.tenant_id, rubric_id, at)
        .await
        .map_err(rubric_err)?;
    let items = rubric::authoring::list_items(&st.pool, caller.tenant_id, version.id)
        .await
        .map_err(rubric_err)?;
    Ok(Json(EffectiveRubric { version, items }))
}
