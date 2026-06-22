//! FR-AI-009 §3 — Time-source abstraction for deterministic testing.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Clock abstraction so tests can advance time deterministically.
pub trait Clock: Send + Sync {
    fn nanos_now(&self) -> u64;
}

/// Production clock backed by `std::time::Instant`.
#[derive(Debug)]
pub struct SystemClock {
    epoch: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn nanos_now(&self) -> u64 {
        Instant::now().duration_since(self.epoch).as_nanos() as u64
    }
}

/// Blanket impl so `Arc<T>` can be used as a `Clock` (enables sharing in tests).
impl<T: Clock> Clock for std::sync::Arc<T> {
    fn nanos_now(&self) -> u64 {
        (**self).nanos_now()
    }
}

/// Test clock with explicit `advance()`.
#[derive(Debug)]
pub struct MockClock {
    inner: AtomicU64,
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockClock {
    pub fn new() -> Self {
        Self {
            inner: AtomicU64::new(0),
        }
    }

    pub fn advance(&self, by: std::time::Duration) {
        self.inner
            .fetch_add(by.as_nanos() as u64, Ordering::Relaxed);
    }
}

impl Clock for MockClock {
    fn nanos_now(&self) -> u64 {
        self.inner.load(Ordering::Relaxed)
    }
}
