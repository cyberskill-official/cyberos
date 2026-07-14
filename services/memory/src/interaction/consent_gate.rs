//! TASK-MEMORY-121 §1 #8, #12 (DEC-2702) — the consent gate HOOK.
//!
//! Capture for any *subject* is hard-gated on that subject's acknowledgment of the TASK-EVAL-001 monitoring
//! notice. This FR DEFINES the gate as a trait and ships a default-deny stub; TASK-MEMORY-122 wires the real
//! implementation that consults TASK-EVAL-001's acknowledgment ledger (`monitoring_notice` /
//! `subject_acknowledgment`, owned by `services/eval`). This crate deliberately does NOT depend on
//! `services/eval`: the gate is injected as a `&dyn ConsentGate`, so the wiring is a later, additive change
//! and memory stays independent of eval.
//!
//! The default is DENY (`DenyAll`): until the real gate is wired and a subject has acknowledged, no subject
//! interaction is captured. That is the safe posture the governance phase exists to guarantee — there is no
//! code path that captures a person before the notice is acknowledged.
//!
//! System actors (`subject_id = None`) are exempt and never reach the gate — `emit` short-circuits them —
//! because there is no person to notify (§1 #8).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// The capture consent gate. `is_capture_allowed` answers "may I capture interactions for this subject in
/// this tenant right now?". Implementations read the TASK-EVAL-001 acknowledgment ledger; this crate ships
/// only the default-deny stub. Async because the real impl performs a (cached) DB read.
///
/// Errors propagate as `sqlx::Error` so a gate-DB failure is distinguishable from a clean deny — `emit`
/// surfaces it as `EmitError` and the caller swallows it (best-effort capture, §1 #7); it MUST NOT be
/// silently treated as "allowed".
#[async_trait::async_trait]
pub trait ConsentGate: Send + Sync {
    async fn is_capture_allowed(
        &self,
        tenant_id: Uuid,
        subject_id: Uuid,
    ) -> Result<bool, sqlx::Error>;
}

/// Default-deny stub (DEC-2702): denies every subject. This is the gate in force until TASK-MEMORY-122 wires
/// the real TASK-EVAL-001-backed gate. Capturing nothing is the correct, safe behaviour before consent is
/// established — never the reverse.
#[derive(Clone, Copy, Debug, Default)]
pub struct DenyAll;

#[async_trait::async_trait]
impl ConsentGate for DenyAll {
    async fn is_capture_allowed(&self, _tenant: Uuid, _subject: Uuid) -> Result<bool, sqlx::Error> {
        Ok(false)
    }
}

/// Test/seed-only gate that allows every subject. NOT for production — it bypasses the TASK-EVAL-001 notice.
/// Lives here (behind no feature flag, but named so misuse is obvious) so the emit happy-path test and
/// future local-demo seeding have an allow-gate without reaching into eval.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAll;

#[async_trait::async_trait]
impl ConsentGate for AllowAll {
    async fn is_capture_allowed(&self, _tenant: Uuid, _subject: Uuid) -> Result<bool, sqlx::Error> {
        Ok(true)
    }
}

/// §1 #12 — a bounded-TTL in-process cache decorator over any inner `ConsentGate`. A signed-in person
/// generates a stream of interactions; without this, the gate would issue a ledger query per event. The
/// verdict for `(tenant, subject)` is cached for `ttl` (default 60 s), so a burst from one person issues
/// at most one ledger read per window. A revocation (or a fresh acknowledgment) takes effect within the
/// TTL — the window is the documented bound. TASK-MEMORY-122 wraps its real gate in this.
pub struct CachingGate<G: ConsentGate> {
    inner: G,
    ttl: Duration,
    cache: Mutex<HashMap<(Uuid, Uuid), (bool, Instant)>>,
}

impl<G: ConsentGate> CachingGate<G> {
    /// Wrap `inner` with the default 60 s TTL (§1 #12).
    pub fn new(inner: G) -> Self {
        Self::with_ttl(inner, Duration::from_secs(60))
    }

    /// Wrap `inner` with an explicit TTL (tests use a short TTL to exercise expiry).
    pub fn with_ttl(inner: G, ttl: Duration) -> Self {
        Self {
            inner,
            ttl,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn cached(&self, key: (Uuid, Uuid)) -> Option<bool> {
        let map = self.cache.lock().unwrap();
        map.get(&key).and_then(|(verdict, at)| {
            if at.elapsed() < self.ttl {
                Some(*verdict)
            } else {
                None
            }
        })
    }

    fn store(&self, key: (Uuid, Uuid), verdict: bool) {
        let mut map = self.cache.lock().unwrap();
        map.insert(key, (verdict, Instant::now()));
    }
}

#[async_trait::async_trait]
impl<G: ConsentGate> ConsentGate for CachingGate<G> {
    async fn is_capture_allowed(&self, tenant: Uuid, subject: Uuid) -> Result<bool, sqlx::Error> {
        let key = (tenant, subject);
        if let Some(v) = self.cached(key) {
            return Ok(v);
        }
        let v = self.inner.is_capture_allowed(tenant, subject).await?;
        self.store(key, v);
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn deny_all_denies_every_subject() {
        let g = DenyAll;
        assert!(!g
            .is_capture_allowed(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn allow_all_allows_every_subject() {
        let g = AllowAll;
        assert!(g
            .is_capture_allowed(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap());
    }

    /// A counting inner gate proves the cache collapses a burst into a single inner call within the TTL.
    struct Counting(Arc<AtomicUsize>);
    #[async_trait::async_trait]
    impl ConsentGate for Counting {
        async fn is_capture_allowed(&self, _t: Uuid, _s: Uuid) -> Result<bool, sqlx::Error> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(true)
        }
    }

    #[tokio::test]
    async fn cache_collapses_a_burst_to_one_inner_call() {
        let calls = Arc::new(AtomicUsize::new(0));
        let gate = CachingGate::new(Counting(calls.clone()));
        let (t, s) = (Uuid::new_v4(), Uuid::new_v4());
        for _ in 0..5 {
            assert!(gate.is_capture_allowed(t, s).await.unwrap());
        }
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "burst must hit the inner gate once"
        );
    }

    #[tokio::test]
    async fn cache_expires_after_ttl() {
        let calls = Arc::new(AtomicUsize::new(0));
        let gate = CachingGate::with_ttl(Counting(calls.clone()), Duration::from_millis(20));
        let (t, s) = (Uuid::new_v4(), Uuid::new_v4());
        assert!(gate.is_capture_allowed(t, s).await.unwrap());
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert!(gate.is_capture_allowed(t, s).await.unwrap());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "a query past the TTL must re-hit the inner gate"
        );
    }
}
