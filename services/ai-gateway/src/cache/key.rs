//! TASK-AI-017 §3 — Cache key derivation with SHA-256 and per-tenant prefix.

use sha2::{Digest, Sha256};

/// Cryptographic cache key: per-tenant prefix + SHA-256 prompt hash.
///
/// Each field is length-prefixed (u64-LE byte length, then the bytes) before hashing, so the
/// hash is an injective encoding of `(tenant_id, redacted_prompt, model, persona_handle)`. A
/// single-separator join (the previous `\x1f` scheme) is forgeable: any field may itself
/// contain the separator byte, so `derive("a", "b\x1fc", m, p)` and `derive("a\x1fb", "c", m, p)`
/// hashed the same stream while belonging to different tenants - a cross-tenant collision
/// (TASK-AI-018 §1 #3-5). Length-prefix framing removes that: lengths are unambiguous, so no byte
/// inside any field can forge a collision.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub tenant_id: String,
    pub prompt_hash: [u8; 32],
}

impl CacheKey {
    /// §1 #1: cryptographic key derivation over length-prefixed, injectively-framed inputs.
    pub fn derive(
        tenant_id: &str,
        redacted_prompt: &str,
        model: &str,
        persona_handle: &str,
    ) -> Self {
        let mut h = Sha256::new();
        // Length-prefix every field (u64-LE len, then bytes). This framing is uniquely
        // decodable for a fixed field count, so distinct tuples never share a hash input -
        // regardless of which bytes (including \x1f) appear inside a field.
        for field in [tenant_id, redacted_prompt, model, persona_handle] {
            h.update((field.len() as u64).to_le_bytes());
            h.update(field.as_bytes());
        }
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
    fn length_prefix_prevents_concat_collision() {
        // Boundary ambiguity: ("tenant","model") vs ("tenantm","odel") must not collide.
        let k1 = CacheKey::derive("tenant", "model", "chat.smart", "p");
        let k2 = CacheKey::derive("tenantm", "odel", "chat.smart", "p");
        assert_ne!(k1.prompt_hash, k2.prompt_hash);
    }

    #[test]
    fn separator_injection_cannot_forge_cross_tenant_collision() {
        // TASK-AI-018 regression: the old single-`\x1f` join let a separator byte inside a field
        // move the boundary, so a different tenant produced the same hash. Length-prefix framing
        // must keep these distinct even though the naive joined streams were identical.
        let victim = CacheKey::derive("a", "b\u{1f}c", "chat.smart", "p");
        let attacker = CacheKey::derive("a\u{1f}b", "c", "chat.smart", "p");
        assert_ne!(
            victim.prompt_hash, attacker.prompt_hash,
            "cross-tenant cache-key collision via separator injection"
        );
    }
}
