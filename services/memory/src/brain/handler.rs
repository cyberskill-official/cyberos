//! FR-MEMORY-123 §1 #7 — the `POST /v1/memory/recall` axum handler. A thin HTTP shell over
//! [`crate::brain::recall::recall`]; the access scope, provenance, chain verify, and summaries-first logic
//! all live in `recall`. Mounted in `main.rs` next to `search::search`.
//!
//! Caller resolution: the memory service scopes tenant by the `x-tenant-id` header in this slice (mirroring
//! `search.rs`), and reads the caller's own subject from `x-subject-id`. Both move to the FR-AUTH-004 JWT
//! Extension when the memory service grows its JWT-verify middleware (the same migration `search.rs` notes).
//! The caller subject is REQUIRED: without it there is no FR-EVAL-001 viewer identity, so recall fails closed
//! (a missing subject can see nothing) rather than defaulting to a wildcard.

use axum::{
    extract::{Json as JsonInput, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use uuid::Uuid;

use super::{recall, Caller, EmbedClient, RecallQuery};
use crate::state::AppState;

/// `POST /v1/memory/recall`. Resolves the caller, runs the access-scoped recall, and maps errors to status
/// codes: limit>100 -> 400, all backends down -> 503, DB error -> 500. Empty results are a normal 200 `[]`.
pub async fn recall_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    JsonInput(q): JsonInput<RecallQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let tenant_id = require_header_uuid(&headers, "x-tenant-id")
        .ok_or_else(|| bad("x-tenant-id header required (UUID)"))?;
    let viewer_subject_id = require_header_uuid(&headers, "x-subject-id")
        .ok_or_else(|| bad("x-subject-id header required (UUID) — recall needs the caller identity"))?;

    let caller = Caller {
        tenant_id,
        viewer_subject_id,
    };
    let gw = EmbedClient::from_env();

    match recall::recall(q, &caller, &state.pg, &gw).await {
        Ok(results) => {
            let body = serde_json::to_value(&results).unwrap_or_else(|_| serde_json::json!({
                "items": [], "degraded_backends": []
            }));
            Ok((StatusCode::OK, Json(body)))
        }
        Err(recall::RecallError::LimitTooLarge) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "limit_too_large", "max": super::MAX_RECALL_LIMIT})),
        )),
        Err(recall::RecallError::AllBackendsDown) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "recall_backends_unavailable"})),
        )),
        Err(recall::RecallError::Db(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("recall failed: {e}")})),
        )),
    }
}

fn require_header_uuid(headers: &HeaderMap, name: &str) -> Option<Uuid> {
    headers
        .get(name)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

fn bad(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg })),
    )
}
