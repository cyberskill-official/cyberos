//! Closed Resource × Action enums backing the FR-AUTH-101 permission matrix.
//!
//! Per DEC-122 + AUTHORING §3.4: these enums are CLOSED. Adding a 41st
//! resource or a 6th action is an ADR-gated change.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// The 40 cross-module resource surfaces governed by RBAC. One per cross-module
/// surface that needs role-gated access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Resource {
    // AUTH internals
    Subject,
    Tenant,
    RoleAssignment,
    JwtJwks,
    AuditRow,
    // CRM
    CrmAccount,
    CrmContact,
    CrmDeal,
    // PROJ
    ProjIssue,
    ProjEngagement,
    ProjRateCard,
    ProjTimeline,
    // TIME
    TimeEntry,
    TimeExpense,
    // INV
    InvInvoice,
    InvPayment,
    InvHoaDon,
    // KB
    KbDocument,
    KbRunbook,
    // HR
    HrMember,
    HrContract,
    HrLeave,
    HrCccdPhoto,
    // REW
    RewPayslip,
    RewBpLedger,
    // ESOP
    EsopGrant,
    EsopValuation,
    // LEARN
    LearnSkill,
    LearnCertification,
    // OKR
    OkrObjective,
    OkrKr,
    // RES
    ResAllocation,
    // DOC
    DocDocument,
    DocSignature,
    // EMAIL
    EmailThread,
    // CHAT
    ChatChannel,
    ChatMessage,
    // CUO
    CuoChain,
    // memory
    MemoryMemory,
    // OBS
    ObsAlert,
}

impl Resource {
    pub const ALL: [Resource; 40] = [
        Resource::Subject,
        Resource::Tenant,
        Resource::RoleAssignment,
        Resource::JwtJwks,
        Resource::AuditRow,
        Resource::CrmAccount,
        Resource::CrmContact,
        Resource::CrmDeal,
        Resource::ProjIssue,
        Resource::ProjEngagement,
        Resource::ProjRateCard,
        Resource::ProjTimeline,
        Resource::TimeEntry,
        Resource::TimeExpense,
        Resource::InvInvoice,
        Resource::InvPayment,
        Resource::InvHoaDon,
        Resource::KbDocument,
        Resource::KbRunbook,
        Resource::HrMember,
        Resource::HrContract,
        Resource::HrLeave,
        Resource::HrCccdPhoto,
        Resource::RewPayslip,
        Resource::RewBpLedger,
        Resource::EsopGrant,
        Resource::EsopValuation,
        Resource::LearnSkill,
        Resource::LearnCertification,
        Resource::OkrObjective,
        Resource::OkrKr,
        Resource::ResAllocation,
        Resource::DocDocument,
        Resource::DocSignature,
        Resource::EmailThread,
        Resource::ChatChannel,
        Resource::ChatMessage,
        Resource::CuoChain,
        Resource::MemoryMemory,
        Resource::ObsAlert,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Resource::Subject => "subject",
            Resource::Tenant => "tenant",
            Resource::RoleAssignment => "role-assignment",
            Resource::JwtJwks => "jwt-jwks",
            Resource::AuditRow => "audit-row",
            Resource::CrmAccount => "crm-account",
            Resource::CrmContact => "crm-contact",
            Resource::CrmDeal => "crm-deal",
            Resource::ProjIssue => "proj-issue",
            Resource::ProjEngagement => "proj-engagement",
            Resource::ProjRateCard => "proj-rate-card",
            Resource::ProjTimeline => "proj-timeline",
            Resource::TimeEntry => "time-entry",
            Resource::TimeExpense => "time-expense",
            Resource::InvInvoice => "inv-invoice",
            Resource::InvPayment => "inv-payment",
            Resource::InvHoaDon => "inv-hoa-don",
            Resource::KbDocument => "kb-document",
            Resource::KbRunbook => "kb-runbook",
            Resource::HrMember => "hr-member",
            Resource::HrContract => "hr-contract",
            Resource::HrLeave => "hr-leave",
            Resource::HrCccdPhoto => "hr-cccd-photo",
            Resource::RewPayslip => "rew-payslip",
            Resource::RewBpLedger => "rew-bp-ledger",
            Resource::EsopGrant => "esop-grant",
            Resource::EsopValuation => "esop-valuation",
            Resource::LearnSkill => "learn-skill",
            Resource::LearnCertification => "learn-certification",
            Resource::OkrObjective => "okr-objective",
            Resource::OkrKr => "okr-kr",
            Resource::ResAllocation => "res-allocation",
            Resource::DocDocument => "doc-document",
            Resource::DocSignature => "doc-signature",
            Resource::EmailThread => "email-thread",
            Resource::ChatChannel => "chat-channel",
            Resource::ChatMessage => "chat-message",
            Resource::CuoChain => "cuo-chain",
            Resource::MemoryMemory => "memory-memory",
            Resource::ObsAlert => "obs-alert",
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<Resource> for String {
    fn from(r: Resource) -> String {
        r.as_str().to_string()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown resource: {0:?}")]
pub struct UnknownResource(pub String);

impl FromStr for Resource {
    type Err = UnknownResource;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for r in Resource::ALL {
            if r.as_str() == s {
                return Ok(r);
            }
        }
        Err(UnknownResource(s.to_string()))
    }
}
impl TryFrom<String> for Resource {
    type Error = UnknownResource;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Resource::from_str(&s)
    }
}

// ---------------------------------------------------------------------------
// Action — the 5 verbs RBAC recognises.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Action {
    /// Read-only access.
    Read,
    /// Mutation (POST / PATCH / PUT / DELETE generally).
    Write,
    /// Privileged admin action — grant/revoke, configuration change.
    Admin,
    /// Dual-signoff approval workflow leg (CFO+CEO co-sign, etc.).
    Approve,
    /// E-signature on DOC + hóa đơn emissions.
    Sign,
}

impl Action {
    pub const ALL: [Action; 5] = [
        Action::Read,
        Action::Write,
        Action::Admin,
        Action::Approve,
        Action::Sign,
    ];
    pub const fn as_str(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Admin => "admin",
            Action::Approve => "approve",
            Action::Sign => "sign",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl From<Action> for String {
    fn from(a: Action) -> String {
        a.as_str().to_string()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown action: {0:?}")]
pub struct UnknownAction(pub String);

impl FromStr for Action {
    type Err = UnknownAction;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for a in Action::ALL {
            if a.as_str() == s {
                return Ok(a);
            }
        }
        Err(UnknownAction(s.to_string()))
    }
}
impl TryFrom<String> for Action {
    type Error = UnknownAction;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Action::from_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_count_is_exactly_40() {
        assert_eq!(Resource::ALL.len(), 40);
    }

    #[test]
    fn actions_count_is_exactly_5() {
        assert_eq!(Action::ALL.len(), 5);
    }

    #[test]
    fn resources_round_trip() {
        for r in Resource::ALL {
            assert_eq!(Resource::from_str(r.as_str()).unwrap(), r);
        }
    }

    #[test]
    fn actions_round_trip() {
        for a in Action::ALL {
            assert_eq!(Action::from_str(a.as_str()).unwrap(), a);
        }
    }
}
