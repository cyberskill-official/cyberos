//! FR-AUTH-102 — MFA lockout state machine.
//!
//! Policy: 5 failed MFA attempts within a 15-minute rolling window locks
//! the account for 30 minutes. A root-admin can unlock early via the admin
//! endpoint. Successful verification resets the counter.

use chrono::{DateTime, Duration, Utc};

pub const MAX_FAILURES: i32 = 5;
pub const WINDOW_SECS: i64 = 900; // 15 minutes
pub const LOCKOUT_SECS: i64 = 1800; // 30 minutes

#[derive(Debug, Clone)]
pub struct LockoutState {
    pub failed_count: i32,
    pub window_started_at: DateTime<Utc>,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

impl Default for LockoutState {
    fn default() -> Self {
        Self {
            failed_count: 0,
            window_started_at: Utc::now(),
            locked_until: None,
            last_attempt_at: None,
        }
    }
}

impl LockoutState {
    /// Check if the account is currently locked out.
    pub fn is_locked(&self) -> bool {
        match self.locked_until {
            Some(until) => Utc::now() < until,
            None => false,
        }
    }

    /// Record a failed MFA attempt. Returns `true` if the account just
    /// became locked.
    pub fn record_failure(&mut self) -> bool {
        let now = Utc::now();
        self.last_attempt_at = Some(now);

        // If the current window has elapsed, start a new one.
        if now > self.window_started_at + Duration::seconds(WINDOW_SECS) {
            self.window_started_at = now;
            self.failed_count = 1;
            return false;
        }

        self.failed_count += 1;

        if self.failed_count >= MAX_FAILURES {
            self.locked_until = Some(now + Duration::seconds(LOCKOUT_SECS));
            return true;
        }
        false
    }

    /// Reset the lockout state after a successful verification.
    pub fn reset(&mut self) {
        self.failed_count = 0;
        self.window_started_at = Utc::now();
        self.locked_until = None;
        self.last_attempt_at = None;
    }

    /// Admin unlock — clears the lock immediately.
    pub fn admin_unlock(&mut self) {
        self.locked_until = None;
        self.failed_count = 0;
        self.window_started_at = Utc::now();
    }

    /// Remaining lockout duration, if locked.
    pub fn remaining_lockout(&self) -> Option<Duration> {
        self.locked_until
            .filter(|&until| Utc::now() < until)
            .map(|until| until - Utc::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_locked() {
        let s = LockoutState::default();
        assert!(!s.is_locked());
        assert_eq!(s.failed_count, 0);
    }

    #[test]
    fn five_failures_triggers_lockout() {
        let mut s = LockoutState::default();
        for i in 0..4 {
            let locked = s.record_failure();
            assert!(!locked, "should not lock on attempt {}", i + 1);
        }
        let locked = s.record_failure();
        assert!(locked, "5th failure should trigger lockout");
        assert!(s.is_locked());
    }

    #[test]
    fn reset_clears_lockout() {
        let mut s = LockoutState::default();
        for _ in 0..5 {
            s.record_failure();
        }
        assert!(s.is_locked());
        s.reset();
        assert!(!s.is_locked());
        assert_eq!(s.failed_count, 0);
    }

    #[test]
    fn admin_unlock_clears_lockout() {
        let mut s = LockoutState::default();
        for _ in 0..5 {
            s.record_failure();
        }
        assert!(s.is_locked());
        s.admin_unlock();
        assert!(!s.is_locked());
    }

    #[test]
    fn window_reset_after_expiry() {
        let mut s = LockoutState::default();
        // Put 4 failures in the current window.
        for _ in 0..4 {
            s.record_failure();
        }
        // Simulate window expiry by moving window_started_at back.
        s.window_started_at = Utc::now() - Duration::seconds(WINDOW_SECS + 1);
        // Next failure starts a new window — count resets to 1.
        let locked = s.record_failure();
        assert!(!locked);
        assert_eq!(s.failed_count, 1);
    }

    #[test]
    fn lockout_expires_naturally() {
        let mut s = LockoutState::default();
        for _ in 0..5 {
            s.record_failure();
        }
        // Simulate lockout expiry.
        s.locked_until = Some(Utc::now() - Duration::seconds(1));
        assert!(!s.is_locked());
    }

    #[test]
    fn remaining_lockout_returns_none_when_not_locked() {
        let s = LockoutState::default();
        assert!(s.remaining_lockout().is_none());
    }
}
