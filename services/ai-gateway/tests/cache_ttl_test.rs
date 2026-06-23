//! FR-AI-017 §5 — TTL jitter tests.

use cyberos_ai_gateway::cache::ttl::{jittered_ttl, ttl_for_alias};
use rand::SeedableRng;
use std::time::Duration;

#[test]
fn jitter_within_10_percent() {
    let nominal = Duration::from_secs(3600);
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for _ in 0..1000 {
        let actual = jittered_ttl(nominal, &mut rng);
        let ratio = actual.as_secs_f64() / nominal.as_secs_f64();
        assert!(
            (0.9..=1.1).contains(&ratio),
            "TTL jitter outside ±10%: {ratio}"
        );
    }
}

#[test]
fn chat_fast_ttl_is_1h() {
    let ttl = ttl_for_alias("chat.fast").unwrap();
    assert_eq!(ttl, Duration::from_secs(3600));
}

#[test]
fn chat_smart_ttl_is_30m() {
    let ttl = ttl_for_alias("chat.smart").unwrap();
    assert_eq!(ttl, Duration::from_secs(1800));
}

#[test]
fn chat_long_no_cache() {
    assert!(ttl_for_alias("chat.long-resolved-bedrock").is_none());
}

#[test]
fn unknown_alias_no_cache() {
    assert!(ttl_for_alias("novel.alias-resolved-foo").is_none());
}

#[test]
fn resolved_alias_strips_suffix_for_ttl() {
    let ttl = ttl_for_alias("chat.smart-resolved-bedrock-claude-3");
    assert_eq!(ttl, Some(Duration::from_secs(1800)));
}
