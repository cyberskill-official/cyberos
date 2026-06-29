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

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        return Err(forbidden("only the founder may publish a monitoring notice"));
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
            return Err(forbidden("only the founder or a manager may read the notice"));
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
        return Err((StatusCode::BAD_REQUEST, "declared purpose is required".to_string()));
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
        return Err((StatusCode::BAD_REQUEST, "retain_days must be > 0".to_string()));
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
            .map(|(id, viewer_subject_id, scope, granted_at, revoked_at)| MyGrant {
                id,
                viewer_subject_id,
                scope,
                granted_at,
                revoked_at,
            })
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
