//! Postgres pool plus per-request tenant scoping, and small membership helpers. Every read and write runs
//! inside a transaction that first sets the RLS GUC, so a caller can only touch rows in their own tenant.

use uuid::Uuid;

pub type Pool = sqlx::PgPool;

/// Begin a transaction scoped to `tenant` via the transaction-local RLS GUC.
pub async fn tenant_tx<'a>(
    pool: &'a Pool,
    tenant: &Uuid,
) -> Result<sqlx::Transaction<'a, sqlx::Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

/// Channel roles, most-privileged first: owner > admin > member.
pub fn is_manager(role: &str) -> bool {
    role == "owner" || role == "admin"
}

/// The caller's role in a channel, or None if not a member. Assumes the tenant GUC is already set.
pub async fn role_in_channel(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    channel: Uuid,
    subject: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM chat_channel_members WHERE channel_id = $1 AND subject_id = $2",
    )
    .bind(channel)
    .bind(subject)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|r| r.0))
}
