//! Domain models for tenant + subject. Plain serde structs that round-trip
//! between Postgres and JSON.

use chrono::{DateTime, Utc};
use cyberos_types::{SubjectId, TenantId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub slug: String,
    pub display_name: String,
    pub country: String,   // ISO-3166-1 alpha-2
    pub plan_tier: String, // 'starter' | 'team' | 'enterprise' | 'sandbox'
    pub status: String,    // 'active' | 'terminating' | 'terminated' | 'hostile'
    pub residency: String, // 'sg-1' | 'eu-1' | 'us-1' | 'vn-1'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTenantRequest {
    pub slug: String,
    pub display_name: String,
    #[serde(default = "default_country")]
    pub country: String,
    #[serde(default = "default_plan_tier")]
    pub plan_tier: String,
    #[serde(default = "default_residency")]
    pub residency: String,
}

fn default_country() -> String {
    "VN".into()
}
fn default_plan_tier() -> String {
    "starter".into()
}
fn default_residency() -> String {
    "sg-1".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: SubjectId,
    pub tenant_id: TenantId,
    pub handle: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub kind: String,   // 'human' | 'agent' | 'system'
    pub status: String, // 'active' | 'revoked' | 'pending'
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSubjectRequest {
    pub handle: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    #[serde(default = "default_kind")]
    pub kind: String,
    /// Required when kind = 'human'. Plain text — server hashes with bcrypt.
    pub password: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
}

fn default_kind() -> String {
    "human".into()
}
