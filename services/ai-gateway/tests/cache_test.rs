//! TASK-AI-017 §5 — Cache hit/miss/cross-tenant/skip tests.
//!
//! Requires a running Redis at 127.0.0.1:6379.
//! Run with: docker run -d --name test-redis -p 6379:6379 redis:7

use cyberos_ai_gateway::cache::{
    self, CacheInsertOutcome, CacheKey, CacheLookupOutcome, SkipReason, CACHE_SCHEMA_VERSION,
};
use cyberos_ai_gateway::router::types::{Choice, FinishReason, ProviderResponse, ProviderUsage};

fn k(tenant: &str, prompt: &str, persona: &str) -> CacheKey {
    CacheKey::derive(tenant, prompt, "chat.smart", persona)
}

fn test_provider_response() -> ProviderResponse {
    ProviderResponse {
        id: "test-resp-1".into(),
        usage: ProviderUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            cached_input_tokens: 0,
        },
        choices: vec![Choice {
            index: 0,
            content: "Hello, world!".into(),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
        }],
        finish_reason: FinishReason::Stop,
        latency_ms: 150,
        cache_state: cyberos_ai_gateway::router::types::CacheState::None,
        attempts: vec![],
    }
}

/// True when a Redis is reachable at the test URL; probed once and cached. The Redis-backed tests
/// below skip when it is false so the no-Redis `lint + test` job stays green, while the integration
/// job and the awh gate - which both provide Redis - run them for real. Mirrors how the cost tests
/// skip when `DATABASE_URL` is unset: an absent backend is "cannot run here", not a failure.
fn redis_available() -> bool {
    use std::sync::OnceLock;
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        redis::Client::open("redis://127.0.0.1:6379")
            .and_then(|c| c.get_connection())
            .is_ok()
    })
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn cache_hit_returns_response() {
    if !redis_available() {
        return;
    }
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let key = k("tenant_a", "What's the weather?", "cuo-cpo@0.4.1");
    let resp = test_provider_response();
    let _ = cache::insert(&key, &resp, "chat.fast").await;

    match cache::lookup(&key).await {
        CacheLookupOutcome::Hit(cr, _) => {
            assert_eq!(cr.choices[0].content, "Hello, world!");
            assert_eq!(cr.schema_version, CACHE_SCHEMA_VERSION);
        }
        e => panic!("expected Hit, got {e:?}"),
    }
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn cache_miss_returns_miss() {
    if !redis_available() {
        return;
    }
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let key = CacheKey::derive("nonexistent", "no-such-prompt", "chat.smart", "p@1.0");

    match cache::lookup(&key).await {
        CacheLookupOutcome::Miss => {}
        e => panic!("expected Miss, got {e:?}"),
    }
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn cross_tenant_miss() {
    if !redis_available() {
        return;
    }
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let k_a = k("tenant_x", "same prompt", "cuo-cpo@0.4.1");
    let k_b = k("tenant_y", "same prompt", "cuo-cpo@0.4.1");
    let _ = cache::insert(&k_a, &test_provider_response(), "chat.fast").await;

    match cache::lookup(&k_b).await {
        CacheLookupOutcome::Miss => {}
        e => panic!("expected Miss for tenant_y; got {e:?}"),
    }
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn persona_version_change_invalidates() {
    if !redis_available() {
        return;
    }
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let k1 = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let k2 = k("tenant_a", "prompt", "cuo-cpo@0.4.2");
    let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;

    match cache::lookup(&k2).await {
        CacheLookupOutcome::Miss => {}
        e => panic!("expected Miss for new persona version; got {e:?}"),
    }
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn chat_long_skipped() {
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let key = k("tenant_a", "long story prompt", "cuo-cpo@0.4.1");
    let outcome = cache::insert(
        &key,
        &test_provider_response(),
        "chat.long-resolved-bedrock",
    )
    .await;
    assert!(matches!(
        outcome,
        CacheInsertOutcome::Skipped(SkipReason::ChatLongOrUnknownAlias)
    ));
    // The Skipped assertion above needs no Redis; the Miss lookup below does. Gate just the
    // lookup so the no-Redis lint job still exercises the chat.long skip decision.
    if !redis_available() {
        return;
    }
    assert!(matches!(
        cache::lookup(&key).await,
        CacheLookupOutcome::Miss
    ));
}

#[tokio::test]
async fn unknown_alias_skipped() {
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let outcome = cache::insert(&key, &test_provider_response(), "novel.alias").await;
    assert!(matches!(
        outcome,
        CacheInsertOutcome::Skipped(SkipReason::ChatLongOrUnknownAlias)
    ));
}

#[tokio::test]
async fn oversize_response_skipped() {
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let mut resp = test_provider_response();
    resp.choices[0].content = "x".repeat(2_000_000);

    match cache::insert(&key, &resp, "chat.fast").await {
        CacheInsertOutcome::Skipped(SkipReason::Oversize { actual_bytes }) => {
            assert!(actual_bytes > cache::MAX_PAYLOAD_BYTES);
        }
        o => panic!("expected Oversize; got {o:?}"),
    }
}

#[tokio::test]
async fn failed_response_not_cached() {
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let mut resp = test_provider_response();
    resp.finish_reason = FinishReason::ContentFilter;

    let outcome = cache::insert(&key, &resp, "chat.fast").await;
    assert!(matches!(
        outcome,
        CacheInsertOutcome::Skipped(SkipReason::FailedResponse)
    ));
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn schema_mismatch_treated_as_miss() {
    if !redis_available() {
        return;
    }
    cache::redis_backend::init("redis://127.0.0.1:6379");
    // This test inserts a payload with a different schema version directly.
    // We'll use a key that has a "v0" schema_version in the payload.
    let key = CacheKey::derive("schema_test", "prompt", "chat.smart", "p@1.0");

    // Build a payload with wrong schema version.
    let bad = serde_json::json!({
        "schema_version": "v0",
        "usage": {"prompt_tokens": 0, "completion_tokens": 0, "cached_input_tokens": 0},
        "choices": [],
        "finish_reason": "stop",
        "cached_at": "2026-05-15T00:00:00Z",
        "provider_ms": 100
    });

    // Insert raw via Redis.
    use redis::AsyncCommands;

    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_async_connection().await.unwrap();
    let _: Result<(), _> = conn
        .set_ex(key.redis_key(), bad.to_string().as_bytes().to_vec(), 3600)
        .await;

    match cache::lookup(&key).await {
        CacheLookupOutcome::SchemaMismatch => {}
        o => panic!("expected SchemaMismatch; got {o:?}"),
    }
}

#[tokio::test]
#[ignore = "integration: needs a live Redis; run with cargo test -- --ignored"]
async fn redis_keys_are_tenant_isolated() {
    if !redis_available() {
        return;
    }
    cache::redis_backend::init("redis://127.0.0.1:6379");
    let tenant_a = "isolation_test_a";
    let tenant_b = "isolation_test_b";

    for i in 0..5 {
        let key = CacheKey::derive(tenant_a, &format!("p{i}"), "chat.smart", "p@1.0");
        let _ = cache::insert(&key, &test_provider_response(), "chat.fast").await;
    }
    for i in 0..5 {
        let key = CacheKey::derive(tenant_b, &format!("p{i}"), "chat.smart", "p@1.0");
        let _ = cache::insert(&key, &test_provider_response(), "chat.fast").await;
    }

    // Scan for tenant_a keys only.
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_async_connection().await.unwrap();
    let pattern = format!("ai_cache:v1:{tenant_a}:*");
    let mut cursor: u64 = 0;
    let mut all_keys: Vec<String> = Vec::new();
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
        if cursor == 0 {
            break;
        }
    }

    assert!(all_keys
        .iter()
        .all(|k| k.starts_with(&format!("ai_cache:v1:{tenant_a}:"))));
    assert_eq!(all_keys.len(), 5);
}
