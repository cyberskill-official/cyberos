//! Device registration (FR-CHAT-101 slice 4): a subject registers a push token; the actual APNS/FCM send is
//! a deploy-time integration (see push.rs). Tokens are per-subject and upserted on conflict.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Deserialize)]
pub struct RegisterDevice {
    pub platform: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct Device {
    pub id: Uuid,
    pub platform: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn register(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RegisterDevice>,
) -> Result<(StatusCode, Json<Device>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let platform = body.platform.trim().to_string();
    let token = body.token.trim().to_string();
    if platform.is_empty() || token.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "platform and token are required".to_string()));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let row: (Uuid, String, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO chat_devices (tenant_id, subject_id, platform, token)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (subject_id, token) DO UPDATE SET platform = EXCLUDED.platform
         RETURNING id, platform, created_at",
    )
    .bind(tenant)
    .bind(subject)
    .bind(&platform)
    .bind(&token)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    audit::emit(
        &st,
        tenant,
        subject,
        "chat.device_registered",
        serde_json::json!({"platform": platform}),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(Device {
            id: row.0,
            platform: row.1,
            created_at: row.2,
        }),
    ))
}
