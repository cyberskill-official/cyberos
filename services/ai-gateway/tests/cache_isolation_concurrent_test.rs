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
        let ns = ns.clone();
        joinset.spawn(async move {
            let owner = &tenants[task_id % 50];
            // Look up under a reader tenant that NO task ever inserts under. With the previous
            // `other = tenants[(task_id+1) % 50]`, the lookup target was itself an owner for
            // another task and concurrently cached the same shared prompt `p{op}`, so a Hit was
            // that tenant reading its OWN entry, not a cross-tenant leak — a false positive under
            // load. A reader tenant is never written, so a Hit here can only be a genuine leak.
            let reader = ns.tenant(&format!("reader{task_id}"));
            for op in 0..100 {
                let k_owner = CacheKey::derive(owner, &format!("p{op}"), "chat.smart", "p@1.0.0");
                let _ = cache::insert(&k_owner, &test_provider_response(), "chat.fast").await;
                // Same prompt+model+persona, a tenant nobody writes — must miss.
                let k_reader = CacheKey::derive(&reader, &format!("p{op}"), "chat.smart", "p@1.0.0");
                let outcome = cache::lookup(&k_reader).await;
                // Isolation is about cross-tenant READS: only a Hit on another tenant's key is a
                // leak. Miss is correct; a transient Redis Error/Timeout under load is not a read
                // and must not be misreported as a leak.
                assert!(
                    !matches!(outcome, CacheLookupOutcome::Hit(..)),
                    "concurrent leak: task={task_id} owner={owner} reader={reader} outcome={outcome:?}"
                );
            }
        });
    }
    while let Some(r) = joinset.join_next().await {
        r.unwrap();
    }
}
