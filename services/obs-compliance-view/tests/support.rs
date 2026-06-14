#![allow(dead_code)]

use std::sync::Arc;

use chrono::{Duration, Utc};
use cyberos_obs_compliance_view::auth::{issue_local_auditor_token, AuthConfig, Claims};
use cyberos_obs_compliance_view::chain_proof::ChainProofSigner;
use cyberos_obs_compliance_view::memory::{AuditRow, InMemoryBackend};
use cyberos_obs_compliance_view::router::ViewQuery;
use cyberos_obs_compliance_view::views::Format;
use cyberos_obs_compliance_view::AppState;

pub const SECRET: &str = "secret";

pub fn rows() -> Vec<AuditRow> {
    let now = Utc::now();
    vec![
        AuditRow::new(
            now,
            "ai.invocation",
            "t1",
            serde_json::json!({"persona_handle": "cto"}),
        ),
        AuditRow::new(
            now,
            "ai.persona_loaded",
            "t1",
            serde_json::json!({"persona_handle": "cto"}),
        ),
        AuditRow::new(
            now,
            "auth.token_issued",
            "t1",
            serde_json::json!({"subject": "s1"}),
        ),
        AuditRow::new(
            now,
            "dsar.export_completed",
            "t1",
            serde_json::json!({"request": "r1"}),
        ),
        AuditRow::new(
            now,
            "obs.langsmith_export_enabled",
            "t1",
            serde_json::json!({"consent": true}),
        ),
        AuditRow::new(
            now,
            "obs.alert_triaged",
            "t1",
            serde_json::json!({"alert": "a1"}),
        ),
        AuditRow::new(
            now,
            "asset.inventory_snapshot",
            "t1",
            serde_json::json!({"assets": 3}),
        ),
        AuditRow::new(
            now,
            "risk.assessment_completed",
            "t1",
            serde_json::json!({"risk": "low"}),
        ),
        AuditRow::new(
            now,
            "ai.invocation",
            "t2",
            serde_json::json!({"persona_handle": "cfo"}),
        ),
    ]
}

pub fn backend() -> Arc<InMemoryBackend> {
    Arc::new(InMemoryBackend::with_rows(rows()))
}

pub fn signer() -> ChainProofSigner {
    ChainProofSigner::from_seed([7u8; 32])
}

pub fn claims(tenant_id: &str) -> Claims {
    let now = Utc::now();
    Claims {
        sub: "auditor-1".to_string(),
        tenant_id: tenant_id.to_string(),
        roles: vec!["external_auditor".to_string()],
        exp: (now + Duration::days(30)).timestamp(),
        iat: now.timestamp(),
        iss: None,
        aud: None,
        request_id: Some("req-1".to_string()),
    }
}

pub fn query() -> ViewQuery {
    ViewQuery {
        since: Utc::now() - Duration::days(30),
        until: Utc::now() + Duration::seconds(1),
        format: Format::Json,
        tenant_id: None,
    }
}

pub fn app_state(memory: Arc<InMemoryBackend>) -> AppState {
    AppState::new(AuthConfig::local(SECRET), memory, signer())
}

pub fn token(roles: Vec<String>) -> String {
    issue_local_auditor_token(SECRET, "t1", "auditor-1", roles, 30)
}
