//! TASK-AUTH-102 — MFA challenge state machine.
//!
//! Challenge FSM: `pending → consumed | expired | failed`
//!
//! A challenge is issued when a login attempt requires MFA. The challenge
//! is bound to a `(subject_id, factor_id)` pair and has a 5-minute TTL.
//! A challenge can only be consumed ONCE (replay protection).

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub const CHALLENGE_TTL_SECS: i64 = 300; // 5 minutes

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChallengeStatus {
    Pending,
    Consumed,
    Expired,
    Failed,
}

impl ChallengeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Consumed => "consumed",
            Self::Expired => "expired",
            Self::Failed => "failed",
        }
    }

    pub fn parse_status(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "consumed" => Some(Self::Consumed),
            "expired" => Some(Self::Expired),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MfaChallenge {
    pub challenge_id: Uuid,
    pub subject_id: Uuid,
    pub factor_id: Option<Uuid>,
    pub challenge_kind: String,
    pub status: ChallengeStatus,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

impl MfaChallenge {
    /// Create a new pending challenge with standard TTL.
    pub fn new(subject_id: Uuid, factor_id: Option<Uuid>, kind: &str) -> Self {
        let now = Utc::now();
        Self {
            challenge_id: Uuid::new_v4(),
            subject_id,
            factor_id,
            challenge_kind: kind.to_string(),
            status: ChallengeStatus::Pending,
            issued_at: now,
            expires_at: now + Duration::seconds(CHALLENGE_TTL_SECS),
            consumed_at: None,
        }
    }

    /// Attempt to consume this challenge. Returns `Err` if not pending or expired.
    pub fn try_consume(&mut self) -> Result<(), ChallengeError> {
        let now = Utc::now();
        if self.status != ChallengeStatus::Pending {
            return Err(ChallengeError::AlreadyConsumed);
        }
        if now > self.expires_at {
            self.status = ChallengeStatus::Expired;
            return Err(ChallengeError::Expired);
        }
        self.status = ChallengeStatus::Consumed;
        self.consumed_at = Some(now);
        Ok(())
    }

    /// Mark the challenge as failed (e.g. wrong code submitted).
    pub fn mark_failed(&mut self) {
        self.status = ChallengeStatus::Failed;
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChallengeError {
    #[error("challenge already consumed or in terminal state")]
    AlreadyConsumed,
    #[error("challenge expired")]
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_challenge_is_pending() {
        let c = MfaChallenge::new(Uuid::new_v4(), None, "totp");
        assert_eq!(c.status, ChallengeStatus::Pending);
        assert!(c.consumed_at.is_none());
    }

    #[test]
    fn consume_transitions_to_consumed() {
        let mut c = MfaChallenge::new(Uuid::new_v4(), None, "totp");
        assert!(c.try_consume().is_ok());
        assert_eq!(c.status, ChallengeStatus::Consumed);
        assert!(c.consumed_at.is_some());
    }

    #[test]
    fn double_consume_rejected() {
        let mut c = MfaChallenge::new(Uuid::new_v4(), None, "totp");
        c.try_consume().unwrap();
        assert!(c.try_consume().is_err());
    }

    #[test]
    fn expired_challenge_rejected() {
        let mut c = MfaChallenge::new(Uuid::new_v4(), None, "totp");
        // Force expiry
        c.expires_at = Utc::now() - Duration::seconds(1);
        let result = c.try_consume();
        assert!(result.is_err());
        assert_eq!(c.status, ChallengeStatus::Expired);
    }

    #[test]
    fn mark_failed_transitions_status() {
        let mut c = MfaChallenge::new(Uuid::new_v4(), None, "totp");
        c.mark_failed();
        assert_eq!(c.status, ChallengeStatus::Failed);
    }

    #[test]
    fn status_round_trip() {
        for s in &["pending", "consumed", "expired", "failed"] {
            let parsed = ChallengeStatus::parse_status(s).unwrap();
            assert_eq!(parsed.as_str(), *s);
        }
    }
}
