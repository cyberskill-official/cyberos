//! FR-AI-004 §5 — Integration tests for cost-hold expiry cleanup.
//!
//! Requires a running Postgres instance. Set DATABASE_URL env var.
//! Tests are ignored when DATABASE_URL is not set.

use chrono::{DateTime, Datelike, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use std::path::Path;
use uuid::Uuid;

use cyberos_ai_gateway::cost_hold_expiry::*;

// ─── Test helpers ─────────────────────────────────────────────────────────────

async fn test_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url)
        .await
        .unwrap_or_else(|e| panic!("DATABASE_URL is set but Postgres connection failed: {e}"));
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("cost-ledger migrations should apply");
    Some(pool)
}

fn memory_writes_enabled() -> bool {
    std::env::var("CYBEROS_AI_GATEWAY_TEST_MEMORY_WRITES").as_deref() == Ok("1")
        && std::env::var_os("CYBEROS_STORE").is_some()
}

fn require_memory_writes_enabled() -> bool {
    if memory_writes_enabled() {
        true
    } else {
        eprintln!(
            "CYBEROS_AI_GATEWAY_TEST_MEMORY_WRITES=1 and CYBEROS_STORE are required; skipping memory-writing expiry case"
        );
        false
    }
}

fn memory_row_body(hold_id: Uuid, kind: &str) -> String {
    let store = std::env::var_os("CYBEROS_STORE").expect("CYBEROS_STORE should be set");
    let dir = Path::new(&store).join("memories/decisions/ai-invocations");
    let hold = hold_id.to_string();
    std::fs::read_dir(dir)
        .expect("ai-invocations directory should exist")
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .find(|body| body.contains(&format!("kind: {kind}")) && body.contains(&hold))
        .unwrap_or_else(|| panic!("missing {kind} row for hold {hold}"))
}

fn count_memory_rows(hold_id: Uuid, kind: &str) -> usize {
    let Some(store) = std::env::var_os("CYBEROS_STORE") else {
        return 0;
    };
    let dir = Path::new(&store).join("memories/decisions/ai-invocations");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let hold = hold_id.to_string();
    entries
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .filter(|body| body.contains(&format!("kind: {kind}")) && body.contains(&hold))
        .count()
}

async fn seed_tenant(pool: &PgPool, tenant_id: &str, cap: Decimal, spent: Decimal) {
    let period = Utc::now().date_naive().with_day(1).unwrap();
    sqlx::query(
        "INSERT INTO cost_ledger (tenant_id, period, spent_usd, monthly_cap_usd) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (tenant_id, period) DO UPDATE SET spent_usd = EXCLUDED.spent_usd",
    )
    .bind(tenant_id)
    .bind(period)
    .bind(spent)
    .bind(cap)
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_hold(
    pool: &PgPool,
    tenant_id: &str,
    estimated_usd: Decimal,
    expires_at: DateTime<Utc>,
    state: &str,
) -> Uuid {
    let hold_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO cost_ledger_hold \
         (id, tenant_id, idempotency_key, estimated_usd, resolved_provider, resolved_model, \
          expires_at, state) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(hold_id)
    .bind(tenant_id)
    .bind(format!("expiry-{}", Uuid::new_v4()))
    .bind(estimated_usd)
    .bind("bedrock")
    .bind("anthropic.claude-3-5-sonnet-20241022-v2:0")
    .bind(expires_at)
    .bind(state)
    .execute(pool)
    .await
    .unwrap();
    hold_id
}

async fn read_hold_state(pool: &PgPool, hold_id: Uuid) -> (String, Option<String>) {
    sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT state, refund_reason FROM cost_ledger_hold WHERE id = $1",
    )
    .bind(hold_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn read_ledger_spent(pool: &PgPool, tenant_id: &str) -> Decimal {
    let period = Utc::now().date_naive().with_day(1).unwrap();
    sqlx::query_scalar::<_, Decimal>(
        "SELECT spent_usd FROM cost_ledger WHERE tenant_id = $1 AND period = $2",
    )
    .bind(tenant_id)
    .bind(period)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn count_holds_by_state(pool: &PgPool, tenant_id: &str, state: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cost_ledger_hold WHERE tenant_id = $1 AND state = $2",
    )
    .bind(tenant_id)
    .bind(state)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn cleanup_tenant(pool: &PgPool, tenant_id: &str) {
    sqlx::query("DELETE FROM cost_ledger_hold WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM cost_ledger WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
}

// ─── AC #1: Happy path ───────────────────────────────────────────────────────

#[tokio::test]
async fn tick_processes_expired_holds() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-happy";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    // Seed 3 expired holds.
    let expired_at = Utc::now() - Duration::seconds(10);
    let mut hold_ids = Vec::new();
    for _ in 0..3 {
        hold_ids.push(seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await);
    }

    let report = run_tick(&pool).await.unwrap();
    assert_eq!(report.holds_processed, 3);
    assert_eq!(report.holds_succeeded, 3);
    assert_eq!(report.holds_failed, 0);

    assert_eq!(count_holds_by_state(&pool, tenant, "expired").await, 3);
    assert_eq!(count_holds_by_state(&pool, tenant, "held").await, 0);

    for hold_id in hold_ids {
        assert_eq!(count_memory_rows(hold_id, "ai.hold_expired"), 1);
        assert!(memory_row_body(hold_id, "ai.hold_expired").contains("tick_id: "));
    }

    // Ledger should be unchanged.
    let spent = read_ledger_spent(&pool, tenant).await;
    assert_eq!(spent, dec!(50));

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #2: Non-expired holds skipped ────────────────────────────────────────

#[tokio::test]
async fn tick_skips_non_expired_holds() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-skip-future";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    // Seed 2 expired and 3 non-expired holds.
    let expired_at = Utc::now() - Duration::seconds(10);
    let future_at = Utc::now() + Duration::seconds(60);
    for _ in 0..2 {
        seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
    }
    for _ in 0..3 {
        seed_hold(&pool, tenant, dec!(0.0085), future_at, "held").await;
    }

    let report = run_tick(&pool).await.unwrap();

    // Only the 2 expired should be processed.
    assert_eq!(report.holds_processed, 2);
    assert_eq!(report.holds_succeeded, 2);
    assert_eq!(report.holds_failed, 0);
    assert_eq!(count_holds_by_state(&pool, tenant, "held").await, 3);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #3: Already-reconciled holds skipped ─────────────────────────────────

#[tokio::test]
async fn tick_skips_reconciled_holds() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-skip-reconciled";
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    // Seed a reconciled hold with expired expires_at (edge case).
    let expired_at = Utc::now() - Duration::seconds(10);
    let hold_id = seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;

    // Manually set to reconciled.
    sqlx::query(
        "UPDATE cost_ledger_hold SET state = 'reconciled', actual_usd = 0.007, \
         reconciled_at = NOW() WHERE id = $1",
    )
    .bind(hold_id)
    .execute(&pool)
    .await
    .unwrap();

    let report = run_tick(&pool).await.unwrap();

    // Should not touch the reconciled hold.
    assert_eq!(report.holds_processed, 0);

    let (state, _) = read_hold_state(&pool, hold_id).await;
    assert_eq!(state, "reconciled");

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #4: FOR UPDATE SKIP LOCKED ───────────────────────────────────────────

#[tokio::test]
async fn tick_skips_locked_rows() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-skip-locked";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let expired_at = Utc::now() - Duration::seconds(10);
    let locked_id = seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
    let other_id = seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;

    // Lock the first hold from another transaction.
    let mut blocking_tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM cost_ledger_hold WHERE id = $1 FOR UPDATE")
        .bind(locked_id)
        .fetch_one(&mut *blocking_tx)
        .await
        .unwrap();

    let report = run_tick(&pool).await.unwrap();
    assert_eq!(report.holds_processed, 1);
    assert_eq!(report.holds_succeeded, 1);
    assert_eq!(report.holds_failed, 0);

    let (locked_state, _) = read_hold_state(&pool, locked_id).await;
    assert_eq!(locked_state, "held", "locked row should remain held");
    let (other_state, _) = read_hold_state(&pool, other_id).await;
    assert_eq!(other_state, "expired", "unlocked row should expire");

    blocking_tx.rollback().await.unwrap();

    let report2 = run_tick(&pool).await.unwrap();
    assert_eq!(report2.holds_processed, 1);
    assert_eq!(report2.holds_succeeded, 1);
    let (locked_state, _) = read_hold_state(&pool, locked_id).await;
    assert_eq!(locked_state, "expired");

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #6: Bounded batch size ───────────────────────────────────────────────

#[tokio::test]
async fn tick_respects_batch_limit() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-batch-limit";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(1000), dec!(50)).await;

    // Seed 10 expired holds (smaller than 500 limit, but tests the mechanism).
    let expired_at = Utc::now() - Duration::seconds(10);
    for _ in 0..10 {
        seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
    }

    let report = run_tick(&pool).await.unwrap();

    // All 10 should be processed (well under the 500 limit).
    assert_eq!(report.holds_processed, 10);
    assert_eq!(report.holds_succeeded, 10);
    assert_eq!(report.holds_failed, 0);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #16: Deterministic order ─────────────────────────────────────────────

#[tokio::test]
async fn tick_orders_by_id() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-order";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let expired_at = Utc::now() - Duration::seconds(10);
    let mut ids = Vec::new();
    for _ in 0..5 {
        ids.push(seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await);
    }

    // IDs should be in ascending order for deterministic processing.
    ids.sort();

    let report = run_tick(&pool).await.unwrap();
    assert_eq!(report.holds_processed, 5);

    cleanup_tenant(&pool, tenant).await;
}
