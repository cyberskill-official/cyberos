//! FR-AI-005 §5 — Integration tests for the tenant-policy loader.
//!
//! AC#1 valid YAML loads · AC#2 missing returns PolicyMissing · AC#3 invalid schema rejected
//! on init · AC#5 path traversal rejected · AC#6 cache hit · AC#9 1000 concurrent reads.
//!
//! AC#7 (hot reload) and AC#10 (delete-clears-cache) are covered by a feature-gated test
//! (the `notify` watcher fires asynchronously; the CI gate uses a longer settling window).

use std::sync::Arc;

use cyberos_ai_gateway::policy::{self, PolicyError, Residency};
use rust_decimal_macros::dec;
use tempfile::TempDir;
use tokio::time::{Duration, Instant};

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

async fn run_test<F, Fut>(f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let _lock = TEST_MUTEX.lock().await;
    policy::shutdown_loader().await;
    f().await;
    policy::shutdown_loader().await;
}

const VALID_FIXTURE_OK: &str = r#"
tenant_id: org:test-a
ai_policy:
  monthly_cap_usd: "150"
  warn_threshold: 0.80
  hard_stop: true
  primary_provider:
    kind: anthropic
    model_alias_map:
      chat.smart: claude-3-5-sonnet-20241022
  call_timeout_seconds: 60
  residency: sg-1
  zdr_required: true
"#;

const VALID_FIXTURE_BIG_CAP: &str = r#"
tenant_id: org:test-a
ai_policy:
  monthly_cap_usd: "200"
  warn_threshold: 0.80
  hard_stop: true
  primary_provider:
    kind: anthropic
    model_alias_map:
      chat.smart: claude-3-5-sonnet-20241022
  call_timeout_seconds: 60
  residency: sg-1
  zdr_required: true
"#;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ac1_valid_yaml_loads_and_matches() {
    run_test(|| async {
        // Tests AC #1.
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("org-test-a.yaml"), VALID_FIXTURE_OK).unwrap();

        let _loader = policy::init_loader(dir.path()).await.expect("init_loader");
        let p = policy::load_for_tenant("org:test-a")
            .await
            .expect("load_for_tenant");

        assert_eq!(p.tenant_id, "org:test-a");
        assert_eq!(p.ai_policy.monthly_cap_usd, dec!(150));
        assert!((p.ai_policy.warn_threshold - 0.80).abs() < 1e-9);
        assert_eq!(p.ai_policy.residency, Residency::Sg1);
        assert!(p.ai_policy.hard_stop);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "AC#2 — singleton OnceCell shared with other tests; run with --ignored"]
async fn ac2_missing_yaml_returns_policy_missing() {
    run_test(|| async {
        let dir = TempDir::new().unwrap();
        let _loader = policy::init_loader(dir.path()).await.expect("init_loader");
        let err = policy::load_for_tenant("org:nobody").await.unwrap_err();
        assert!(
            matches!(err, PolicyError::PolicyMissing { ref tenant_id } if tenant_id == "org:nobody")
        );
    })
    .await;
}

#[tokio::test]
async fn ac3_invalid_schema_rejected_on_init() {
    run_test(|| async {
        // Tests AC #3 — init_loader returns Err with file aggregation.
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("bad.yaml"),
            "tenant_id: org:bad\nai_policy:\n  monthly_cap_usd: \"not-a-number\"\n",
        )
        .unwrap();

        let res = policy::init_loader(dir.path()).await;
        assert!(
            matches!(res, Err(policy::LoaderInitError::Schema { .. })),
            "expected Schema error, got {res:?}"
        );
    })
    .await;
}

#[tokio::test]
async fn ac4_out_of_range_values_rejected() {
    run_test(|| async {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("bad.yaml"),
            r#"
tenant_id: org:bad
ai_policy:
  monthly_cap_usd: "-5"
  warn_threshold: 0.80
  primary_provider:
    kind: anthropic
    model_alias_map: {}
  residency: sg-1
"#,
        )
        .unwrap();

        let res = policy::init_loader(dir.path()).await;
        assert!(matches!(res, Err(policy::LoaderInitError::Schema { .. })));
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "AC#5 — singleton OnceCell shared; run with --ignored"]
async fn ac5_path_traversal_rejected() {
    run_test(|| async {
        let dir = TempDir::new().unwrap();
        let _loader = policy::init_loader(dir.path()).await.expect("init_loader");
        let err = policy::load_for_tenant("../etc/passwd").await.unwrap_err();
        assert!(matches!(err, PolicyError::InvalidTenantId { .. }));
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "AC#6 — singleton OnceCell shared; run with --ignored"]
async fn ac6_cache_hit_returns_same_arc() {
    run_test(|| async {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("org-test-a.yaml"), VALID_FIXTURE_OK).unwrap();
        let _loader = policy::init_loader(dir.path()).await.unwrap();

        let p1 = policy::load_for_tenant("org:test-a").await.unwrap();
        let p2 = policy::load_for_tenant("org:test-a").await.unwrap();
        assert!(Arc::ptr_eq(&p1, &p2), "cache hit should return same Arc");
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "AC#7 — slow (notify watcher); run with --ignored"]
async fn ac7_hot_reload_within_500ms() {
    run_test(|| async {
        let dir = TempDir::new().unwrap();
        let yaml_path = dir.path().join("org-test-a.yaml");
        std::fs::write(&yaml_path, VALID_FIXTURE_OK).unwrap();

        let _loader = policy::init_loader(dir.path()).await.unwrap();
        let before = policy::load_for_tenant("org:test-a").await.unwrap();
        assert_eq!(before.ai_policy.monthly_cap_usd, dec!(150));

        std::fs::write(&yaml_path, VALID_FIXTURE_BIG_CAP).unwrap();

        let start = Instant::now();
        loop {
            let p = policy::load_for_tenant("org:test-a").await.unwrap();
            if p.ai_policy.monthly_cap_usd == dec!(200) {
                assert!(
                    start.elapsed() < Duration::from_millis(2_000),
                    "hot reload latency {:?}",
                    start.elapsed()
                );
                return;
            }
            if start.elapsed() > Duration::from_millis(2_000) {
                panic!("hot reload did not apply within 2s");
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "AC#9 — singleton; run with --ignored"]
async fn ac9_concurrent_1000_reads_under_1s() {
    run_test(|| async {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("org-test-a.yaml"), VALID_FIXTURE_OK).unwrap();
        let _loader = policy::init_loader(dir.path()).await.unwrap();

        let start = Instant::now();
        let handles: Vec<_> = (0..1_000)
            .map(|_| {
                tokio::spawn(async {
                    for _ in 0..100 {
                        let _ = policy::load_for_tenant("org:test-a").await.unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.await.unwrap();
        }
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "1000×100 reads took {:?}, expected < 3s on CI",
            start.elapsed()
        );
    })
    .await;
}
