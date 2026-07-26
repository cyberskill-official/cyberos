//! Per-profile Wise public-key cache (DEC-841) + force-refresh on verify fail (DEC-848).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const PUBLIC_KEY_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PublicKeyError {
    #[error("public_key_fetch_failed")]
    FetchFailed,
}

/// Pluggable key source — static PEM in tests; HTTP fetch in production later.
pub trait PublicKeySource: Send + Sync + std::fmt::Debug {
    fn fetch(&self, profile_id: i64) -> Result<String, PublicKeyError>;
}

#[derive(Debug)]
struct CacheEntry {
    pem: String,
    fetched_at: Instant,
}

/// In-process PEM cache keyed by `profile_id`, 24h TTL.
#[derive(Debug)]
pub struct CachedPublicKeys<S: PublicKeySource> {
    source: S,
    ttl: Duration,
    cache: Mutex<HashMap<i64, CacheEntry>>,
}

impl<S: PublicKeySource> CachedPublicKeys<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            ttl: PUBLIC_KEY_TTL,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_ttl(source: S, ttl: Duration) -> Self {
        Self {
            source,
            ttl,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_fetch(&self, profile_id: i64) -> Result<String, PublicKeyError> {
        {
            let guard = self.cache.lock().expect("public key cache lock");
            if let Some(entry) = guard.get(&profile_id) {
                if entry.fetched_at.elapsed() < self.ttl {
                    return Ok(entry.pem.clone());
                }
            }
        }
        self.force_refresh(profile_id)
    }

    pub fn force_refresh(&self, profile_id: i64) -> Result<String, PublicKeyError> {
        let pem = self.source.fetch(profile_id)?;
        let mut guard = self.cache.lock().expect("public key cache lock");
        guard.insert(
            profile_id,
            CacheEntry {
                pem: pem.clone(),
                fetched_at: Instant::now(),
            },
        );
        Ok(pem)
    }

    /// Test/helper: how many times the source would be hit after cold start is elsewhere.
    pub fn cached_pem(&self, profile_id: i64) -> Option<String> {
        self.cache
            .lock()
            .ok()
            .and_then(|g| g.get(&profile_id).map(|e| e.pem.clone()))
    }
}

/// Fixed PEM for every profile (dev / tests).
#[derive(Debug)]
pub struct StaticPemSource {
    pem: String,
    fetch_count: Mutex<usize>,
}

impl StaticPemSource {
    pub fn new(pem: impl Into<String>) -> Self {
        Self {
            pem: pem.into(),
            fetch_count: Mutex::new(0),
        }
    }

    pub fn fetch_count(&self) -> usize {
        *self.fetch_count.lock().expect("fetch_count lock")
    }
}

impl PublicKeySource for StaticPemSource {
    fn fetch(&self, _profile_id: i64) -> Result<String, PublicKeyError> {
        *self.fetch_count.lock().expect("fetch_count lock") += 1;
        Ok(self.pem.clone())
    }
}

/// Source that returns `first` until `advance()` is called, then `second` (rotation tests).
#[derive(Debug)]
pub struct RotatingPemSource {
    first: String,
    second: String,
    advanced: Mutex<bool>,
    fetch_count: Mutex<usize>,
}

impl RotatingPemSource {
    pub fn new(first: impl Into<String>, second: impl Into<String>) -> Self {
        Self {
            first: first.into(),
            second: second.into(),
            advanced: Mutex::new(false),
            fetch_count: Mutex::new(0),
        }
    }

    pub fn advance(&self) {
        *self.advanced.lock().expect("advanced lock") = true;
    }

    pub fn fetch_count(&self) -> usize {
        *self.fetch_count.lock().expect("fetch_count lock")
    }
}

impl PublicKeySource for RotatingPemSource {
    fn fetch(&self, _profile_id: i64) -> Result<String, PublicKeyError> {
        *self.fetch_count.lock().expect("fetch_count lock") += 1;
        if *self.advanced.lock().expect("advanced lock") {
            Ok(self.second.clone())
        } else {
            Ok(self.first.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    struct CountingSource {
        pem: String,
        count: Arc<Mutex<usize>>,
    }

    impl PublicKeySource for CountingSource {
        fn fetch(&self, _profile_id: i64) -> Result<String, PublicKeyError> {
            *self.count.lock().expect("count") += 1;
            Ok(self.pem.clone())
        }
    }

    #[test]
    fn second_get_within_ttl_does_not_refetch() {
        let count = Arc::new(Mutex::new(0usize));
        let cache = CachedPublicKeys::with_ttl(
            CountingSource {
                pem: "pem-a".into(),
                count: Arc::clone(&count),
            },
            Duration::from_secs(60),
        );
        assert_eq!(cache.get_or_fetch(1).unwrap(), "pem-a");
        assert_eq!(cache.get_or_fetch(1).unwrap(), "pem-a");
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn ttl_expiry_refetches() {
        let count = Arc::new(Mutex::new(0usize));
        let cache = CachedPublicKeys::with_ttl(
            CountingSource {
                pem: "pem-b".into(),
                count: Arc::clone(&count),
            },
            Duration::from_millis(20),
        );
        cache.get_or_fetch(9).unwrap();
        thread::sleep(Duration::from_millis(30));
        cache.get_or_fetch(9).unwrap();
        assert_eq!(*count.lock().unwrap(), 2);
    }
}
