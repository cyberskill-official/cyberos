//! Read-only audit-row query abstraction.

use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::error::ViewError;

/// Audit row projected into a compliance view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRow {
    /// Row timestamp.
    pub ts: DateTime<Utc>,
    /// Canonical kind.
    pub kind: String,
    /// Tenant scope.
    pub tenant_id: String,
    /// Row payload.
    pub payload: serde_json::Value,
    /// Payload SHA-256 hash.
    pub payload_hash: String,
}

impl AuditRow {
    /// Build a row and compute payload hash.
    pub fn new(
        ts: DateTime<Utc>,
        kind: impl Into<String>,
        tenant_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        let payload_hash = hash_payload(&payload);
        Self {
            ts,
            kind: kind.into(),
            tenant_id: tenant_id.into(),
            payload,
            payload_hash,
        }
    }
}

/// Query and audit interface.
#[async_trait]
pub trait MemoryBackend: Send + Sync + std::fmt::Debug {
    /// Query rows read-only.
    async fn query(
        &self,
        tenant_id: &str,
        kinds: &[&str],
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<AuditRow>, ViewError>;

    /// Current signed tree head at query time.
    async fn current_signed_tree_head(&self) -> Result<String, ViewError>;

    /// Emit the auditor self-audit row.
    async fn emit_accessed(&self, row: AuditRow) -> Result<(), ViewError>;
}

/// In-memory backend for tests.
#[derive(Debug, Default)]
pub struct InMemoryBackend {
    rows: Mutex<Vec<AuditRow>>,
    emitted: Mutex<Vec<AuditRow>>,
}

impl InMemoryBackend {
    /// Create with rows.
    pub fn with_rows(rows: Vec<AuditRow>) -> Self {
        Self {
            rows: Mutex::new(rows),
            emitted: Mutex::new(Vec::new()),
        }
    }

    /// Emitted audit rows.
    pub fn emitted(&self) -> Vec<AuditRow> {
        self.emitted.lock().unwrap().clone()
    }
}

#[async_trait]
impl MemoryBackend for InMemoryBackend {
    async fn query(
        &self,
        tenant_id: &str,
        kinds: &[&str],
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<AuditRow>, ViewError> {
        let mut rows: Vec<_> = self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|row| row.tenant_id == tenant_id)
            .filter(|row| row.ts >= since && row.ts <= until)
            .filter(|row| {
                kinds
                    .iter()
                    .any(|kind| row.kind == *kind || row.kind.starts_with(*kind))
            })
            .cloned()
            .collect();
        rows.sort_by(|a, b| {
            (a.ts, &a.kind, &a.payload_hash).cmp(&(b.ts, &b.kind, &b.payload_hash))
        });
        Ok(rows)
    }

    async fn current_signed_tree_head(&self) -> Result<String, ViewError> {
        Ok("test-sth".to_string())
    }

    async fn emit_accessed(&self, row: AuditRow) -> Result<(), ViewError> {
        self.emitted.lock().unwrap().push(row);
        Ok(())
    }
}

/// JSONL backend for local deployment.
#[derive(Debug, Clone)]
pub struct JsonlBackend {
    rows_path: PathBuf,
    audit_path: PathBuf,
}

impl JsonlBackend {
    /// Create a backend.
    pub fn new(rows_path: impl Into<PathBuf>, audit_path: impl Into<PathBuf>) -> Self {
        Self {
            rows_path: rows_path.into(),
            audit_path: audit_path.into(),
        }
    }
}

#[async_trait]
impl MemoryBackend for JsonlBackend {
    async fn query(
        &self,
        tenant_id: &str,
        kinds: &[&str],
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<AuditRow>, ViewError> {
        let raw = tokio::fs::read_to_string(&self.rows_path)
            .await
            .unwrap_or_default();
        let mut rows = Vec::new();
        for line in raw.lines().filter(|line| !line.trim().is_empty()) {
            let row: AuditRow = serde_json::from_str(line)
                .map_err(|err| ViewError::MemoryQueryFailed(err.to_string()))?;
            if row.tenant_id == tenant_id
                && row.ts >= since
                && row.ts <= until
                && kinds
                    .iter()
                    .any(|kind| row.kind == *kind || row.kind.starts_with(*kind))
            {
                rows.push(row);
            }
        }
        rows.sort_by(|a, b| {
            (a.ts, &a.kind, &a.payload_hash).cmp(&(b.ts, &b.kind, &b.payload_hash))
        });
        Ok(rows)
    }

    async fn current_signed_tree_head(&self) -> Result<String, ViewError> {
        Ok("jsonl-local-sth".to_string())
    }

    async fn emit_accessed(&self, row: AuditRow) -> Result<(), ViewError> {
        if let Some(parent) = self.audit_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| ViewError::MemoryQueryFailed(err.to_string()))?;
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_path)
            .await
            .map_err(|err| ViewError::MemoryQueryFailed(err.to_string()))?;
        let mut encoded = serde_json::to_vec(&row)
            .map_err(|err| ViewError::MemoryQueryFailed(err.to_string()))?;
        encoded.push(b'\n');
        file.write_all(&encoded)
            .await
            .map_err(|err| ViewError::MemoryQueryFailed(err.to_string()))
    }
}

/// Hash a payload with SHA-256.
pub fn hash_payload(payload: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(payload).unwrap_or_default();
    hex(&Sha256::digest(&bytes))
}

/// Hex encode bytes.
pub fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}
