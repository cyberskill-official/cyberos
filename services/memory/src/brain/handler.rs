//! FR-MEMORY-123 §1 #7 — the `POST /v1/memory/recall` axum handler. A thin HTTP shell over
//! [`crate::brain::recall::recall`]; the access scope, provenance, chain verify, and summaries-first logic
//! all live in `recall`. Mounted in `main.rs` next to `search::search`.
//!
//! Caller resolution (MEM-001, R73): the caller's tenant AND subject are stamped into the request extensions
//! by the `crate::auth::require_auth` middleware from the verified FR-AUTH-004 JWT — never from request
//! headers. The route is mounted only behind that middleware, so the [`Caller`] extension is always present;
//! reading it here keeps a missing identity a fail-closed impossibility rather than a header the caller can
//! forge.

use axum::{
    extract::{Extension, Json as JsonInput, State},
    http::StatusCode,
    response::Json,
};

use super::{recall, Caller, EmbedClient, RecallQuery};
use crate::state::AppState;

/// `POST /v1/memory/recall`. Runs the access-scoped recall for the authenticated caller, and maps errors to
/// status codes: limit>100 -> 400, all backends down -> 503, DB error -> 500. Empty results are a normal
/// 200 `[]`.
pub async fn recall_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    JsonInput(q): JsonInput<RecallQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let gw = EmbedClient::from_env();

    match recall::recall(q, &caller, &state.pg, &gw).await {
        Ok(results) => {
            let body = serde_json::to_value(&results).unwrap_or_else(|_| {
                serde_json::json!({
                    "items": [], "degraded_backends": []
                })
            });
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
