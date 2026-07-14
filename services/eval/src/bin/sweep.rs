//! `cyberos-eval-sweep` - the TASK-EVAL-001 clause-6 retention/erasure sweeper entry point.
//!
//! DISABLED BY DEFAULT. This is a SEPARATE binary from the eval service (`cyberos-eval`); the service never
//! sweeps. Nothing erases until an operator (or a cron that shells out to this binary) runs it explicitly
//! AND passes `--confirm`. A bare invocation does a dry run: it reports what it would do and erases nothing.
//! This is the "the only automatic erasure is scheduled retention, on the operator's configured policy"
//! posture (clause 6, 11).
//!
//! It reads every `(tenant, category)` retention policy from the EVAL governance Postgres and erases the
//! past-retention derived (L2 / BRAIN) projections in the memory Postgres, NEVER touching `l1_audit_log`
//! (the immutable chain). The erasure is itself appended to L1 (`eval.retention_swept` /
//! `eval.subject_erased`) when an audit pool is configured.
//!
//! USAGE:
//!   cyberos-eval-sweep            # dry run: report policies, erase nothing
//!   cyberos-eval-sweep --confirm  # actually sweep
//!
//! ENV (mirrors `cyberos-eval`'s main):
//!   EVAL_DATABASE_URL        the EVAL governance Postgres (holds retention_policy / data_category). Required.
//!   EVAL_DERIVED_DATABASE_URL the Postgres holding the derived projections + l1_audit_log (memory DB). When
//!                            unset, defaults to EVAL_DATABASE_URL (the single-DB topology auth/memory/eval
//!                            share today).
//!   EVAL_AUDIT_DATABASE_URL  the Postgres holding l1_audit_log for the erasure-audit rows. When unset,
//!                            defaults to EVAL_DERIVED_DATABASE_URL; if that is also unset, the erasure events
//!                            are logged, not chained (the test/local convention).

use cyberos_eval::{db, retention};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cyberos_eval=info".into()),
        )
        .json()
        .init();

    let confirm = std::env::args().any(|a| a == "--confirm");

    let policy_url = std::env::var("EVAL_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("EVAL_DATABASE_URL is required"))?;
    let derived_url =
        std::env::var("EVAL_DERIVED_DATABASE_URL").unwrap_or_else(|_| policy_url.clone());
    let audit_url = std::env::var("EVAL_AUDIT_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("EVAL_DERIVED_DATABASE_URL").ok());

    let policy_pool: db::Pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(&policy_url)
        .await?;
    // Reuse the policy pool handle when the derived DB is the same URL (the colocated topology), so we do not
    // open a second pool against the same database.
    let derived_pool: db::Pool = if derived_url == policy_url {
        policy_pool.clone()
    } else {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect(&derived_url)
            .await?
    };
    let audit_pool: Option<db::Pool> = match audit_url {
        Some(url) if url == policy_url => Some(policy_pool.clone()),
        Some(url) if url == derived_url => Some(derived_pool.clone()),
        Some(url) => Some(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .connect(&url)
                .await?,
        ),
        None => {
            tracing::warn!(
                "no EVAL_AUDIT_DATABASE_URL / EVAL_DERIVED_DATABASE_URL; erasure events are logged, not chained"
            );
            None
        }
    };

    if !confirm {
        // Dry run: prove the entry point is reachable and the policy read works, then stop WITHOUT erasing.
        // We do not call run_retention_sweep (which deletes); we only surface what is configured.
        let count = preview_policy_count(&policy_pool).await?;
        tracing::info!(
            policies = count,
            "DRY RUN: {count} retention policies configured. Re-run with --confirm to erase past-retention \
             derived rows. Nothing was erased."
        );
        println!(
            "DRY RUN: {count} retention policies configured. Nothing erased. Re-run with --confirm to sweep."
        );
        return Ok(());
    }

    let report =
        retention::run_retention_sweep(&policy_pool, &derived_pool, audit_pool.as_ref()).await?;
    tracing::info!(
        rows_erased = report.rows_erased,
        subjects_erased = report.subjects_erased,
        categories_swept = report.categories_swept,
        "retention sweep complete"
    );
    println!(
        "retention sweep complete: {} rows erased across {} categories ({} subjects)",
        report.rows_erased, report.categories_swept, report.subjects_erased
    );
    Ok(())
}

/// Count the configured retention policies for the dry-run report (read-only; nil-tenant admin bypass, same
/// as the sweep's policy read). Never deletes.
async fn preview_policy_count(policy_pool: &db::Pool) -> anyhow::Result<i64> {
    let mut tx = policy_pool.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(uuid::Uuid::nil().to_string())
        .execute(&mut *tx)
        .await?;
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM retention_policy")
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(n)
}
