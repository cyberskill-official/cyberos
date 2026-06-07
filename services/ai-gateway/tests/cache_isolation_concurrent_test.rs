//! FR-AI-018 §1 #8 — Concurrent-load test for cache isolation.

mod support;
use support::redis_isolation_helper::RedisTestNamespace;
use support::test_provider_response;

use cyberos_ai_gateway::cache::{self, CacheKey, CacheLookupOutcome};

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn one_hundred_tasks_racing_no_cross_tenant_reads() {
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = std::sync::Arc::new(RedisTestNamespace::new());
    let tenants: Vec<String> = (0..50).map(|i| ns.tenant(&format!("t{i}"))).collect();
    let mut joinset = tokio::task::JoinSet::new();

    for task_id in 0..100 {
        let tenants = tenants.clone();
        joinset.spawn(async move {
            let owner = &tenants[task_id % 50];
            let other = &tenants[(task_id + 1) % 50];
            for op in 0..100 {
                let prompt = format!("{owner}:p{op}");
                let k_owner = CacheKey::derive(owner, &prompt, "chat.smart", "p@1.0.0");
                let _ = cache::insert(&k_owner, &test_provider_response(), "chat.fast").await;
                // Same owner-specific prompt+model+persona, different tenant — must miss.
                let k_other = CacheKey::derive(other, &prompt, "chat.smart", "p@1.0.0");
                let outcome = cache::lookup(&k_other).await;
                assert!(
                    matches!(
                        outcome,
                        CacheLookupOutcome::Miss | CacheLookupOutcome::Error(_)
                    ),
                    "concurrent leak: task={task_id} owner={owner} other={other} outcome={outcome:?}"
                );
            }
        });
    }
    while let Some(r) = joinset.join_next().await {
        r.unwrap();
    }
}
