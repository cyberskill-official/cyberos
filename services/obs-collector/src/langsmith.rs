//! FR-OBS-004 — LangSmith AI trace records tied to operational trace ids.

use serde::{Deserialize, Serialize};

/// Minimal bridge row emitted for every AI trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiTraceLink {
    /// W3C trace id from the operational request.
    pub trace_id: String,
    /// LangSmith run id.
    pub langsmith_run_id: String,
    /// Tenant scope.
    pub tenant_id: String,
    /// Model alias or concrete model.
    pub model: String,
}

/// Validate and construct a LangSmith bridge row.
pub fn link_trace(
    trace_id: &str,
    langsmith_run_id: &str,
    tenant_id: &str,
    model: &str,
) -> Result<AiTraceLink, String> {
    if trace_id.len() != 32 || !trace_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid_trace_id".into());
    }
    if langsmith_run_id.trim().is_empty() {
        return Err("langsmith_run_id_required".into());
    }
    if tenant_id.trim().is_empty() {
        return Err("tenant_id_required".into());
    }
    Ok(AiTraceLink {
        trace_id: trace_id.to_ascii_lowercase(),
        langsmith_run_id: langsmith_run_id.into(),
        tenant_id: tenant_id.into(),
        model: model.into(),
    })
}
