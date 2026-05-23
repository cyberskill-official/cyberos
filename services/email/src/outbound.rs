//! FR-EMAIL-009 — outbound 1:1 send state machine.

use crate::audit::email_events::hash16;
use crate::delivery_auth::{sign_message, DkimMaterial, DkimOutcome};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendStatus {
    Drafting,
    Queued,
    Sent,
    BouncedHard,
    BouncedSoft,
    Complaint,
    Suppressed,
}

impl SendStatus {
    pub const ALL: [Self; 7] = [
        Self::Drafting,
        Self::Queued,
        Self::Sent,
        Self::BouncedHard,
        Self::BouncedSoft,
        Self::Complaint,
        Self::Suppressed,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Drafting => "drafting",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::BouncedHard => "bounced_hard",
            Self::BouncedSoft => "bounced_soft",
            Self::Complaint => "complaint",
            Self::Suppressed => "suppressed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    HardBounce,
    Complaint,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeRequest {
    pub tenant_id: Uuid,
    pub sender_subject_id: Uuid,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub in_reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftMessage {
    pub message_id: Uuid,
    pub tenant_id: Uuid,
    pub sender_subject_id: Uuid,
    pub recipients_hash16: Vec<String>,
    pub subject_sha256: String,
    pub body_sha256: String,
    pub status: SendStatus,
    pub confirm_token: Uuid,
    pub confirm_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OutboundError {
    #[error("send requires a valid confirmation token")]
    ConfirmTokenInvalid,
    #[error("confirmation token expired")]
    ConfirmTokenExpired,
    #[error("recipient is suppressed: {0}")]
    RecipientSuppressed(String),
    #[error("rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Default)]
pub struct SuppressionList {
    entries: HashMap<(Uuid, String), SuppressionReason>,
}

impl SuppressionList {
    pub fn suppress(&mut self, tenant_id: Uuid, address: &str, reason: SuppressionReason) {
        self.entries.insert((tenant_id, hash16(address)), reason);
    }

    pub fn is_suppressed(&self, tenant_id: Uuid, address: &str) -> bool {
        self.entries.contains_key(&(tenant_id, hash16(address)))
    }
}

#[derive(Default)]
pub struct SendRateLimiter {
    sends_by_hour: HashMap<(Uuid, i64), HashSet<Uuid>>,
}

impl SendRateLimiter {
    pub fn check_and_record(
        &mut self,
        sender_subject_id: Uuid,
        message_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<(), OutboundError> {
        let hour = now.timestamp() / 3600;
        let bucket = self
            .sends_by_hour
            .entry((sender_subject_id, hour))
            .or_default();
        if bucket.len() >= 100 && !bucket.contains(&message_id) {
            return Err(OutboundError::RateLimitExceeded);
        }
        bucket.insert(message_id);
        Ok(())
    }
}

pub fn compose(
    req: &ComposeRequest,
    suppressions: &SuppressionList,
    now: DateTime<Utc>,
) -> Result<DraftMessage, OutboundError> {
    for recipient in req.to.iter().chain(req.cc.iter()).chain(req.bcc.iter()) {
        if suppressions.is_suppressed(req.tenant_id, recipient) {
            return Err(OutboundError::RecipientSuppressed(hash16(recipient)));
        }
    }

    Ok(DraftMessage {
        message_id: Uuid::new_v4(),
        tenant_id: req.tenant_id,
        sender_subject_id: req.sender_subject_id,
        recipients_hash16: req
            .to
            .iter()
            .chain(req.cc.iter())
            .chain(req.bcc.iter())
            .map(|r| hash16(r))
            .collect(),
        subject_sha256: sha256_hex(&req.subject),
        body_sha256: sha256_hex(&req.body_text),
        status: SendStatus::Drafting,
        confirm_token: Uuid::new_v4(),
        confirm_expires_at: now + Duration::minutes(5),
    })
}

pub fn queue_send(
    draft: &mut DraftMessage,
    confirm_token: Uuid,
    now: DateTime<Utc>,
    limiter: &mut SendRateLimiter,
    dkim_key: Option<&DkimMaterial>,
) -> Result<DkimOutcome, OutboundError> {
    if draft.confirm_token != confirm_token {
        return Err(OutboundError::ConfirmTokenInvalid);
    }
    if now > draft.confirm_expires_at {
        return Err(OutboundError::ConfirmTokenExpired);
    }
    limiter.check_and_record(draft.sender_subject_id, draft.message_id, now)?;
    let signed = sign_message(&draft.body_sha256, dkim_key);
    draft.status = SendStatus::Queued;
    Ok(signed.outcome)
}

pub fn handle_bounce(
    draft: &mut DraftMessage,
    hard: bool,
    primary_recipient: &str,
    suppressions: &mut SuppressionList,
) {
    if hard {
        draft.status = SendStatus::BouncedHard;
        suppressions.suppress(
            draft.tenant_id,
            primary_recipient,
            SuppressionReason::HardBounce,
        );
    } else {
        draft.status = SendStatus::BouncedSoft;
    }
}

pub fn handle_complaint(
    draft: &mut DraftMessage,
    primary_recipient: &str,
    suppressions: &mut SuppressionList,
) {
    draft.status = SendStatus::Complaint;
    suppressions.suppress(
        draft.tenant_id,
        primary_recipient,
        SuppressionReason::Complaint,
    );
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub status: &'static str,
    pub trace_id: Option<String>,
}

pub fn audit_row(
    kind: &'static str,
    draft: &DraftMessage,
    trace_id: Option<&str>,
) -> OutboundAuditRow {
    OutboundAuditRow {
        kind,
        tenant_id: draft.tenant_id,
        message_id: draft.message_id,
        status: draft.status.as_str(),
        trace_id: trace_id.map(str::to_owned),
    }
}

fn sha256_hex(input: &str) -> String {
    let digest = sha2::Sha256::digest(input.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery_auth::DkimKeyKind;

    fn req() -> ComposeRequest {
        ComposeRequest {
            tenant_id: Uuid::new_v4(),
            sender_subject_id: Uuid::new_v4(),
            to: vec!["customer@example.com".into()],
            cc: vec![],
            bcc: vec![],
            subject: "Hello".into(),
            body_text: "Body".into(),
            in_reply_to: None,
        }
    }

    fn dkim_key(tenant_id: Uuid) -> DkimMaterial {
        DkimMaterial {
            tenant_id,
            selector: "cyberos".into(),
            domain: "example.com".into(),
            key_kind: DkimKeyKind::Ed25519,
            public_dns_txt: "v=DKIM1; p=abc".into(),
            signing_secret: Some("secret".into()),
        }
    }

    #[test]
    fn send_status_cardinality_is_seven() {
        assert_eq!(SendStatus::ALL.len(), 7);
    }

    #[test]
    fn compose_returns_five_minute_confirm_token() {
        let now = Utc::now();
        let draft = compose(&req(), &SuppressionList::default(), now).unwrap();
        assert_eq!(draft.status, SendStatus::Drafting);
        assert_eq!(draft.confirm_expires_at, now + Duration::minutes(5));
    }

    #[test]
    fn send_requires_valid_confirm_token() {
        let now = Utc::now();
        let mut draft = compose(&req(), &SuppressionList::default(), now).unwrap();
        let mut limiter = SendRateLimiter::default();
        let err = queue_send(&mut draft, Uuid::new_v4(), now, &mut limiter, None).unwrap_err();
        assert_eq!(err, OutboundError::ConfirmTokenInvalid);
    }

    #[test]
    fn valid_confirm_queues_and_dkim_signs() {
        let now = Utc::now();
        let mut draft = compose(&req(), &SuppressionList::default(), now).unwrap();
        let token = draft.confirm_token;
        let key = dkim_key(draft.tenant_id);
        let mut limiter = SendRateLimiter::default();
        let outcome = queue_send(&mut draft, token, now, &mut limiter, Some(&key)).unwrap();
        assert_eq!(draft.status, SendStatus::Queued);
        assert_eq!(outcome, DkimOutcome::SignedEd25519);
    }

    #[test]
    fn suppressed_recipient_blocks_compose() {
        let req = req();
        let mut suppressions = SuppressionList::default();
        suppressions.suppress(
            req.tenant_id,
            "customer@example.com",
            SuppressionReason::Manual,
        );
        let err = compose(&req, &suppressions, Utc::now()).unwrap_err();
        assert!(matches!(err, OutboundError::RecipientSuppressed(_)));
    }

    #[test]
    fn hard_bounce_and_complaint_suppress_recipient() {
        let req = req();
        let mut suppressions = SuppressionList::default();
        let mut draft = compose(&req, &suppressions, Utc::now()).unwrap();
        handle_bounce(&mut draft, true, "customer@example.com", &mut suppressions);
        assert_eq!(draft.status, SendStatus::BouncedHard);
        assert!(suppressions.is_suppressed(req.tenant_id, "customer@example.com"));

        let mut draft = compose(
            &ComposeRequest {
                to: vec!["other@example.com".into()],
                ..req.clone()
            },
            &suppressions,
            Utc::now(),
        )
        .unwrap();
        handle_complaint(&mut draft, "other@example.com", &mut suppressions);
        assert_eq!(draft.status, SendStatus::Complaint);
        assert!(suppressions.is_suppressed(req.tenant_id, "other@example.com"));
    }

    #[test]
    fn rate_limit_blocks_101st_distinct_message() {
        let now = Utc::now();
        let sender = Uuid::new_v4();
        let mut limiter = SendRateLimiter::default();
        for _ in 0..100 {
            limiter
                .check_and_record(sender, Uuid::new_v4(), now)
                .unwrap();
        }
        let err = limiter
            .check_and_record(sender, Uuid::new_v4(), now)
            .unwrap_err();
        assert_eq!(err, OutboundError::RateLimitExceeded);
    }
}
