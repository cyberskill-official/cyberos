//! Idempotency-Key handling for admin POST endpoints.
//!
//! For now this is just the schema; a more complete impl lands with
//! FR-AUTH-005 (admin REST surface).

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Stores a fresh idempotent response so subsequent retries return it.
pub async fn record(
    pool: &PgPool,
    idempotency_key: &str,
    route: &str,
    tenant_id: Uuid,
    response_status: i16,
    response_body: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO admin_idempotency
                (idempotency_key, route, tenant_id, response_status, response_body)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (idempotency_key, route, tenant_id) DO NOTHING",
    )
    .bind(idempotency_key)
    .bind(route)
    .bind(tenant_id)
    .bind(response_status)
    .bind(response_body)
    .execute(pool)
    .await
    .map(|_| ())
}

/// Look up a prior idempotent response.
pub async fn lookup(
    pool: &PgPool,
    idempotency_key: &str,
    route: &str,
    tenant_id: Uuid,
) -> Result<Option<(i16, Value)>, sqlx::Error> {
    sqlx::query_as::<_, (i16, Value)>(
        "SELECT response_status, response_body
           FROM admin_idempotency
          WHERE idempotency_key = $1
            AND route = $2
            AND tenant_id = $3
            AND expires_at > NOW()",
    )
    .bind(idempotency_key)
    .bind(route)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
}
