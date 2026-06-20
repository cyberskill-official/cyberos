//! Read-only memory-audit query for the compliance views (FR-OBS-008 §1 #4). Queries `l1_audit_log` by
//! tenant + the view's audit kinds (the generated `event_type` column from migration 0004) + the time
//! window, ordered by seq. Read-only - never mutates (DEC-177). Filters by `tenant_id` explicitly and
//! also sets the per-transaction RLS GUC as defence-in-depth.

use sqlx::PgPool;
use uuid::Uuid;

/// One audit row in a compliance view. The chain already stores PII as placeholders; `pii_scan` is the
/// defence check before the response is served.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditRow {
    pub seq: i64,
    pub event_type: Option<String>,
    pub op: String,
    pub path: String,
    pub subject_id: Option<String>,
    pub chain_anchor_hex: String,
    pub ts_ns: i64,
}

type Row = (
    i64,
    Option<String>,
    String,
    String,
    Option<Uuid>,
    String,
    i64,
);

/// Fetch the rows a view selects: tenant-scoped, kind-filtered (`event_type`), time-windowed.
pub async fn fetch_rows(
    pool: &PgPool,
    tenant_id: Uuid,
    kinds: &[&str],
    since_ns: i64,
    until_ns: i64,
) -> Result<Vec<AuditRow>, sqlx::Error> {
    let kinds_owned: Vec<String> = kinds.iter().map(|k| (*k).to_string()).collect();

    let mut tx = pool.begin().await?;
    // RLS defence-in-depth: scope this transaction to the tenant.
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;

    let rows: Vec<Row> = sqlx::query_as(
        "SELECT seq, event_type, op, path, subject_id, chain_anchor_hex, ts_ns
           FROM l1_audit_log
          WHERE tenant_id = $1
            AND event_type = ANY($2)
            AND ts_ns BETWEEN $3 AND $4
          ORDER BY seq",
    )
    .bind(tenant_id)
    .bind(&kinds_owned)
    .bind(since_ns)
    .bind(until_ns)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(rows
        .into_iter()
        .map(
            |(seq, event_type, op, path, subject_id, chain_anchor_hex, ts_ns)| AuditRow {
                seq,
                event_type,
                op,
                path,
                subject_id: subject_id.map(|u| u.to_string()),
                chain_anchor_hex,
                ts_ns,
            },
        )
        .collect())
}
