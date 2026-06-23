//! FR-AI-018 §1 #7 — Adversarial tenant string tests.

mod support;
use support::proptest_strategies::adversarial_tenant_strings;
use support::redis_isolation_helper::{redis_available, RedisTestNamespace};
use support::test_provider_response;

use cyberos_ai_gateway::cache::{self, CacheKey, CacheLookupOutcome};

#[tokio::test]
async fn adversarial_tenant_strings_dont_leak() {
    // Needs a live Redis to look up against; skip cleanly where there is none (the no-Redis
    // lint job) so a connect timeout is not misread as a cross-tenant leak.
    if !redis_available() {
        eprintln!("skipping adversarial_tenant_strings_dont_leak: no Redis at REDIS_URL");
        return;
    }
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let benign = ns.tenant("benign_tenant");
    let benign_key = CacheKey::derive(&benign, "p", "chat.smart", "p@1.0.0");
    let _ = cache::insert(&benign_key, &test_provider_response(), "chat.fast").await;

    for adv in adversarial_tenant_strings() {
        let adv_namespaced = ns.tenant(adv);
        let adv_key = CacheKey::derive(&adv_namespaced, "p", "chat.smart", "p@1.0.0");
        match cache::lookup(&adv_key).await {
            CacheLookupOutcome::Miss => {}
            other => panic!("adversarial leak: adv={adv:?} outcome={other:?}"),
        }
    }
}

#[tokio::test]
async fn unit_separator_in_tenant_id_is_distinct() {
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let t1 = ns.tenant("tenant\x1fa");
    let t2 = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&t1, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&t2, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

#[tokio::test]
async fn very_long_tenant_id_distinct_from_short() {
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let short = ns.tenant("t");
    let long = ns.tenant(&"a".repeat(10_000));
    let k1 = CacheKey::derive(&short, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&long, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}
