//! Audit rows for proxied queries (TASK-OBS-002 §1 #9, §8).
//!
//! Decoupled and best-effort by design: a query must still succeed if the sink can't write (task §10),
//! and the proxy is stateless, so it does not write Postgres directly the way auth's memory_bridge
//! does. Rows carry the SHA-256 of the query, never the raw query (§1 #9 / §2), because queries can
//! encode tenant-business semantics. The real memory sink (direct l1_audit_log write or an ingest
//! POST) is wired in a follow-up; this slice defines the row shapes, the hash, and a pluggable sink.

use crate::error::Backend;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Lowercased backend name used in audit payloads and metrics labels.
pub fn backend_name(b: Backend) -> &'static str {
    match b {
        Backend::Prometheus => "prometheus",
        Backend::Loki => "loki",
        Backend::Tempo => "tempo",
    }
}

/// SHA-256 of a query, lowercase hex (64 chars). Logged instead of the raw query.
pub fn query_sha256(query: &str) -> String {
    let digest = Sha256::digest(query.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// An audit row destined for the memory chain.
#[derive(Debug, Clone)]
pub struct AuditRow {
    pub kind: &'static str,
    pub payload: Value,
}

/// `obs.query_proxied` - one per successfully proxied query (task §8). `outcome` is `proxied` or
/// `root_admin_unfiltered`.
pub fn query_proxied(
    tenant_id: &str,
    caller_subject_id: &str,
    backend: Backend,
    query: &str,
    outcome: &str,
    request_id: &str,
) -> AuditRow {
    AuditRow {
        kind: "obs.query_proxied",
        payload: json!({
            "tenant_id": tenant_id,
            "caller_subject_id": caller_subject_id,
            "backend": backend_name(backend),
            "query_sha256": query_sha256(query),
            "outcome": outcome,
            "request_id": request_id,
        }),
    }
}

/// `obs.cross_tenant_query_attempt` (sev-1) - a caller supplied a `tenant_id` label (task §1 #4 / §8).
pub fn cross_tenant_query_attempt(
    caller_tenant_id: &str,
    query: &str,
    request_id: &str,
) -> AuditRow {
    AuditRow {
        kind: "obs.cross_tenant_query_attempt",
        payload: json!({
            "caller_tenant_id": caller_tenant_id,
            "query_sha256": query_sha256(query),
            "request_id": request_id,
        }),
    }
}

/// Where audit rows go. Best-effort: implementations must not fail the caller (task §10).
pub trait AuditSink: Send + Sync {
    fn emit(&self, row: &AuditRow);
}

/// In-memory sink for tests and the §5 `memory_test_helper` role; also a usable dev default.
#[derive(Default)]
pub struct RecordingSink {
    pub rows: std::sync::Mutex<Vec<AuditRow>>,
}

impl RecordingSink {
    pub fn latest(&self, kind: &str) -> Option<AuditRow> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|r| r.kind == kind)
            .cloned()
    }
}

impl AuditSink for RecordingSink {
    fn emit(&self, row: &AuditRow) {
        self.rows.lock().unwrap().push(row.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_64_lowercase_hex() {
        let s = query_sha256("rate(foo[5m])");
        assert_eq!(s.len(), 64);
        assert!(s
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn query_proxied_row_shape() {
        let r = query_proxied(
            "org:cyberskill",
            "sub-1",
            Backend::Prometheus,
            "rate(foo[5m])",
            "proxied",
            "obs_1",
        );
        assert_eq!(r.kind, "obs.query_proxied");
        assert_eq!(r.payload["tenant_id"], "org:cyberskill");
        assert_eq!(r.payload["backend"], "prometheus");
        assert_eq!(r.payload["query_sha256"].as_str().unwrap().len(), 64);
        assert_eq!(r.payload["outcome"], "proxied");
    }

    #[test]
    fn recording_sink_collects_and_finds_latest() {
        let sink = RecordingSink::default();
        sink.emit(&cross_tenant_query_attempt(
            "org:cyberskill",
            "foo{tenant_id=\"other\"}",
            "obs_2",
        ));
        assert_eq!(sink.rows.lock().unwrap().len(), 1);
        let row = sink.latest("obs.cross_tenant_query_attempt").unwrap();
        assert_eq!(row.payload["caller_tenant_id"], "org:cyberskill");
    }
}
