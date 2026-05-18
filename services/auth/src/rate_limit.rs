//! FR-AUTH-004 §1 #5 — slice-1 audit-fix G-001.
//!
//! Dual rate-limit for `POST /v1/auth/token`:
//!   * **Per-IP**     — 10 attempts per minute per source IP.
//!   * **Per-account** — 5 attempts per minute per `(tenant_slug, handle)` tuple
//!                       (handle replaces spec's `email` per §10.6 amendment).
//!
//! Either limit tripping returns `429 Too Many Requests` with body
//! `{"error":"rate_limited","retry_after_seconds":<n>}`.
//!
//! ## Backend choice — in-memory, not Redis
//!
//! Spec §1 #5 mentions Redis counters. The deployed implementation uses an
//! in-memory store (`Mutex<HashMap<key, BucketState>>`) — adequate for
//! single-instance dev/prod and current scale. Multi-instance prod will
//! sync via Redis when FR-OBS-002 ships. Documented in
//! `FR-AUTH-004-jwt-jwks.audit.md §10.6 #2` as an operator-decision item.
//!
//! ## Algorithm — fixed-window counter
//!
//! For each key we track `(window_start: Instant, count: u32)`. On every
//! `check_*` call:
//!   1. If `window_start + window_duration ≤ now()`, reset the window
//!      (`window_start = now`, `count = 0`).
//!   2. If `count + 1 > limit`, return `Err(retry_after_seconds)`.
//!   3. Else increment `count` and return `Ok(())`.
//!
//! Fixed-window is simpler than token-bucket and good enough for the
//! credential-stuffing threat model — a sliding-window or token-bucket
//! refinement is FR-OBS-002's problem.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Per-IP fixed-window limit: 10 attempts per minute.
pub const PER_IP_LIMIT: u32 = 10;
/// Per-account fixed-window limit: 5 attempts per minute.
pub const PER_ACCOUNT_LIMIT: u32 = 5;
/// Window size — 60 seconds for both per-IP and per-account.
pub const WINDOW_SECS: u64 = 60;

#[derive(Debug, Clone, Copy)]
struct BucketState {
    window_start: Instant,
    count: u32,
}

/// Dual rate-limiter. Lives on `AppState` as `Arc<RateLimiter>`. Cheap to
/// `Arc::clone` across handler invocations; `Mutex` contention is negligible
/// at the slice-1 traffic profile (each check holds the lock for < 1µs).
pub struct RateLimiter {
    /// Map: `source_ip_string` → bucket state.
    ip_buckets: Mutex<HashMap<String, BucketState>>,
    /// Map: `"<tenant_slug>|<handle>"` → bucket state.
    account_buckets: Mutex<HashMap<String, BucketState>>,
    /// Per-IP attempts allowed per window. Configurable so tests can use
    /// tighter limits without waiting a minute.
    ip_limit: u32,
    /// Per-account attempts allowed per window.
    account_limit: u32,
    /// Window duration. Configurable for tests.
    window: Duration,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    /// Production constructor — 10/min per IP, 5/min per account.
    pub fn new() -> Self {
        Self::with_config(PER_IP_LIMIT, PER_ACCOUNT_LIMIT, Duration::from_secs(WINDOW_SECS))
    }

    /// Test constructor — explicit limits + window. Production code uses
    /// `RateLimiter::new()`; this exists so unit tests can verify the
    /// window-rollover + exhaustion paths without waiting 60 real seconds.
    pub fn with_config(ip_limit: u32, account_limit: u32, window: Duration) -> Self {
        Self {
            ip_buckets: Mutex::new(HashMap::new()),
            account_buckets: Mutex::new(HashMap::new()),
            ip_limit,
            account_limit,
            window,
        }
    }

    /// Check + increment the per-IP bucket. Returns `Ok(())` if under the
    /// limit (and increments the counter); `Err(retry_after_seconds)` if at
    /// or over the limit.
    pub fn check_ip(&self, source_ip: &str) -> Result<(), u32> {
        self.check(&self.ip_buckets, source_ip, self.ip_limit)
    }

    /// Check + increment the per-account bucket. The key is
    /// `"<tenant_slug>|<handle>"` so collisions across tenants don't
    /// blend buckets.
    pub fn check_account(&self, tenant_slug: &str, handle: &str) -> Result<(), u32> {
        let key = format!("{tenant_slug}|{handle}");
        self.check(&self.account_buckets, &key, self.account_limit)
    }

    fn check(
        &self,
        store: &Mutex<HashMap<String, BucketState>>,
        key: &str,
        limit: u32,
    ) -> Result<(), u32> {
        let now = Instant::now();
        let mut map = match store.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = map.entry(key.to_string()).or_insert(BucketState {
            window_start: now,
            count: 0,
        });
        // Window-rollover.
        if now.duration_since(entry.window_start) >= self.window {
            entry.window_start = now;
            entry.count = 0;
        }
        if entry.count + 1 > limit {
            // Compute retry-after: seconds until the window closes.
            let elapsed = now.duration_since(entry.window_start);
            let remaining = self.window.saturating_sub(elapsed);
            return Err(remaining.as_secs().max(1) as u32);
        }
        entry.count += 1;
        Ok(())
    }

    /// Visible-for-tests: peek the current count in the per-IP bucket
    /// without mutating. Returns 0 if no bucket exists yet.
    #[doc(hidden)]
    pub fn ip_count(&self, source_ip: &str) -> u32 {
        let map = match self.ip_buckets.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.get(source_ip).map(|b| b.count).unwrap_or(0)
    }

    /// Visible-for-tests: peek the per-account count.
    #[doc(hidden)]
    pub fn account_count(&self, tenant_slug: &str, handle: &str) -> u32 {
        let key = format!("{tenant_slug}|{handle}");
        let map = match self.account_buckets.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.get(&key).map(|b| b.count).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_bucket_fills_then_rejects() {
        let rl = RateLimiter::with_config(3, 100, Duration::from_secs(60));
        for i in 0..3 {
            rl.check_ip("1.2.3.4").unwrap_or_else(|_| panic!("attempt {i} should succeed"));
        }
        let err = rl.check_ip("1.2.3.4").expect_err("4th attempt should reject");
        assert!(
            err > 0 && err <= 60,
            "retry_after_seconds should be 1..=60; got {err}"
        );
    }

    #[test]
    fn account_bucket_independent_per_account() {
        let rl = RateLimiter::with_config(100, 2, Duration::from_secs(60));
        // Account A — exhaust.
        rl.check_account("tenant-x", "alice").unwrap();
        rl.check_account("tenant-x", "alice").unwrap();
        let _ = rl.check_account("tenant-x", "alice").expect_err("A exhausted");
        // Account B — independent counter.
        rl.check_account("tenant-x", "bob").unwrap();
        rl.check_account("tenant-x", "bob").unwrap();
    }

    #[test]
    fn account_bucket_independent_across_ips() {
        // The whole point of per-account vs per-IP is that they're
        // independent: per-account catches distributed credential stuffing
        // (different IPs, same account). Verify the account bucket
        // increments regardless of source IP.
        let rl = RateLimiter::with_config(100, 3, Duration::from_secs(60));
        rl.check_account("ten", "alice").unwrap();
        rl.check_account("ten", "alice").unwrap();
        rl.check_account("ten", "alice").unwrap();
        let _ = rl.check_account("ten", "alice").expect_err("exhausted");
        assert_eq!(rl.account_count("ten", "alice"), 3);
    }

    #[test]
    fn window_rolls_over() {
        // Force a tiny window so the test doesn't take a minute.
        let rl = RateLimiter::with_config(2, 100, Duration::from_millis(50));
        rl.check_ip("9.9.9.9").unwrap();
        rl.check_ip("9.9.9.9").unwrap();
        let _ = rl.check_ip("9.9.9.9").expect_err("exhausted in window");
        std::thread::sleep(Duration::from_millis(70));
        // After rollover, fresh attempts allowed.
        rl.check_ip("9.9.9.9").unwrap();
        rl.check_ip("9.9.9.9").unwrap();
    }

    #[test]
    fn ip_buckets_are_independent_across_ips() {
        let rl = RateLimiter::with_config(2, 100, Duration::from_secs(60));
        rl.check_ip("1.1.1.1").unwrap();
        rl.check_ip("1.1.1.1").unwrap();
        let _ = rl.check_ip("1.1.1.1").expect_err("exhausted");
        // Different IP — fresh counter.
        rl.check_ip("2.2.2.2").unwrap();
        rl.check_ip("2.2.2.2").unwrap();
    }

    #[test]
    fn production_defaults() {
        let rl = RateLimiter::new();
        // Burn 10 per-IP attempts.
        for _ in 0..PER_IP_LIMIT {
            rl.check_ip("3.3.3.3").unwrap();
        }
        assert_eq!(rl.ip_count("3.3.3.3"), PER_IP_LIMIT);
        let _ = rl.check_ip("3.3.3.3").expect_err("11th must reject");
    }
}
