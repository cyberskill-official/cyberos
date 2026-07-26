//! Postgres metering_events writer + period SUM (TASK-TEN-206).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axes::MeteringAxis;
use crate::recorder::{validate_quantity, MeteringEvent, RecordError};

/// Insert one event. Duplicate unique key → `Ok(None)`.
pub async fn insert_event(pool: &PgPool, event: &MeteringEvent) -> Result<Option<Uuid>, RecordError> {
    validate_quantity(event.axis, event.quantity)?;
    let axis = event.axis.as_str();
    let unit = match event.axis {
        MeteringAxis::Seats => "seat",
        MeteringAxis::ApiCalls => "request",
        MeteringAxis::AiTokens => "token",
        MeteringAxis::StorageBytes => "byte",
    };
    let qty = event.quantity as i64;
    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO metering_events
          (tenant_id, axis, quantity, unit, idempotency_key, source_service, occurred_at, extra)
        VALUES
          ($1, $2::metering_axis, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (tenant_id, axis, idempotency_key) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&event.tenant_id)
    .bind(axis)
    .bind(qty)
    .bind(unit)
    .bind(&event.idempotency_key)
    .bind(&event.source_service)
    .bind(event.occurred_at)
    .bind(&event.extra)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "metering pg insert failed");
        RecordError::Storage
    })?;
    Ok(row)
}

/// Sum active quantities for tenant+axis since `period_start`.
pub async fn period_sum(
    pool: &PgPool,
    tenant_id: &str,
    axis: MeteringAxis,
    period_start: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let sum: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(quantity), 0)::bigint
        FROM metering_events
        WHERE tenant_id = $1
          AND axis = $2::metering_axis
          AND state = 'active'
          AND occurred_at >= $3
        "#,
    )
    .bind(tenant_id)
    .bind(axis.as_str())
    .bind(period_start)
    .fetch_one(pool)
    .await?;
    Ok(sum.unwrap_or(0).max(0) as u64)
}

/// Drain WAL into Postgres. Returns newly inserted row count.
pub async fn drain_wal_to_pg(
    wal: &std::sync::Mutex<crate::wal_queue::WalQueue>,
    pool: &PgPool,
) -> usize {
    let mut inserted = 0usize;
    loop {
        let item = match wal.lock() {
            Ok(mut q) => q.pop(),
            Err(_) => break,
        };
        let Some(event) = item else {
            break;
        };
        match insert_event(pool, &event).await {
            Ok(Some(_)) => inserted += 1,
            Ok(None) => {}
            Err(e) => tracing::warn!(error = %e, "drain_wal_to_pg skipped"),
        }
    }
    inserted
}
