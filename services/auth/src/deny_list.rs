//! FR-AUTH-005 §1 #3 + #11 + G-011 — in-memory JWT jti deny-list.
//!
//! ### Architectural decision
//!
//! Per **DEC-DENY-LIST-001** (FR-AUTH-005 audit §10.5), slice-1 uses an
//! in-process `HashMap<jti, expires_at>` guarded by `RwLock`. The spec's
//! §1 #11 "30-second propagation via Redis pub/sub" requirement is
//! **trivially satisfied** for the single-instance deploy that is wave-1-2's
//! topology because the deny-list lives in the same process as the
//! `verify_jwt` middleware — there's no propagation step at all.
//!
//! The Redis-backed variant lifts to **FR-AUTH-110** once AUTH scales
//! horizontally (post-wave-2). The public surface here (`deny()`,
//! `is_denied()`, `gc()`) is intentionally trait-shaped so the Redis
//! implementation can drop in behind the same interface.
//!
//! ### Memory bounds
//!
//! Entries auto-expire at `exp` (the JWT's natural expiry); the GC sweeper
//! is called opportunistically on every `deny()` call to amortise cleanup.
//! In the steady state the map size is bounded by **active jti count ×
//! revocation rate × (max JWT lifetime)** which for a 1-hour token TTL and
//! a few revocations per hour stays well under 10k entries. A periodic
//! sweeper (future) can run in a background task if the deploy hits scale.
//!
//! ### Why no `Drop` on revoke
//!
//! Per FR-AUTH-005 §1 #12 + G-012, the deny-list **MUST NOT** clear on
//! unrevoke — the security default is "explicit re-auth required". So the
//! only removal path is natural expiry via GC. There is no `un_deny()`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot_compat::RwLock;

/// Internal alias to keep call-sites short.
type Inner = Arc<RwLock<HashMap<String, Instant>>>;

/// Process-wide JWT jti deny-list. `Clone` is cheap (Arc).
#[derive(Clone, Default)]
pub struct DenyList {
    inner: Inner,
}

impl DenyList {
    /// Fresh empty deny-list. Use one per `AppState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a jti to the deny-list with the given absolute expiry. After
    /// `expires_at` passes, GC will purge the entry; until then,
    /// `is_denied()` returns `true`.
    ///
    /// Opportunistically GCs on every insert to keep the map bounded.
    pub fn deny(&self, jti: &str, expires_at: Instant) {
        let mut g = self.inner.write();
        // Opportunistic GC — cheap (single pass) and amortised across denies.
        let now = Instant::now();
        g.retain(|_, exp| *exp > now);
        g.insert(jti.to_string(), expires_at);
    }

    /// Convenience: `deny` with `expires_at = now + ttl`.
    pub fn deny_for(&self, jti: &str, ttl: Duration) {
        self.deny(jti, Instant::now() + ttl);
    }

    /// Is this jti currently denied? Returns `true` iff the entry exists
    /// AND has not expired. Stale entries are reported as not-denied
    /// (a request right after expiry MUST succeed because the JWT itself
    /// is also at its `exp`).
    pub fn is_denied(&self, jti: &str) -> bool {
        let g = self.inner.read();
        match g.get(jti) {
            Some(exp) => Instant::now() < *exp,
            None => false,
        }
    }

    /// Explicit GC sweep — useful in tests + future background sweeper.
    pub fn gc(&self) {
        let now = Instant::now();
        self.inner.write().retain(|_, exp| *exp > now);
    }

    /// Current entry count (post-GC) — useful for OTel
    /// `auth_admin_deny_list_size{service}` gauge (FR-AUTH-005 §1 #15).
    pub fn len(&self) -> usize {
        self.gc();
        self.inner.read().len()
    }

    /// Empty after construction; also true after GC purges everything.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Local RwLock shim
//
// `tokio::sync::RwLock` is async and over-engineered for this use case (the
// critical section is microseconds — a sync RwLock is the right primitive).
// `std::sync::RwLock` works fine but its poison semantics complicate the
// API. We use a tiny shim that wraps `std::sync::RwLock` with
// `.into_inner()`-style poison recovery so the deny-list keeps working
// even if a thread panicked while holding the write guard.
// ─────────────────────────────────────────────────────────────────────────────
mod parking_lot_compat {
    use std::sync::{RwLock as StdRwLock, RwLockReadGuard, RwLockWriteGuard};

    pub struct RwLock<T>(StdRwLock<T>);

    impl<T> RwLock<T> {
        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            self.0.read().unwrap_or_else(|p| p.into_inner())
        }
        pub fn write(&self) -> RwLockWriteGuard<'_, T> {
            self.0.write().unwrap_or_else(|p| p.into_inner())
        }
    }

    impl<T: Default> Default for RwLock<T> {
        fn default() -> Self {
            Self(StdRwLock::new(T::default()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_deny_list_is_empty() {
        let d = DenyList::new();
        assert!(d.is_empty());
        assert!(!d.is_denied("any"));
    }

    #[test]
    fn deny_then_is_denied_round_trip() {
        let d = DenyList::new();
        d.deny_for("jti-abc", Duration::from_secs(60));
        assert!(d.is_denied("jti-abc"));
        assert!(!d.is_denied("jti-xyz"));
    }

    #[test]
    fn past_expiry_reports_not_denied() {
        let d = DenyList::new();
        // Insert with an expiry already in the past — gets GC'd on next deny,
        // and `is_denied` returns false because `Instant::now() < past` is false.
        let past = Instant::now() - Duration::from_secs(1);
        d.deny("jti-stale", past);
        assert!(!d.is_denied("jti-stale"));
    }

    #[test]
    fn gc_purges_expired_entries() {
        let d = DenyList::new();
        d.deny("jti-fresh", Instant::now() + Duration::from_secs(60));
        d.deny("jti-stale", Instant::now() - Duration::from_secs(1));
        // First deny call would have GC'd `jti-stale` — but be defensive.
        d.gc();
        assert!(d.is_denied("jti-fresh"));
        assert!(!d.is_denied("jti-stale"));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn deny_is_idempotent() {
        let d = DenyList::new();
        d.deny_for("jti-abc", Duration::from_secs(60));
        d.deny_for("jti-abc", Duration::from_secs(60));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn clone_shares_state() {
        let d = DenyList::new();
        let cloned = d.clone();
        d.deny_for("jti-shared", Duration::from_secs(60));
        // Both views see the entry → confirms Arc sharing.
        assert!(cloned.is_denied("jti-shared"));
    }
}
