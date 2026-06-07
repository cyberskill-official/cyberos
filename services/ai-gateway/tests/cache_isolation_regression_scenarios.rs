//! FR-AI-018 §1 #6 — 7 enumerated regression scenarios for cache isolation.

mod support;
use support::redis_isolation_helper::RedisTestNamespace;
use support::test_provider_response;

use std::sync::{Mutex, MutexGuard, OnceLock};

use cyberos_ai_gateway::cache::{self, CacheKey, CacheLookupOutcome};

static REGRESSION_CACHE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn regression_cache_test_lock() -> MutexGuard<'static, ()> {
    REGRESSION_CACHE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// REGRESSION-001 (incident 2026-04-12, pre-fix):
/// `tenant_a` and `tenanta` previously collided due to insufficient input separation
/// when the key derivation used naive concat without unit-separator.
#[tokio::test]
async fn regression_001_underscore_collision() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let k1 = CacheKey::derive(&ns.tenant("tenant_a"), "prompt", "chat.smart", "p@1.0.0");
    let k2 = CacheKey::derive(&ns.tenant("tenanta"), "prompt", "chat.smart", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);

    let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;
    assert!(matches!(cache::lookup(&k2).await, CacheLookupOutcome::Miss));
}

/// REGRESSION-002 (incident 2026-04-15):
/// Unicode normalisation form differences ("é" precomposed vs. e + combining acute)
/// produced different hashes; the cache treats them as distinct (FR-AI-005 enforces NFC).
#[tokio::test]
async fn regression_002_unicode_normalization() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let nfc = ns.tenant("tenant_\u{00E9}"); // precomposed é
    let nfd = ns.tenant("tenant_e\u{0301}"); // e + combining acute
    let k1 = CacheKey::derive(&nfc, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&nfd, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);

    let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;
    assert!(matches!(cache::lookup(&k2).await, CacheLookupOutcome::Miss));
}

/// REGRESSION-003: trailing whitespace must NOT collapse two distinct tenant ids.
#[tokio::test]
async fn regression_003_trailing_whitespace() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let a = ns.tenant("tenant_a");
    let b = ns.tenant("tenant_a ");
    let k1 = CacheKey::derive(&a, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&b, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

/// REGRESSION-004: tenant ids are case-sensitive; "Tenant_A" ≠ "tenant_a".
#[tokio::test]
async fn regression_004_case_folding() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let upper = ns.tenant("Tenant_A");
    let lower = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&upper, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&lower, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

/// REGRESSION-005: empty prompt vs. single-space prompt must NOT collide.
#[tokio::test]
async fn regression_005_empty_prompt() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let t = ns.tenant("tenant_a");
    let k_empty = CacheKey::derive(&t, "", "chat.smart", "p@1.0.0");
    let k_space = CacheKey::derive(&t, " ", "chat.smart", "p@1.0.0");
    assert_ne!(k_empty.prompt_hash, k_space.prompt_hash);
}

/// REGRESSION-006: persona-handle MUST be in the key (FR-AI-017 ISS-001 fix verification).
/// Pre-fix, persona changes didn't invalidate cache.
#[tokio::test]
async fn regression_006_persona_omitted() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let t = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&t, "p", "chat.smart", "cuo-cpo@0.4.1");
    let k2 = CacheKey::derive(&t, "p", "chat.smart", "cuo-cpo@0.4.2");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

/// REGRESSION-007: model substring "chat.smart" vs. "chat.smartx" must NOT collide.
#[tokio::test]
async fn regression_007_model_substring() {
    let _guard = regression_cache_test_lock();
    cache::redis_backend::init(
        &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
    );
    let ns = RedisTestNamespace::new();
    let t = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&t, "p", "chat.smart", "p@1.0.0");
    let k2 = CacheKey::derive(&t, "p", "chat.smartx", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}
