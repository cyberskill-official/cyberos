//! TASK-AI-017 §3 — TTL table per alias-class with jitter.

use std::time::Duration;

/// §1 #4: TTL per alias-class. Single source of truth.
/// New aliases added to TASK-AI-006 MUST extend this map; missing-alias → no-cache + WARN.
pub fn ttl_for_alias(alias: &str) -> Option<Duration> {
    match alias_class(alias) {
        "chat.fast" => Some(Duration::from_secs(3600)),
        "chat.smart" => Some(Duration::from_secs(1800)),
        "chat.long" => None, // no cache
        "embed.standard" => Some(Duration::from_secs(86400)),
        "embed.code" => Some(Duration::from_secs(86400)),
        "rerank.fast" => Some(Duration::from_secs(900)),
        _ => {
            tracing::warn!(
                alias = %alias,
                "ttl_for_alias: unknown alias class; treating as no-cache"
            );
            None
        }
    }
}

fn alias_class(alias: &str) -> &str {
    // alias is like "chat.smart" or "chat.smart-resolved-bedrock-claude-..."
    alias.split('-').next().unwrap_or(alias)
}

/// §1 #5: ±10% jitter to prevent thundering-herd expiry.
pub fn jittered_ttl(nominal: Duration, rng: &mut impl rand::Rng) -> Duration {
    let factor = 1.0 + rng.gen_range(-0.1..0.1);
    Duration::from_secs_f64(nominal.as_secs_f64() * factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn chat_fast_ttl() {
        assert_eq!(ttl_for_alias("chat.fast"), Some(Duration::from_secs(3600)));
    }

    #[test]
    fn chat_smart_ttl() {
        assert_eq!(ttl_for_alias("chat.smart"), Some(Duration::from_secs(1800)));
    }

    #[test]
    fn chat_long_no_cache() {
        assert_eq!(ttl_for_alias("chat.long-resolved-bedrock"), None);
    }

    #[test]
    fn embed_ttl() {
        assert_eq!(
            ttl_for_alias("embed.standard"),
            Some(Duration::from_secs(86400))
        );
    }

    #[test]
    fn rerank_ttl() {
        assert_eq!(ttl_for_alias("rerank.fast"), Some(Duration::from_secs(900)));
    }

    #[test]
    fn unknown_alias_returns_none() {
        assert_eq!(ttl_for_alias("novel.alias-resolved-foo"), None);
    }

    #[test]
    fn resolved_alias_strips_suffix() {
        assert_eq!(
            ttl_for_alias("chat.smart-resolved-bedrock-claude-3"),
            Some(Duration::from_secs(1800))
        );
    }

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
}
