//! Postgres pool plus per-request tenant scoping. Every read and write that must respect tenant
//! isolation runs inside a transaction that first sets the RLS GUC (app.current_tenant_id, TASK-AUTH-003),
//! so a caller can only touch rows in their own tenant. Mirrors `cyberos_chat::db`.

use uuid::Uuid;

pub type Pool = sqlx::PgPool;

/// Begin a transaction scoped to `tenant` via the transaction-local RLS GUC. The third `set_config`
/// argument `true` makes the setting local to the transaction, so it does not leak across pooled
/// connections.
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
