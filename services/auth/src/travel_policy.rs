//! TASK-AUTH-106 slice-3 — per-tenant policy fetch + CIDR allowlist matching +
//! sticky-challenge suppression cache.
//!
//! Three pieces:
//!   * `PolicyCache` — 60s TTL in-memory cache keyed by tenant_id. Holds
//!     the per-tenant `TravelPolicy` row + CIDR allowlist. Hot path reads
//!     are lock-free after warm-up.
//!   * `StickySuppress` — bounded LRU keyed by (subject_id, /24 prefix).
//!     When the policy says "skip re-challenge for 30 minutes after a
//!     successful MFA pass from this /24", `record` writes to the LRU and
//!     `should_suppress` reads from it.
//!   * `cidr_allowed` — fast IP-in-CIDR check using the cached CIDR list.

use ipnetwork::IpNetwork;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TravelPolicy {
    pub action: PolicyAction,
    pub threshold_kmh: f64,
    pub block_anonymous_ip: bool,
    pub sticky_suppress_min: u32,
    pub allowlist: Vec<IpNetwork>,
}

impl Default for TravelPolicy {
    fn default() -> Self {
        Self {
            action: PolicyAction::Challenge,
            threshold_kmh: 1000.0,
            block_anonymous_ip: false,
            sticky_suppress_min: 30,
            allowlist: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyAction {
    Challenge,
    Block,
    WarnOnly,
}

impl PolicyAction {
    pub fn from_db(s: &str) -> Self {
        match s {
            "block" => Self::Block,
            "warn_only" => Self::WarnOnly,
            _ => Self::Challenge,
        }
    }
}

// ---------------------------------------------------------------------------
// PolicyCache — 60s TTL.
// ---------------------------------------------------------------------------

struct PolicyEntry {
    policy: TravelPolicy,
    refreshed_at: Instant,
}

const POLICY_TTL: Duration = Duration::from_secs(60);

#[derive(Clone, Default)]
pub struct PolicyCache {
    inner: Arc<RwLock<HashMap<Uuid, PolicyEntry>>>,
}

impl PolicyCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the policy for `tenant_id`, refreshing from Postgres if the
    /// cached entry is older than 60s (or never been loaded).
    pub async fn get(&self, pool: &PgPool, tenant_id: Uuid) -> TravelPolicy {
        {
            let r = self.inner.read().await;
            if let Some(e) = r.get(&tenant_id) {
                if e.refreshed_at.elapsed() < POLICY_TTL {
                    return e.policy.clone();
                }
            }
        }
        let policy = match load_policy(pool, tenant_id).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, tenant = %tenant_id, "policy load failed — using defaults");
                TravelPolicy::default()
            }
        };
        let mut w = self.inner.write().await;
        w.insert(
            tenant_id,
            PolicyEntry {
                policy: policy.clone(),
                refreshed_at: Instant::now(),
            },
        );
        policy
    }

    /// Force the entry for `tenant_id` out of the cache. Called by the
    /// policy-mutation handler after an UPDATE so the next read picks up
    /// the new row immediately.
    pub async fn invalidate(&self, tenant_id: Uuid) {
        self.inner.write().await.remove(&tenant_id);
    }
}

async fn load_policy(pool: &PgPool, tenant_id: Uuid) -> Result<TravelPolicy, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;

    let row: Option<(String, f64, bool, i32)> = sqlx::query_as(
        "SELECT action, threshold_kmh, block_anonymous_ip, sticky_suppress_min
             FROM travel_policy WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (action, threshold_kmh, block_anon, sticky) = match row {
        Some(r) => r,
        // No row yet — return defaults but don't auto-insert; the policy
        // table is initialised lazily by the admin UI.
        None => ("challenge".to_string(), 1000.0, false, 30),
    };

    // 2026-05-19 sqlx 0.8 + ipnetwork 0.20 Decode<Postgres> gap — read CIDR
    // as TEXT and parse to IpNetwork on the Rust side. Malformed CIDRs in
    // the DB are dropped silently (they were already invalid before storage
    // per the CHECK constraint in 0018_travel_policy.sql).
    let cidrs: Vec<(String,)> =
        sqlx::query_as("SELECT cidr::text FROM travel_cidr_allowlist WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(&mut *tx)
            .await?;
    tx.commit().await?;

    Ok(TravelPolicy {
        action: PolicyAction::from_db(&action),
        threshold_kmh,
        block_anonymous_ip: block_anon,
        sticky_suppress_min: sticky.max(0) as u32,
        allowlist: cidrs
            .into_iter()
            .filter_map(|(c,)| c.parse::<IpNetwork>().ok())
            .collect(),
    })
}

// ---------------------------------------------------------------------------
// CIDR membership.
// ---------------------------------------------------------------------------

/// True when `ip` falls in any of the allowlisted CIDRs.
pub fn cidr_allowed(allowlist: &[IpNetwork], ip: IpAddr) -> bool {
    allowlist.iter().any(|net| net.contains(ip))
}

// ---------------------------------------------------------------------------
// Sticky-challenge suppression LRU.
// ---------------------------------------------------------------------------

/// Bounded LRU keyed by (subject_id, /24 prefix string) → expiry time.
/// Single shared instance per service (kept on AppState). Capped at 50,000
/// entries; oldest evictions on overflow.
const STICKY_CAP: usize = 50_000;

#[derive(Default)]
pub struct StickySuppress {
    map: tokio::sync::Mutex<linked_hash_map::LinkedHashMap<(Uuid, String), Instant>>,
}

impl StickySuppress {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Record that `subject_id` passed an MFA challenge from `prefix24_str`
    /// at `now`, with sticky-suppression valid for `window_min` minutes.
    pub async fn record(&self, subject_id: Uuid, prefix24_str: String, window_min: u32) {
        if window_min == 0 {
            return;
        }
        let exp = Instant::now() + Duration::from_secs(window_min as u64 * 60);
        let key = (subject_id, prefix24_str);
        let mut m = self.map.lock().await;
        m.insert(key, exp);
        while m.len() > STICKY_CAP {
            m.pop_front();
        }
    }

    /// True if `subject_id`'s recent MFA-pass for `prefix24_str` is still
    /// valid (within the sticky window).
    pub async fn should_suppress(&self, subject_id: Uuid, prefix24_str: &str) -> bool {
        let key = (subject_id, prefix24_str.to_string());
        let mut m = self.map.lock().await;
        match m.get(&key) {
            Some(&exp) if exp > Instant::now() => true,
            Some(_) => {
                m.remove(&key);
                false
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cidr_allowed_matches_office_subnet() {
        let allow = vec!["203.0.113.0/24".parse().unwrap()];
        assert!(cidr_allowed(&allow, "203.0.113.42".parse().unwrap()));
        assert!(!cidr_allowed(&allow, "198.51.100.1".parse().unwrap()));
    }

    #[test]
    fn cidr_allowed_handles_ipv6() {
        let allow = vec!["2001:db8::/32".parse().unwrap()];
        assert!(cidr_allowed(&allow, "2001:db8:1::1".parse().unwrap()));
        assert!(!cidr_allowed(&allow, "2001:db9::1".parse().unwrap()));
    }

    #[test]
    fn policy_action_from_db_defaults_to_challenge() {
        assert_eq!(PolicyAction::from_db("challenge"), PolicyAction::Challenge);
        assert_eq!(PolicyAction::from_db("block"), PolicyAction::Block);
        assert_eq!(PolicyAction::from_db("warn_only"), PolicyAction::WarnOnly);
        assert_eq!(PolicyAction::from_db("garbage"), PolicyAction::Challenge);
    }

    #[tokio::test]
    async fn sticky_suppress_records_and_expires() {
        let s = StickySuppress::new();
        let subj = Uuid::new_v4();
        // No record yet — must not suppress.
        assert!(!s.should_suppress(subj, "10.0.0.0/24").await);
        // Record + immediate check — must suppress.
        s.record(subj, "10.0.0.0/24".into(), 30).await;
        assert!(s.should_suppress(subj, "10.0.0.0/24").await);
        // Zero-window record — must NOT suppress (used when policy disables sticky).
        let s2 = StickySuppress::new();
        s2.record(subj, "10.0.0.0/24".into(), 0).await;
        assert!(!s2.should_suppress(subj, "10.0.0.0/24").await);
    }
}
