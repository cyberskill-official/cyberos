//! `POST /v1/webhooks/wise/{profile_id}` — signature auth, fast 200 via WAL (DEC-850).

use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use chrono::Utc;
use tracing::warn;

use super::parser::{is_stale, parse_event_type, profile_id_mismatch, WiseEvent};
use super::public_key::{CachedPublicKeys, PublicKeySource};
use super::signature::verify_signature;
use super::wal::{WiseWalError, WiseWalItem, WiseWalQueue};
use super::processor::WiseProcessor;

/// 1 MiB defensive body cap (INV-004 §11.4).
pub const MAX_WEBHOOK_BODY: usize = 1024 * 1024;

#[derive(Debug)]
pub struct WiseHostState<S: PublicKeySource> {
    pub keys: Arc<CachedPublicKeys<S>>,
    pub wal: Arc<Mutex<WiseWalQueue>>,
    pub processor: Arc<WiseProcessor>,
}

pub fn router<S: PublicKeySource + 'static>(state: Arc<WiseHostState<S>>) -> Router {
    Router::new()
        .route(
            "/v1/webhooks/wise/:profile_id",
            post(wise_webhook::<S>),
        )
        .layer(DefaultBodyLimit::max(MAX_WEBHOOK_BODY))
        .with_state(state)
}

async fn wise_webhook<S: PublicKeySource + 'static>(
    State(state): State<Arc<WiseHostState<S>>>,
    Path(profile_id): Path<i64>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let sig = match headers
        .get("X-Signature-SHA256")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) if !s.is_empty() => s,
        _ => return StatusCode::UNAUTHORIZED,
    };

    let pem = match state.keys.get_or_fetch(profile_id) {
        Ok(p) => p,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };

    let verified = match verify_signature(&pem, &body, sig) {
        Ok(()) => true,
        Err(_) => match state.keys.force_refresh(profile_id) {
            Ok(fresh) => verify_signature(&fresh, &body, sig).is_ok(),
            Err(_) => false,
        },
    };
    if !verified {
        return StatusCode::UNAUTHORIZED;
    }

    let event: WiseEvent = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(_) => {
            state
                .processor
                .dead_letter(profile_id, "parse-error".into(), "schema_invalid");
            return StatusCode::OK;
        }
    };

    let Some(event_type) = parse_event_type(&event.event_type) else {
        let eid = resolve_event_id(&event, &headers);
        state
            .processor
            .dead_letter(profile_id, eid, "unknown_event_type");
        return StatusCode::OK;
    };

    if is_stale(event.data.occurred_at, Utc::now()) {
        // DEC-844 — 200 so Wise stops retrying; no WAL / processor receipt.
        return StatusCode::OK;
    }

    if profile_id_mismatch(profile_id, event.data.resource.profile_id) {
        return StatusCode::BAD_REQUEST;
    }

    let event_id = resolve_event_id(&event, &headers);
    let item = WiseWalItem {
        profile_id,
        event_id,
        event_type,
        body: body.to_vec(),
    };

    match state.wal.lock() {
        Ok(mut q) => {
            if let Err(WiseWalError::Overflow) = q.push(item) {
                warn!("wise wal overflow");
                return StatusCode::SERVICE_UNAVAILABLE;
            }
        }
        Err(e) => {
            warn!(error = %e, "wise wal lock poisoned");
            return StatusCode::SERVICE_UNAVAILABLE;
        }
    }

    StatusCode::OK
}

fn resolve_event_id(event: &WiseEvent, headers: &HeaderMap) -> String {
    if let Some(id) = event.event_id.as_ref().filter(|s| !s.is_empty()) {
        return id.clone();
    }
    if let Some(v) = headers.get("X-Delivery-Id").and_then(|v| v.to_str().ok()) {
        if !v.is_empty() {
            return v.to_string();
        }
    }
    // Host-c fallback: resource id + occurred_at (stable for retries without UUID).
    format!(
        "{}:{}",
        event.data.resource.id,
        event.data.occurred_at.to_rfc3339()
    )
}
