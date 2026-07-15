//! TASK-AI-005 §3 — Lock-free policy cache backed by `arc_swap::ArcSwap`.
//!
//! AC #9 requires that 1000 tokio tasks × 100 reads each complete in under 1 second on a
//! 4-core dev machine — i.e. effectively unbounded throughput on the read path. Use
//! `ArcSwap<HashMap<…>>` to achieve a lock-free read path; writes rebuild the map and swap.

use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;

use super::schema::TenantPolicy;

/// Lock-free read-mostly cache of `tenant_id → Arc<TenantPolicy>`.
#[derive(Debug)]
pub struct PolicyCache {
    inner: ArcSwap<HashMap<String, Arc<TenantPolicy>>>,
}

impl PolicyCache {
    /// Empty cache.
    pub fn new() -> Self {
        Self {
            inner: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    /// Sub-microsecond hit on the read path.
    pub fn get(&self, tenant_id: &str) -> Option<Arc<TenantPolicy>> {
        self.inner.load().get(tenant_id).cloned()
    }

    /// Insert or replace a policy. Writes rebuild the map; reads remain lock-free.
    pub fn insert(&self, tenant_id: String, policy: Arc<TenantPolicy>) {
        let mut new_map = (**self.inner.load()).clone();
        new_map.insert(tenant_id, policy);
        self.inner.store(Arc::new(new_map));
    }

    /// Remove a policy entry (called by the file-watcher on file deletion).
    pub fn remove(&self, tenant_id: &str) {
        let mut new_map = (**self.inner.load()).clone();
        if new_map.remove(tenant_id).is_some() {
            self.inner.store(Arc::new(new_map));
        }
    }

    /// Snapshot of tenant ids currently cached, sorted for deterministic emission.
    pub fn loaded_tenants_sorted(&self) -> Vec<String> {
        let snapshot = self.inner.load();
        let mut ids: Vec<String> = snapshot.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Count of cached policies.
    pub fn len(&self) -> usize {
        self.inner.load().len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.load().is_empty()
    }
}

impl Default for PolicyCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::schema::{AiPolicy, Provider, Residency};
    use rust_decimal_macros::dec;

    fn dummy_policy(id: &str) -> Arc<TenantPolicy> {
        Arc::new(TenantPolicy {
            tenant_id: id.to_string(),
            ai_policy: AiPolicy {
                monthly_cap_usd: dec!(150),
                warn_threshold: 0.8,
                hard_stop: true,
                primary_provider: Provider::Anthropic {
                    model_alias_map: Default::default(),
                },
                fallback_chain: vec![],
                call_timeout_seconds: 60,
                residency: Residency::Sg1,
                zdr_required: true,
                emergency_override: Default::default(),
                allowed_personas: None,
                alias_overrides: None,
                residency_requires_regional_provider: None,
                pii_redaction_extra: None,
                langsmith_export: false,
            },
        })
    }

    #[test]
    fn insert_get_roundtrip() {
        let cache = PolicyCache::new();
        cache.insert("org:a".into(), dummy_policy("org:a"));
        assert_eq!(cache.get("org:a").unwrap().tenant_id, "org:a");
        assert!(cache.get("org:b").is_none());
    }

    #[test]
    fn remove_clears_entry() {
        let cache = PolicyCache::new();
        cache.insert("org:a".into(), dummy_policy("org:a"));
        cache.remove("org:a");
        assert!(cache.get("org:a").is_none());
    }

    #[test]
    fn loaded_tenants_sorted_is_deterministic() {
        let cache = PolicyCache::new();
        cache.insert("org:c".into(), dummy_policy("org:c"));
        cache.insert("org:a".into(), dummy_policy("org:a"));
        cache.insert("org:b".into(), dummy_policy("org:b"));
        assert_eq!(
            cache.loaded_tenants_sorted(),
            vec!["org:a", "org:b", "org:c"]
        );
    }
}
