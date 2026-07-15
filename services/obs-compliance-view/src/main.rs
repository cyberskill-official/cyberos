//! `obs-compliance-view` HTTP service (TASK-OBS-008 I/O shell). Serves the four read-only compliance views
//! over the memory audit chain. Every request: authenticates the auditor JWT (external_auditor role),
//! enforces tenant scope, validates the time window, queries the kind-filtered audit rows, summarises
//! them, runs the defence-in-depth PII scan, signs the canonical response with the Ed25519 chain-proof,
//! and emits an `obs.compliance_view_accessed` audit line. Read-only - it never writes the chain (DEC-177).
//!
//! Routes: `GET /:view` (eu-ai-act | pdpl | soc2 | iso27001) with `?since=&until=` RFC3339 params, and
//! `GET /healthz`. Config (env): `DATABASE_URL`, one of `OBS_COMPLIANCE_JWKS_JSON` /
//! `OBS_COMPLIANCE_HS256_SECRET`, `OBS_COMPLIANCE_SIGNING_KEY_HEX` (64 hex), `OBS_COMPLIANCE_ADDR`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use cyberos_obs_compliance_view::{
    auth::{enforce_tenant_scope, AuthError, Authenticator},
    pii_scan, proof, query, summary,
    views::View,
    window,
};

const DEFAULT_ADDR: &str = "0.0.0.0:7788";
const NS_PER_SEC: i64 = 1_000_000_000;
const DEFAULT_WINDOW_DAYS: i64 = 90;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    auth: Arc<Authenticator>,
    signing_key: [u8; 32],
}

#[tokio::main]
async fn main() {
    let addr = std::env::var("OBS_COMPLIANCE_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());

    let pool = match build_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("fatal: cannot connect to audit store: {e}");
            std::process::exit(1);
        }
    };

    let auth = match build_authenticator() {
        Ok(a) => Arc::new(a),
        Err(e) => {
            eprintln!("fatal: cannot build auditor verifier: {e}");
            std::process::exit(1);
        }
    };

    let signing_key = match signing_key_from_env() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("fatal: cannot load chain-proof signing key: {e}");
            std::process::exit(1);
        }
    };

    let state = AppState {
        pool,
        auth,
        signing_key,
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/:view", get(view_handler))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("fatal: cannot bind {addr}: {e}");
            std::process::exit(1);
        }
    };
    eprintln!("obs-compliance-view listening on {addr}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("fatal: server error: {e}");
        std::process::exit(1);
    }
}

async fn healthz() -> &'static str {
    "ok"
}

/// The signed view body: the payload (what the proof covers) plus the proof footer.
#[derive(serde::Serialize)]
struct ViewEnvelope {
    payload: ViewPayload,
    proof: ProofOut,
}

/// The canonical payload - the bytes the chain-proof signs. Field order is fixed and `summary.by_kind`
/// is a `BTreeMap`, so `serde_json::to_vec` is deterministic and an auditor can re-canonicalise and
/// verify independently.
#[derive(serde::Serialize)]
struct ViewPayload {
    view: &'static str,
    tenant_id: String,
    since_ns: i64,
    until_ns: i64,
    summary: summary::Summary,
    rows: Vec<query::AuditRow>,
}

#[derive(serde::Serialize)]
struct ProofOut {
    signature_hex: String,
    public_key_hex: String,
}

async fn view_handler(
    State(st): State<AppState>,
    Path(view_slug): Path<String>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // 1. Authenticate the auditor JWT and require the external_auditor role (§1 #2).
    let token = match bearer(&headers) {
        Some(t) => t,
        None => return err(StatusCode::UNAUTHORIZED, "missing bearer token"),
    };
    let claims = match st.auth.authorize_auditor(token) {
        Ok(c) => c,
        Err(AuthError::NotAuditor) => {
            return err(StatusCode::FORBIDDEN, "external_auditor role required")
        }
        Err(AuthError::CrossTenant) => return err(StatusCode::FORBIDDEN, "cross-tenant refused"),
        Err(e) => return err(StatusCode::UNAUTHORIZED, &e.to_string()),
    };

    // 2. Resolve the view from the path slug.
    let view = match View::parse(&view_slug) {
        Some(v) => v,
        None => return err(StatusCode::NOT_FOUND, "unknown view"),
    };

    // 3. Enforce tenant scope: an explicit ?tenant_id= must match the JWT tenant (§1 #3).
    if enforce_tenant_scope(
        &claims.tenant_id,
        params.get("tenant_id").map(String::as_str),
    )
    .is_err()
    {
        return err(StatusCode::FORBIDDEN, "cross-tenant refused");
    }
    let tenant_uuid = match Uuid::parse_str(&claims.tenant_id) {
        Ok(u) => u,
        Err(_) => return err(StatusCode::BAD_REQUEST, "tenant_id is not a uuid"),
    };

    // 4. Parse and validate the time window (§1 #6).
    let (since_ns, until_ns) = match parse_window(&params) {
        Ok(w) => w,
        Err(m) => return err(StatusCode::BAD_REQUEST, &m),
    };
    if let Err(e) = window::validate(
        since_ns.div_euclid(NS_PER_SEC),
        until_ns.div_euclid(NS_PER_SEC),
    ) {
        return err(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 5. Query the kind-filtered, tenant-scoped, windowed audit rows (§1 #4).
    let rows =
        match query::fetch_rows(&st.pool, tenant_uuid, view.kinds(), since_ns, until_ns).await {
            Ok(r) => r,
            Err(e) => {
                return err(
                    StatusCode::SERVICE_UNAVAILABLE,
                    &format!("audit store unavailable: {e}"),
                )
            }
        };

    // 6. Summarise, then build the canonical payload.
    let summary = summary::summarize(&rows);
    let payload = ViewPayload {
        view: view.slug(),
        tenant_id: claims.tenant_id.clone(),
        since_ns,
        until_ns,
        summary,
        rows,
    };
    let canonical = match serde_json::to_vec(&payload) {
        Ok(b) => b,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("serialize: {e}"),
            )
        }
    };

    // 7. Defence-in-depth PII scan: the chain stores placeholders, so any raw PII is a sev-1, not a leak.
    if !pii_scan::is_clean(&String::from_utf8_lossy(&canonical)) {
        eprintln!(
            "{{\"sev\":1,\"event\":\"compliance_view_pii_detected\",\"view\":\"{}\",\"tenant\":\"{}\"}}",
            view.slug(),
            claims.tenant_id
        );
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal: response failed PII scan",
        );
    }

    // 8. Sign the canonical bytes with the Ed25519 chain-proof (§1 #5).
    let p = proof::sign(&st.signing_key, &canonical);

    // 9. Append the access-audit row to the memory chain (§1 #10), best-effort: a failed audit write must
    //    not fail the auditor's read. obs.compliance_view_accessed records who read which view over what
    //    window so a later auditor can see the views themselves were accessed.
    {
        let auditor_subject = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
        let audit_body = serde_json::json!({
            "event_type": "obs.compliance_view_accessed",
            "view": view.slug(),
            "tenant_id": claims.tenant_id,
            "auditor": claims.sub,
            "rows": payload.summary.total_rows,
            "since_ns": since_ns,
            "until_ns": until_ns,
        })
        .to_string();
        let audit_path = format!("obs/compliance/{}/{}/accessed", tenant_uuid, view.slug());
        if let Err(e) = cyberos_audit_chain::emit_genesis(
            &st.pool,
            tenant_uuid,
            auditor_subject,
            &audit_path,
            &audit_body,
        )
        .await
        {
            eprintln!(
                "{{\"sev\":2,\"event\":\"compliance_audit_emit_failed\",\"view\":\"{}\",\"error\":\"{}\"}}",
                view.slug(),
                e
            );
        }
    }

    let envelope = ViewEnvelope {
        payload,
        proof: ProofOut {
            signature_hex: p.signature_hex,
            public_key_hex: p.public_key_hex,
        },
    };
    (StatusCode::OK, Json(envelope)).into_response()
}

/// Extract the bearer token from the Authorization header.
fn bearer(headers: &HeaderMap) -> Option<&str> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    raw.strip_prefix("Bearer ").map(str::trim)
}

/// Parse the `since` / `until` RFC3339 params into epoch nanoseconds. Defaults: `until` = now, `since` =
/// now - 90 days, so a bare view request returns a bounded recent window rather than the whole chain.
fn parse_window(params: &HashMap<String, String>) -> Result<(i64, i64), String> {
    let now = chrono::Utc::now();
    let until = match params.get("until") {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| format!("until: {e}"))?
            .with_timezone(&chrono::Utc),
        None => now,
    };
    let since = match params.get("since") {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| format!("since: {e}"))?
            .with_timezone(&chrono::Utc),
        None => now - chrono::Duration::days(DEFAULT_WINDOW_DAYS),
    };
    let since_ns = since
        .timestamp_nanos_opt()
        .ok_or("since out of representable range")?;
    let until_ns = until
        .timestamp_nanos_opt()
        .ok_or("until out of representable range")?;
    Ok((since_ns, until_ns))
}

fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

async fn build_pool() -> Result<PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://cyberos:cyberos@localhost:5432/cyberos".to_string());
    PgPoolOptions::new().max_connections(5).connect(&url).await
}

/// Build the auditor JWT verifier: RS256 from `OBS_COMPLIANCE_JWKS_JSON` if set, else HS256 from
/// `OBS_COMPLIANCE_HS256_SECRET` (dev). No insecure default - one of the two must be present.
fn build_authenticator() -> Result<Authenticator, String> {
    if let Ok(jwks) = std::env::var("OBS_COMPLIANCE_JWKS_JSON") {
        return Authenticator::from_jwks(&jwks).map_err(|e| e.to_string());
    }
    if let Ok(secret) = std::env::var("OBS_COMPLIANCE_HS256_SECRET") {
        if secret.is_empty() {
            return Err("OBS_COMPLIANCE_HS256_SECRET is empty".to_string());
        }
        return Ok(Authenticator::from_hs256_secret(secret.as_bytes()));
    }
    Err("set OBS_COMPLIANCE_JWKS_JSON (prod) or OBS_COMPLIANCE_HS256_SECRET (dev)".to_string())
}

/// Load the 32-byte Ed25519 signing key from `OBS_COMPLIANCE_SIGNING_KEY_HEX` (64 hex chars).
fn signing_key_from_env() -> Result<[u8; 32], String> {
    let hex = std::env::var("OBS_COMPLIANCE_SIGNING_KEY_HEX")
        .map_err(|_| "OBS_COMPLIANCE_SIGNING_KEY_HEX not set".to_string())?;
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(format!(
            "signing key must be 64 hex chars, got {}",
            hex.len()
        ));
    }
    let mut key = [0u8; 32];
    for (i, slot) in key.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|e| format!("signing key hex: {e}"))?;
    }
    Ok(key)
}
