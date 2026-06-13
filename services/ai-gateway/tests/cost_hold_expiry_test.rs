//! FR-AI-004 §5 — Integration tests for cost-hold expiry cleanup.
//!
//! Requires a running Postgres instance. Set DATABASE_URL env var.
//! Tests are ignored when DATABASE_URL is not set.

use chrono::{DateTime, Datelike, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use std::{
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration as StdDuration, Instant},
};
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
    memory_row_body_containing(kind, &hold_id.to_string())
}

fn memory_row_body_containing(kind: &str, needle: &str) -> String {
    let store = std::env::var_os("CYBEROS_STORE").expect("CYBEROS_STORE should be set");
    let dir = Path::new(&store).join("memories/decisions/ai-invocations");
    std::fs::read_dir(dir)
        .expect("ai-invocations directory should exist")
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .find(|body| body.contains(&format!("kind: {kind}\n")) && body.contains(needle))
        .unwrap_or_else(|| panic!("missing {kind} row containing {needle}"))
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
        .filter(|body| body.contains(&format!("kind: {kind}\n")) && body.contains(&hold))
        .count()
}

fn metric_counter_value(name: &str, labels: &[(&str, &str)]) -> f64 {
    prometheus::gather()
        .into_iter()
        .find(|family| family.get_name() == name)
        .and_then(|family| {
            family
                .get_metric()
                .iter()
                .find(|metric| {
                    labels.iter().all(|(want_name, want_value)| {
                        metric.get_label().iter().any(|label| {
                            label.get_name() == *want_name && label.get_value() == *want_value
                        })
                    })
                })
                .map(|metric| metric.get_counter().get_value())
        })
        .unwrap_or(0.0)
}

fn metric_gauge_value(name: &str) -> f64 {
    prometheus::gather()
        .into_iter()
        .find(|family| family.get_name() == name)
        .and_then(|family| {
            family
                .get_metric()
                .first()
                .map(|metric| metric.get_gauge().get_value())
        })
        .unwrap_or(0.0)
}

fn metric_histogram_count(name: &str) -> u64 {
    prometheus::gather()
        .into_iter()
        .find(|family| family.get_name() == name)
        .and_then(|family| {
            family
                .get_metric()
                .first()
                .map(|metric| metric.get_histogram().get_sample_count())
        })
        .unwrap_or(0)
}

fn expiry_binary_path() -> Option<String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_cost-hold-expiry") {
        return Some(path);
    }
    let current = std::env::current_exe().ok()?;
    let debug_dir = current.parent()?.parent()?;
    let candidate = debug_dir.join("cost-hold-expiry");
    candidate
        .is_file()
        .then(|| candidate.to_string_lossy().to_string())
}

fn spawn_expiry_binary(database_url: &str) -> Option<Child> {
    let bin = expiry_binary_path()?;
    let mut cmd = Command::new(bin);
    cmd.env("DATABASE_URL", database_url)
        .env("CYBEROS_AI_EXPIRY_TICK_SECONDS", "5")
        .env(
            "PYTHONPATH",
            "/Users/stephencheng/Projects/CyberSkill/cyberos/target/codex-python-memory:/Users/stephencheng/Projects/CyberSkill/cyberos/modules/memory",
        )
        .env(
            "CYBEROS_STORE",
            std::env::var("CYBEROS_STORE").unwrap_or_else(|_| {
                "/Users/stephencheng/Projects/CyberSkill/cyberos/target/fr-ai-002-memory".to_string()
            }),
        )
        .current_dir("/Users/stephencheng/Projects/CyberSkill/cyberos")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    Some(cmd.spawn().expect("cost-hold-expiry binary should spawn"))
}

fn terminate_child(child: &mut Child) {
    let _ = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status();
}

fn wait_for_child(child: &mut Child, timeout: StdDuration) -> Option<ExitStatus> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            return Some(status);
        }
        if started.elapsed() >= timeout {
            return None;
        }
        std::thread::sleep(StdDuration::from_millis(50));
    }
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

// ─── AC #5: Memory failure rolls back one hold ───────────────────────────────

#[tokio::test]
async fn tick_memory_failure_rolls_back_one_hold_and_continues() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let tenant = "test:expiry-memory-failure";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let expired_at = Utc::now() - Duration::seconds(10);
    let first = seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
    let failed = seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
    let third = seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;

    std::env::set_var("CYBEROS_AI_EXPIRY_FAIL_HOLD_ID", failed.to_string());
    let report = run_tick(&pool).await.unwrap();
    std::env::remove_var("CYBEROS_AI_EXPIRY_FAIL_HOLD_ID");

    assert_eq!(report.holds_processed, 3);
    assert_eq!(report.holds_succeeded, 2);
    assert_eq!(report.holds_failed, 1);

    assert_eq!(read_hold_state(&pool, first).await.0, "expired");
    assert_eq!(read_hold_state(&pool, failed).await.0, "held");
    assert_eq!(read_hold_state(&pool, third).await.0, "expired");
    assert_eq!(count_memory_rows(failed, "ai.hold_expired"), 0);
    assert_eq!(count_memory_rows(failed, "ai.hold_expired_completed"), 0);

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

    // Seed the declared 5,000 expired holds; one tick must process only 500.
    let expired_at = Utc::now() - Duration::seconds(10);
    for _ in 0..5_000 {
        seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
    }

    let report = run_tick(&pool).await.unwrap();

    assert_eq!(report.holds_processed, 500);
    assert_eq!(report.holds_succeeded, 500);
    assert_eq!(report.holds_failed, 0);
    assert_eq!(count_holds_by_state(&pool, tenant, "expired").await, 500);
    assert_eq!(count_holds_by_state(&pool, tenant, "held").await, 4_500);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #7: Graceful shutdown ────────────────────────────────────────────────

#[tokio::test]
async fn binary_sigterm_exits_zero() {
    let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
        eprintln!("DATABASE_URL not set; skipping");
        return;
    };
    if !require_memory_writes_enabled() {
        return;
    }
    let Some(mut child) = spawn_expiry_binary(&database_url) else {
        eprintln!("CARGO_BIN_EXE_cost-hold-expiry not set; skipping binary SIGTERM test");
        return;
    };

    std::thread::sleep(StdDuration::from_millis(600));
    terminate_child(&mut child);
    let status = wait_for_child(&mut child, StdDuration::from_secs(5))
        .expect("cost-hold-expiry should exit within 5 seconds of SIGTERM");
    assert!(status.success(), "SIGTERM should produce exit 0: {status}");
}

// ─── AC #8: Initial DB outage retries instead of crashing ───────────────────

#[tokio::test]
async fn binary_retries_when_database_unavailable() {
    if !require_memory_writes_enabled() {
        return;
    }
    let Some(mut child) =
        spawn_expiry_binary("postgres://cyberos:cyberos@127.0.0.1:1/cyberos_ai_test")
    else {
        eprintln!("CARGO_BIN_EXE_cost-hold-expiry not set; skipping binary DB retry test");
        return;
    };

    std::thread::sleep(StdDuration::from_millis(1400));
    assert!(
        child.try_wait().unwrap().is_none(),
        "binary should keep retrying instead of crashing while DB is unavailable"
    );
    terminate_child(&mut child);
    let status = wait_for_child(&mut child, StdDuration::from_secs(5))
        .expect("retrying binary should handle SIGTERM");
    assert!(
        status.success(),
        "SIGTERM during DB retry should exit 0: {status}"
    );
}

// ─── AC #9: Crash after memory emit, before DB commit ───────────────────────

#[tokio::test]
async fn crash_after_memory_emit_reemits_on_next_tick() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
        eprintln!("DATABASE_URL not set; skipping");
        return;
    };
    if !require_memory_writes_enabled() {
        return;
    }

    let tenant = "test:expiry-crash-restart";
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;
    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        Utc::now() - Duration::seconds(10),
        "held",
    )
    .await;

    let Some(bin) = expiry_binary_path() else {
        eprintln!("CARGO_BIN_EXE_cost-hold-expiry not set; skipping crash restart test");
        cleanup_tenant(&pool, tenant).await;
        return;
    };
    let mut child = Command::new(bin)
        .env("DATABASE_URL", &database_url)
        .env("CYBEROS_AI_EXPIRY_TICK_SECONDS", "5")
        .env("CYBEROS_AI_EXPIRY_EXIT_AFTER_HOLD_EMIT", hold_id.to_string())
        .env(
            "PYTHONPATH",
            "/Users/stephencheng/Projects/CyberSkill/cyberos/target/codex-python-memory:/Users/stephencheng/Projects/CyberSkill/cyberos/modules/memory",
        )
        .env("CYBEROS_STORE", std::env::var("CYBEROS_STORE").unwrap())
        .current_dir("/Users/stephencheng/Projects/CyberSkill/cyberos")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("crash-injection binary should spawn");
    let status = wait_for_child(&mut child, StdDuration::from_secs(10))
        .expect("crash-injection binary should exit");
    assert_eq!(status.code(), Some(42));

    assert_eq!(read_hold_state(&pool, hold_id).await.0, "held");
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired"), 1);
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired_completed"), 0);

    let report = run_tick(&pool).await.unwrap();
    assert_eq!(report.holds_succeeded, 1);
    assert_eq!(read_hold_state(&pool, hold_id).await.0, "expired");
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired"), 2);
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired_completed"), 1);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #10/#12/#14: Metrics emitted ────────────────────────────────────────

#[tokio::test]
async fn tick_emits_required_metrics() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    if !require_memory_writes_enabled() {
        return;
    }

    let tenant = "test:expiry-metrics";
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let processed_before = metric_counter_value(
        "ai_expiry_holds_processed_total",
        &[("result", "succeeded")],
    );
    let succeeded_before = metric_counter_value("ai_expiry_holds_succeeded_total", &[]);
    let ticks_before = metric_counter_value("ai_expiry_ticks_total", &[("result", "success")]);
    let histogram_before = metric_histogram_count("ai_expiry_tick_duration_seconds");

    for batch in [4, 3, 3] {
        let expired_at = Utc::now() - Duration::seconds(10);
        for _ in 0..batch {
            seed_hold(&pool, tenant, dec!(0.0085), expired_at, "held").await;
        }
        let report = run_tick(&pool).await.unwrap();
        assert_eq!(report.holds_failed, 0);
    }

    assert_eq!(
        metric_counter_value(
            "ai_expiry_holds_processed_total",
            &[("result", "succeeded")]
        ) - processed_before,
        10.0
    );
    assert_eq!(
        metric_counter_value("ai_expiry_holds_succeeded_total", &[]) - succeeded_before,
        10.0
    );
    assert_eq!(
        metric_counter_value("ai_expiry_ticks_total", &[("result", "success")]) - ticks_before,
        3.0
    );
    assert!(
        metric_histogram_count("ai_expiry_tick_duration_seconds") >= histogram_before + 3,
        "tick duration histogram should observe each tick"
    );
    assert_eq!(metric_gauge_value("ai_expiry_consecutive_failures"), 0.0);
    assert_eq!(metric_gauge_value("cleanup_holds_pending_gauge"), 0.0);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #15: Pair-write ordering ────────────────────────────────────────────

#[tokio::test]
async fn tick_pair_write_order_is_start_update_commit_completed() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    if !require_memory_writes_enabled() {
        return;
    }

    let tenant = "test:expiry-pair-write";
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;
    clear_expiry_event_log();

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        Utc::now() - Duration::seconds(10),
        "held",
    )
    .await;

    let report = run_tick(&pool).await.unwrap();
    assert_eq!(report.holds_succeeded, 1);

    let events: Vec<&'static str> = expiry_event_log_snapshot()
        .into_iter()
        .filter(|record| record.hold_id == hold_id)
        .map(|record| record.event)
        .collect();
    assert_eq!(
        events,
        vec![
            "hold_expired_started",
            "sql_update",
            "commit",
            "hold_expired_completed"
        ]
    );
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired_started"), 1);
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired"), 1);
    assert_eq!(count_memory_rows(hold_id, "ai.hold_expired_completed"), 1);

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

    let first_id = ids.first().unwrap().to_string();
    let cleanup_body = memory_row_body_containing("ai.cleanup_run_completed", &first_id);
    let expected_ids = serde_json::to_string(&ids).unwrap();
    assert!(
        cleanup_body.contains(&format!("expired_hold_ids: {expected_ids}")),
        "cleanup row should preserve sorted expired_hold_ids: {cleanup_body}"
    );

    cleanup_tenant(&pool, tenant).await;
}
