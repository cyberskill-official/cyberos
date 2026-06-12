//! FR-AI-002 §5 — Integration tests for post-call cost reconciliation.
//!
//! Requires a running Postgres instance. Set DATABASE_URL env var.
//! Tests are ignored when DATABASE_URL is not set.

use std::path::Path;

use chrono::Datelike;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use uuid::Uuid;

use cyberos_ai_gateway::cost_reconcile::*;
use cyberos_ai_gateway::cost_table;

const AGENT_PERSONA: &str = "cuo-cpo@0.4.1";
const MODEL_ALIAS: &str = "chat.smart";

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

fn init_cost_table_for_tests() {
    let fixture_path = std::path::PathBuf::from("tests/fixtures/cost_table/valid_rates.yaml");
    let _ = futures::executor::block_on(cost_table::init_cost_table(&fixture_path));
}

async fn seed_tenant(pool: &PgPool, tenant_id: &str, cap: Decimal, spent: Decimal) {
    let period = chrono::Utc::now().date_naive().with_day(1).unwrap();
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
    resolved_provider: &str,
    resolved_model: &str,
) -> Uuid {
    let hold_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO cost_ledger_hold \
         (id, tenant_id, idempotency_key, estimated_usd, agent_persona, model_alias, \
          resolved_provider, resolved_model, expires_at, state) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW() + INTERVAL '60 seconds', 'held')",
    )
    .bind(hold_id)
    .bind(tenant_id)
    .bind(format!("recon-{}", Uuid::new_v4()))
    .bind(estimated_usd)
    .bind(AGENT_PERSONA)
    .bind(MODEL_ALIAS)
    .bind(resolved_provider)
    .bind(resolved_model)
    .execute(pool)
    .await
    .unwrap();
    hold_id
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
            "CYBEROS_AI_GATEWAY_TEST_MEMORY_WRITES=1 and CYBEROS_STORE are required; skipping memory-writing reconcile case"
        );
        false
    }
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

async fn read_ledger_spent(pool: &PgPool, tenant_id: &str) -> Decimal {
    let period = chrono::Utc::now().date_naive().with_day(1).unwrap();
    sqlx::query_scalar::<_, Decimal>(
        "SELECT spent_usd FROM cost_ledger WHERE tenant_id = $1 AND period = $2",
    )
    .bind(tenant_id)
    .bind(period)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn read_hold_state(
    pool: &PgPool,
    hold_id: Uuid,
) -> (String, Option<Decimal>, Option<String>) {
    sqlx::query_as::<_, (String, Option<Decimal>, Option<String>)>(
        "SELECT state, actual_usd, refund_reason FROM cost_ledger_hold WHERE id = $1",
    )
    .bind(hold_id)
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

// ─── AC #1: Happy path (success) ─────────────────────────────────────────────

#[tokio::test]
async fn reconcile_success_updates_ledger() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-success";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(12.50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    let outcome = reconcile(
        hold_id,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 120,
                completion_tokens: 450,
            },
            latency_ms: 850,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_abc123".to_string(),
        },
        &pool,
    )
    .await;

    match outcome {
        Ok(ReconcileOutcome::Reconciled {
            actual_usd,
            new_spent_total_usd,
            ..
        }) => {
            assert!(actual_usd > dec!(0));
            assert!(new_spent_total_usd > dec!(12.50));
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(other) => panic!("expected Reconciled, got {:?}", other),
    }

    // Verify hold state transition.
    let (state, actual_usd, _) = read_hold_state(&pool, hold_id).await;
    if state == "reconciled" {
        assert!(actual_usd.is_some());
    }
    assert_eq!(count_memory_rows(hold_id, "ai.invocation"), 1);
    let body = memory_row_body(hold_id, "ai.invocation");
    assert!(body.contains("agent_persona: cuo-cpo@0.4.1"));
    assert!(body.contains("model_alias: chat.smart"));
    assert!(body.contains("provider_request_id: prv_abc123"));

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #2: Idempotent retry ─────────────────────────────────────────────────

#[tokio::test]
async fn reconcile_idempotent_double_call() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-idempotent";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    let outcome1 = reconcile(
        hold_id,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 120,
                completion_tokens: 450,
            },
            latency_ms: 850,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_abc123".to_string(),
        },
        &pool,
    )
    .await;
    assert!(
        matches!(outcome1, Ok(ReconcileOutcome::Reconciled { .. })),
        "first call should reconcile: {outcome1:?}"
    );

    // Second call should return AlreadyFinalised.
    let outcome2 = reconcile(
        hold_id,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 120,
                completion_tokens: 450,
            },
            latency_ms: 850,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_abc123".to_string(),
        },
        &pool,
    )
    .await;

    match outcome2 {
        Err(ReconcileError::AlreadyFinalised { current_state, .. }) => {
            assert_eq!(current_state, "reconciled");
        }
        other => panic!("expected AlreadyFinalised, got {:?}", other),
    }

    // Verify no double-counting: spent_usd should not have been incremented twice.
    let spent = read_ledger_spent(&pool, tenant).await;
    assert!(
        spent <= dec!(50.01),
        "spend should not be double-counted: {spent}"
    );
    assert_eq!(count_memory_rows(hold_id, "ai.invocation"), 1);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #3: Provider error refund ────────────────────────────────────────────

#[tokio::test]
async fn reconcile_provider_error_refunds() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-refund";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    let outcome = reconcile(
        hold_id,
        CallOutcome::ProviderError {
            http_status: 503,
            retryable: true,
            provider_error_message: "service temporarily unavailable".to_string(),
        },
        &pool,
    )
    .await;

    match outcome {
        Ok(ReconcileOutcome::Refunded {
            hold_estimated_usd,
            reason,
        }) => {
            assert_eq!(hold_estimated_usd, dec!(0.0085));
            assert_eq!(reason, RefundReason::ProviderError { http_status: 503 });
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(other) => panic!("expected Refunded, got {:?}", other),
    }

    // Ledger should be unchanged.
    let spent = read_ledger_spent(&pool, tenant).await;
    assert_eq!(spent, dec!(50), "ledger should not change on refund");

    // Hold should be refunded.
    let (state, _, _) = read_hold_state(&pool, hold_id).await;
    assert_eq!(state, "refunded");
    assert_eq!(count_memory_rows(hold_id, "ai.invocation_failed"), 1);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #4: Cancelled with partial stream ────────────────────────────────────

#[tokio::test]
async fn reconcile_cancelled_partial_charges_partial() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-cancel-partial";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    let outcome = reconcile(
        hold_id,
        CallOutcome::Cancelled {
            partial_usage: Some(ProviderUsage {
                prompt_tokens: 120,
                completion_tokens: 200,
            }),
            reason: CancelReason::ClientDisconnect,
        },
        &pool,
    )
    .await;

    match outcome {
        Ok(ReconcileOutcome::Reconciled { actual_usd, .. }) => {
            // Should charge for 120 prompt + 200 completion tokens.
            assert!(actual_usd > dec!(0));
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(other) => panic!("expected Reconciled, got {:?}", other),
    }

    let (state, actual_usd, _) = read_hold_state(&pool, hold_id).await;
    if state == "reconciled" {
        assert!(actual_usd.is_some());
        assert!(
            actual_usd.unwrap() >= dec!(0.0001),
            "floor at column precision"
        );
    }
    assert_eq!(count_memory_rows(hold_id, "ai.invocation"), 1);
    assert!(memory_row_body(hold_id, "ai.invocation").contains("cancelled: true"));

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #5: Cancelled with no stream ─────────────────────────────────────────

#[tokio::test]
async fn reconcile_cancelled_no_stream_refunds() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-cancel-none";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    let outcome = reconcile(
        hold_id,
        CallOutcome::Cancelled {
            partial_usage: None,
            reason: CancelReason::TimeoutBeforeFirstToken,
        },
        &pool,
    )
    .await;

    match outcome {
        Ok(ReconcileOutcome::Refunded { reason, .. }) => {
            assert_eq!(reason, RefundReason::ProviderUnreachable);
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(other) => panic!("expected Refunded, got {:?}", other),
    }

    // Ledger should be unchanged.
    let spent = read_ledger_spent(&pool, tenant).await;
    assert_eq!(spent, dec!(50));
    assert_eq!(count_memory_rows(hold_id, "ai.invocation_failed"), 1);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #8: Cost-table missing ───────────────────────────────────────────────

#[tokio::test]
async fn reconcile_cost_table_missing_errors() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-no-cost";
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "fake_provider",
        "nonexistent_model",
    )
    .await;

    let outcome = reconcile(
        hold_id,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 100,
                completion_tokens: 200,
            },
            latency_ms: 500,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_xyz".to_string(),
        },
        &pool,
    )
    .await;

    match outcome {
        Err(ReconcileError::CostTableMissing { provider, model }) => {
            assert_eq!(provider, "fake_provider");
            assert_eq!(model, "nonexistent_model");
        }
        other => panic!("expected CostTableMissing, got {:?}", other),
    }

    // Hold should still be in 'held' state (transaction rolled back).
    let (state, _, _) = read_hold_state(&pool, hold_id).await;
    assert_eq!(state, "held", "hold should remain held on CostTableMissing");
    assert_eq!(count_memory_rows(hold_id, "ai.invocation"), 0);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #7: Warn-threshold crossing de-dupes ────────────────────────────────

#[tokio::test]
async fn reconcile_warn_threshold_crosses_once() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-warn";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(79.90)).await;

    let first_hold = seed_hold(
        &pool,
        tenant,
        dec!(1.0000),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;
    let first = reconcile(
        first_hold,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 10_000,
                completion_tokens: 10_000,
            },
            latency_ms: 50,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_warn_1".to_string(),
        },
        &pool,
    )
    .await
    .unwrap();
    match first {
        ReconcileOutcome::Reconciled { warn_crossed, .. } => assert!(warn_crossed),
        other => panic!("expected Reconciled, got {:?}", other),
    }

    let second_hold = seed_hold(
        &pool,
        tenant,
        dec!(1.0000),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;
    let second = reconcile(
        second_hold,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 100,
                completion_tokens: 100,
            },
            latency_ms: 50,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_warn_2".to_string(),
        },
        &pool,
    )
    .await
    .unwrap();
    match second {
        ReconcileOutcome::Reconciled { warn_crossed, .. } => assert!(!warn_crossed),
        other => panic!("expected Reconciled, got {:?}", other),
    }

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #11: HTTP 400 from provider triggers refund ──────────────────────────

#[tokio::test]
async fn reconcile_400_bad_request_refunds() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-400";
    if !require_memory_writes_enabled() {
        return;
    }
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    let outcome = reconcile(
        hold_id,
        CallOutcome::ProviderError {
            http_status: 400,
            retryable: false,
            provider_error_message: "bad request: invalid prompt".to_string(),
        },
        &pool,
    )
    .await;

    match outcome {
        Ok(ReconcileOutcome::Refunded {
            reason,
            hold_estimated_usd,
        }) => {
            assert_eq!(reason, RefundReason::ProviderError { http_status: 400 });
            assert_eq!(hold_estimated_usd, dec!(0.0085));
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(other) => panic!("expected Refunded, got {:?}", other),
    }

    // Ledger should be unchanged.
    let spent = read_ledger_spent(&pool, tenant).await;
    assert_eq!(spent, dec!(50));
    assert_eq!(count_memory_rows(hold_id, "ai.invocation_failed"), 1);

    cleanup_tenant(&pool, tenant).await;
}

// ─── AC #13: AlreadyFinalised carries persisted outcome ──────────────────────

#[tokio::test]
async fn reconcile_already_finalised_carries_outcome() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };
    init_cost_table_for_tests();

    let tenant = "test:reconcile-finalised";
    cleanup_tenant(&pool, tenant).await;
    seed_tenant(&pool, tenant, dec!(100), dec!(50)).await;

    // Seed a hold that's already reconciled.
    let hold_id = seed_hold(
        &pool,
        tenant,
        dec!(0.0085),
        "bedrock",
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .await;

    // Manually set to reconciled state.
    sqlx::query(
        "UPDATE cost_ledger_hold SET state = 'reconciled', actual_usd = 0.0078, \
         reconciled_at = NOW(), warn_crossed = TRUE WHERE id = $1",
    )
    .bind(hold_id)
    .execute(&pool)
    .await
    .unwrap();

    let outcome = reconcile(
        hold_id,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 120,
                completion_tokens: 450,
            },
            latency_ms: 850,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_abc123".to_string(),
        },
        &pool,
    )
    .await;

    match outcome {
        Err(ReconcileError::AlreadyFinalised {
            current_state,
            original_outcome,
        }) => {
            assert_eq!(current_state, "reconciled");
            match original_outcome {
                ReconcileOutcome::Reconciled {
                    actual_usd,
                    warn_crossed,
                    ..
                } => {
                    assert_eq!(actual_usd, dec!(0.0078));
                    assert!(warn_crossed);
                }
                other => panic!("expected Reconciled in original_outcome, got {:?}", other),
            }
        }
        other => panic!("expected AlreadyFinalised, got {:?}", other),
    }

    cleanup_tenant(&pool, tenant).await;
}

// ─── HoldNotFound ────────────────────────────────────────────────────────────

#[tokio::test]
async fn reconcile_hold_not_found() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set; skipping");
            return;
        }
    };

    let fake_id = Uuid::new_v4();
    let outcome = reconcile(
        fake_id,
        CallOutcome::Success {
            usage: ProviderUsage {
                prompt_tokens: 100,
                completion_tokens: 200,
            },
            latency_ms: 500,
            cache_state: CacheState::Miss,
            provider_request_id: "prv_xyz".to_string(),
        },
        &pool,
    )
    .await;

    match outcome {
        Err(ReconcileError::HoldNotFound(id)) => assert_eq!(id, fake_id),
        other => panic!("expected HoldNotFound, got {:?}", other),
    }
}
