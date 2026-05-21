//! FR-AI-017 §3 — Cache key derivation with SHA-256 and per-tenant prefix.

use sha2::{Digest, Sha256};

/// Cryptographic cache key: per-tenant prefix + SHA-256 prompt hash.
///
/// Inputs joined with unit-separator (`\x1f`) to prevent collision attacks:
/// `tenant_id ␟ redacted_prompt ␟ model ␟ persona_handle`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub tenant_id: String,
    pub prompt_hash: [u8; 32],
}

impl CacheKey {
    /// §1 #1: cryptographic key derivation with unit-separator-joined inputs.
    pub fn derive(
        tenant_id: &str,
        redacted_prompt: &str,
        model: &str,
        persona_handle: &str,
    ) -> Self {
        let mut h = Sha256::new();
        h.update(tenant_id.as_bytes());
        h.update(b"\x1f");
        h.update(redacted_prompt.as_bytes());
        h.update(b"\x1f");
        h.update(model.as_bytes());
        h.update(b"\x1f");
        h.update(persona_handle.as_bytes());
        Self {
            tenant_id: tenant_id.into(),
            prompt_hash: h.finalize().into(),
        }
    }

    /// §1 #2 + §1 #13: per-tenant prefix + schema version.
    pub fn redis_key(&self) -> String {
        format!(
            "ai_cache:{}:{}:{}",
            super::CACHE_SCHEMA_VERSION,
            self.tenant_id,
            hex::encode(self.prompt_hash)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_deterministic() {
        let k1 = CacheKey::derive("t1", "hello", "chat.smart", "p@1.0");
        let k2 = CacheKey::derive("t1", "hello", "chat.smart", "p@1.0");
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_inputs_different_keys() {
        let k1 = CacheKey::derive("t1", "hello", "chat.smart", "p@1.0");
        let k2 = CacheKey::derive("t1", "world", "chat.smart", "p@1.0");
        assert_ne!(k1.prompt_hash, k2.prompt_hash);
    }

    #[test]
    fn different_persona_different_keys() {
        let k1 = CacheKey::derive("t1", "hello", "chat.smart", "p@1.0");
        let k2 = CacheKey::derive("t1", "hello", "chat.smart", "p@2.0");
        assert_ne!(k1.prompt_hash, k2.prompt_hash);
    }

    #[test]
    fn different_tenant_different_keys() {
        let k1 = CacheKey::derive("t1", "hello", "chat.smart", "p@1.0");
        let k2 = CacheKey::derive("t2", "hello", "chat.smart", "p@1.0");
        assert_ne!(k1, k2);
        // Same prompt_hash but different tenant_id → different redis_key.
        assert_ne!(k1.redis_key(), k2.redis_key());
    }

    #[test]
    fn redis_key_has_schema_prefix() {
        let k = CacheKey::derive("t1", "hello", "chat.smart", "p@1.0");
        assert!(k.redis_key().starts_with("ai_cache:v1:t1:"));
    }

    #[test]
    fn unit_separator_prevents_concat_collision() {
        // concat("a", "b") vs concat("ab", "") should differ.
        let k1 = CacheKey::derive("tenant", "model", "chat.smart", "p");
        let k2 = CacheKey::derive("tenantm", "odel", "chat.smart", "p");
        assert_ne!(k1.prompt_hash, k2.prompt_hash);
    }
}
