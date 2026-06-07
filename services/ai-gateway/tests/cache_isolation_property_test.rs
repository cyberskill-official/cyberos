//! FR-AI-018 §1 #3–5 — Property tests for cross-tenant cache isolation.
//!
//! Requires a running Redis at 127.0.0.1:6379.
//! Run with: docker run -d --name test-redis -p 6379:6379 redis:7

mod support;
use support::proptest_strategies::*;
use support::redis_isolation_helper::RedisTestNamespace;
use support::test_provider_response;

use std::sync::{Mutex, MutexGuard, OnceLock};

use cyberos_ai_gateway::cache::{self, CacheInsertOutcome, CacheKey, CacheLookupOutcome};
use proptest::prelude::*;

static REDIS_PROPERTY_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn redis_property_test_lock() -> MutexGuard<'static, ()> {
    REDIS_PROPERTY_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn init_redis() {
    let _guard = redis_property_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn no_cross_tenant_cache_reads(
        (tenant_a, tenant_b) in any_tenant_pair(),
        ops in prop::collection::vec(any_cache_op(), 800..1200),
    ) {
        let _guard = redis_property_test_lock();
        cache::redis_backend::init(
            &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        );
        let ns = RedisTestNamespace::new();
        let t_a = ns.tenant(&tenant_a);
        let t_b = ns.tenant(&tenant_b);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Insert under tenant_a.
            for (prompt, model, persona) in &ops {
                let k = CacheKey::derive(&t_a, prompt, model, persona);
                let _ = cache::insert(&k, &test_provider_response(), model).await;
            }
            // Lookup under tenant_b — MUST all miss.
            for (prompt, model, persona) in &ops {
                let k = CacheKey::derive(&t_b, prompt, model, persona);
                match cache::lookup(&k).await {
                    CacheLookupOutcome::Miss | CacheLookupOutcome::Error(_) => {}
                    other => prop_assert!(
                        false,
                        "cross-tenant leak: t_a={} t_b={} prompt={:?} model={} persona={} \
                         k_a_hash={} k_b_hash={} outcome={:?}",
                        t_a, t_b, prompt, model, persona,
                        hex::encode(CacheKey::derive(&t_a, prompt, model, persona).prompt_hash),
                        hex::encode(k.prompt_hash), other,
                    ),
                }
            }
            Ok(())
        }).unwrap();
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_cross_tenant_key_collisions(
        (a, b) in any_tenant_pair(),
        prompt in any_prompt(),
        model in any_model(),
        persona in any_persona_handle(),
    ) {
        let k_a = CacheKey::derive(&a, &prompt, &model, &persona);
        let k_b = CacheKey::derive(&b, &prompt, &model, &persona);
        let h_a = hex::encode(k_a.prompt_hash);
        let h_b = hex::encode(k_b.prompt_hash);
        prop_assert_ne!(k_a.prompt_hash, k_b.prompt_hash,
            "cache-key collision: tenant_a={} tenant_b={} prompt={:?} k_a={} k_b={}",
            a, b, prompt, h_a, h_b);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn redis_keys_scan_is_tenant_isolated(
        (a, b) in any_tenant_pair(),
        n_ops in 10..100u32,
    ) {
        let _guard = redis_property_test_lock();
        cache::redis_backend::init(
            &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        );
        let ns = RedisTestNamespace::new();
        let t_a = ns.tenant(&a);
        let t_b = ns.tenant(&b);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut expected_tenant_a_keys = std::collections::BTreeSet::new();
            let mut inserted_tenant_a_keys = std::collections::BTreeSet::new();
            let mut expected_tenant_b_keys = std::collections::BTreeSet::new();
            for i in 0..n_ops {
                let k_a = CacheKey::derive(&t_a, &format!("p{i}"), "chat.smart", "p@1.0.0");
                let k_b = CacheKey::derive(&t_b, &format!("p{i}"), "chat.smart", "p@1.0.0");
                expected_tenant_a_keys.insert(k_a.redis_key());
                expected_tenant_b_keys.insert(k_b.redis_key());
                let inserted_a = cache::insert(&k_a, &test_provider_response(), "chat.fast").await;
                let inserted_b = cache::insert(&k_b, &test_provider_response(), "chat.fast").await;
                if matches!(inserted_a, CacheInsertOutcome::Inserted { .. }) {
                    inserted_tenant_a_keys.insert(k_a.redis_key());
                }
                let _ = inserted_b;
            }

            // Scan Redis for tenant_a's keys only.
            let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
            let client = redis::Client::open(redis_url.as_str()).unwrap();
            let mut conn = client.get_async_connection().await.unwrap();
            let pattern = format!("ai_cache:v1:{}:*", t_a);
            let mut cursor: u64 = 0;
            let mut all_keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            loop {
                let (next, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(&mut conn)
                    .await
                    .unwrap();
                all_keys.extend(keys);
                cursor = next;
                if cursor == 0 { break; }
            }

            for key in &all_keys {
                prop_assert!(key.starts_with(&format!("ai_cache:v1:{}:", t_a)),
                    "namespace leak: scan returned {key} when filtering for {t_a}");
                prop_assert!(expected_tenant_a_keys.contains(key),
                    "unexpected key in tenant_a namespace: {key}");
                prop_assert!(!expected_tenant_b_keys.contains(key),
                    "tenant_b key returned from tenant_a scan: {key}");
            }
            for key in &inserted_tenant_a_keys {
                prop_assert!(all_keys.contains(key),
                    "successfully inserted tenant_a key missing from scan: {key}");
            }
            Ok(())
        }).unwrap();
    }
}
