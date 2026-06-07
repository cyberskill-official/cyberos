//! FR-AI-007 §5 — Integration tests for the cost-table loader.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use rust_decimal_macros::dec;

use cyberos_ai_gateway::cost_table::{self, init_cost_table, LoaderInitError};
use cyberos_ai_gateway::policy::ProviderKind;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/cost_table").join(name)
}

static COST_TABLE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn cost_table_test_lock() -> MutexGuard<'static, ()> {
    COST_TABLE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ─── AC #1: Happy lookup ──────────────────────────────────────────────────────

#[tokio::test]
async fn lookup_bedrock_claude_sonnet() {
    let _guard = cost_table_test_lock();
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let rate = cost_table::lookup(
        &ProviderKind::Bedrock,
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    )
    .unwrap();
    assert_eq!(rate.input_per_1k_usd, dec!(0.003));
    assert_eq!(rate.output_per_1k_usd, dec!(0.015));
    assert!(!rate.is_embedding);
}

// ─── AC #2: Miss returns None ─────────────────────────────────────────────────

#[tokio::test]
async fn miss_returns_none() {
    let _guard = cost_table_test_lock();
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    assert!(cost_table::lookup(&ProviderKind::Anthropic, "nonexistent-model").is_none());
}

// ─── AC #3: Embedding flag set ────────────────────────────────────────────────

#[tokio::test]
async fn embedding_flag_set_for_titan() {
    let _guard = cost_table_test_lock();
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let rate = cost_table::lookup(&ProviderKind::Bedrock, "amazon.titan-embed-text-v2:0").unwrap();
    assert!(rate.is_embedding);
    assert_eq!(rate.output_per_1k_usd, dec!(0.0));
}

// ─── AC #4: Self-hosted BGE rates are 0 ──────────────────────────────────────

#[tokio::test]
async fn bge_rates_are_zero() {
    let _guard = cost_table_test_lock();
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let rate = cost_table::lookup(&ProviderKind::Bge, "bge-m3").unwrap();
    assert_eq!(rate.input_per_1k_usd, dec!(0.0));
    assert_eq!(rate.output_per_1k_usd, dec!(0.0));
    assert!(rate.is_embedding);
}

// ─── AC #5: Aggregate failures on init ────────────────────────────────────────

#[tokio::test]
async fn aggregate_three_failures() {
    let _guard = cost_table_test_lock();
    let err = init_cost_table(&fixture("three_failures.yaml"))
        .await
        .unwrap_err();
    match err {
        LoaderInitError::Schema { failures } => {
            assert_eq!(failures.len(), 3, "expected 3 aggregated failures");
        }
        _ => panic!("expected Schema error"),
    }
}

// ─── AC #6: Negative rate rejected ────────────────────────────────────────────

#[tokio::test]
async fn negative_rate_rejected_at_init() {
    let _guard = cost_table_test_lock();
    let err = init_cost_table(&fixture("negative_rate.yaml"))
        .await
        .unwrap_err();
    match err {
        LoaderInitError::Schema { failures } => {
            assert!(
                failures
                    .iter()
                    .any(|f| f.errors.iter().any(|e| e.contains("non-negative"))),
                "expected 'non-negative' error message"
            );
        }
        _ => panic!("expected Schema error"),
    }
}

// ─── AC #12: loaded_at populated after init ───────────────────────────────────

#[tokio::test]
async fn loaded_at_populated_after_init() {
    let _guard = cost_table_test_lock();
    let before = chrono::Utc::now();
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let after = chrono::Utc::now();

    let loaded = cost_table::loaded_at().expect("loaded_at should be Some");
    assert!(
        loaded >= before && loaded <= after,
        "loaded_at should be within init window"
    );
}

// ─── AC #15: is_embedding consistency ─────────────────────────────────────────

#[tokio::test]
async fn is_embedding_with_nonzero_output_rejected() {
    let _guard = cost_table_test_lock();
    let err = init_cost_table(&fixture("embedding_inconsistency.yaml"))
        .await
        .unwrap_err();
    match err {
        LoaderInitError::Schema { failures } => {
            assert!(
                failures.iter().any(|f| f
                    .errors
                    .iter()
                    .any(|e| e.contains("is_embedding: true requires"))),
                "expected 'is_embedding: true requires' error"
            );
        }
        _ => panic!("expected Schema error"),
    }
}

// ─── AC #10: Hot-reload failure preserves cache ──────────────────────────────

#[tokio::test]
async fn hot_reload_invalid_preserves_cache() {
    let _guard = cost_table_test_lock();
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("cost_rates.yaml");
    std::fs::copy(fixture("valid_rates.yaml"), &path).unwrap();

    let _handle = init_cost_table(&path).await.unwrap();

    // Verify initial load works
    let rate_before = cost_table::lookup(
        &ProviderKind::Bedrock,
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    );
    assert!(rate_before.is_some());

    // Corrupt the YAML
    std::fs::write(&path, "not: valid: yaml: at: all").unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Cache should still serve the original valid rate
    let rate_after = cost_table::lookup(
        &ProviderKind::Bedrock,
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
    );
    assert!(
        rate_after.is_some(),
        "cache should be preserved after invalid YAML"
    );
}

// ─── AC #9: Hot reload picks up new model ─────────────────────────────────────

#[tokio::test]
async fn hot_reload_picks_up_new_model() {
    let _guard = cost_table_test_lock();
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("cost_rates.yaml");
    std::fs::copy(fixture("valid_rates.yaml"), &path).unwrap();

    let _handle = init_cost_table(&path).await.unwrap();

    // Add a new model to the Anthropic provider block.
    let yaml = std::fs::read_to_string(&path).unwrap();
    let yaml = yaml.replace(
        "\n  openai:\n",
        "\n    claude-99-future:\n      input_per_1k_usd:  0.001\n      output_per_1k_usd: 0.005\n\n  openai:\n",
    );
    std::fs::write(&path, yaml).unwrap();

    let start = Instant::now();
    loop {
        if let Some(rate) = cost_table::lookup(&ProviderKind::Anthropic, "claude-99-future") {
            assert_eq!(rate.input_per_1k_usd, dec!(0.001));
            assert!(start.elapsed() < std::time::Duration::from_millis(1000));
            return;
        }
        if start.elapsed() > std::time::Duration::from_millis(1000) {
            panic!("hot reload did not pick up new model within 1s");
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

// ─── AC #8: Concurrent reads zero contention ──────────────────────────────────

#[tokio::test]
async fn concurrent_1000_tasks_no_contention() {
    let _guard = cost_table_test_lock();
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let start = Instant::now();

    let handles: Vec<_> = (0..1000)
        .map(|_| {
            tokio::spawn(async {
                for _ in 0..1000 {
                    let _ = cost_table::lookup(
                        &ProviderKind::Bedrock,
                        "anthropic.claude-3-5-sonnet-20241022-v2:0",
                    );
                }
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }

    assert!(
        start.elapsed() < std::time::Duration::from_millis(500),
        "1000 tasks × 1000 lookups should complete in <500ms"
    );
}
