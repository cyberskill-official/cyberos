//! Fingerprint deduplication (TASK-OBS-007 §1 #12). Alerts with the same fingerprint arriving within the
//! 5-minute window collapse to a single CHAT post carrying a "fired N times" counter. The window is
//! evaluated against a caller-supplied monotonic millisecond clock, so the logic is deterministic and
//! testable; the axum shell passes a real monotonic time.

use std::collections::HashMap;
use std::sync::Mutex;

/// The dedup window: 5 minutes in milliseconds (§1 #12).
pub const DEDUP_WINDOW_MS: u64 = 5 * 60 * 1000;

/// What to do with an observed alert fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupOutcome {
    /// First sighting in a fresh window - post it.
    FirstInWindow,
    /// A repeat within the window - bump the existing post's counter to `count`.
    Repeat { count: u32 },
}

struct Entry {
    window_start_ms: u64,
    count: u32,
}

/// Tracks the active window per fingerprint.
#[derive(Default)]
pub struct Deduper {
    seen: Mutex<HashMap<String, Entry>>,
}

impl Deduper {
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe a fingerprint at `now_ms`. Returns whether this is the first in a fresh window (post it)
    /// or a repeat within the window (bump the counter on the existing post). A fingerprint last seen
    /// more than the window ago starts a fresh window.
    pub fn observe(&self, fingerprint: &str, now_ms: u64) -> DedupOutcome {
        let mut seen = self.seen.lock().unwrap();
        match seen.get_mut(fingerprint) {
            Some(entry) if now_ms.saturating_sub(entry.window_start_ms) < DEDUP_WINDOW_MS => {
                entry.count += 1;
                DedupOutcome::Repeat { count: entry.count }
            }
            _ => {
                seen.insert(
                    fingerprint.to_string(),
                    Entry {
                        window_start_ms: now_ms,
                        count: 1,
                    },
                );
                DedupOutcome::FirstInWindow
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sighting_posts() {
        let d = Deduper::new();
        assert_eq!(d.observe("fp", 0), DedupOutcome::FirstInWindow);
    }

    #[test]
    fn repeats_within_window_bump_the_counter() {
        let d = Deduper::new();
        assert_eq!(d.observe("fp", 0), DedupOutcome::FirstInWindow);
        assert_eq!(d.observe("fp", 60_000), DedupOutcome::Repeat { count: 2 });
        assert_eq!(d.observe("fp", 120_000), DedupOutcome::Repeat { count: 3 });
    }

    #[test]
    fn a_refire_after_the_window_starts_fresh() {
        let d = Deduper::new();
        assert_eq!(d.observe("fp", 0), DedupOutcome::FirstInWindow);
        // exactly at the window boundary the old window has elapsed -> fresh.
        assert_eq!(
            d.observe("fp", DEDUP_WINDOW_MS),
            DedupOutcome::FirstInWindow
        );
    }

    #[test]
    fn distinct_fingerprints_are_independent() {
        let d = Deduper::new();
        assert_eq!(d.observe("a", 0), DedupOutcome::FirstInWindow);
        assert_eq!(d.observe("b", 0), DedupOutcome::FirstInWindow);
    }
}
