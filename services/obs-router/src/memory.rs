//! Memory audit row emission for OBS alert routing.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::AsyncWriteExt;

/// Memory audit row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditRow {
    /// Memory kind, for example `obs.alert_triaged`.
    pub kind: String,
    /// Payload.
    pub payload: serde_json::Value,
}

/// Audit sink error.
#[derive(Debug, Error)]
pub enum AuditError {
    /// Filesystem error.
    #[error("audit_io: {0}")]
    Io(String),
    /// Serialization error.
    #[error("audit_encode: {0}")]
    Encode(String),
}

/// Sink capable of emitting memory audit rows.
#[async_trait]
pub trait AuditSink: Send + Sync + std::fmt::Debug {
    /// Emit one audit row.
    async fn emit(&self, row: AuditRow) -> Result<(), AuditError>;
}

/// JSONL sink used by local deployments and tests when the canonical writer is
/// not mounted into the router container.
#[derive(Debug, Clone)]
pub struct JsonlAuditSink {
    path: PathBuf,
}

impl JsonlAuditSink {
    /// Create a sink at the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Default path, outside implementation commits.
    pub fn default_target_path() -> PathBuf {
        Path::new("target")
            .join("cuo-workflow")
            .join("obs-router")
            .join("audit.jsonl")
    }
}

#[async_trait]
impl AuditSink for JsonlAuditSink {
    async fn emit(&self, row: AuditRow) -> Result<(), AuditError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| AuditError::Io(err.to_string()))?;
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map_err(|err| AuditError::Io(err.to_string()))?;
        let mut encoded =
            serde_json::to_vec(&row).map_err(|err| AuditError::Encode(err.to_string()))?;
        encoded.push(b'\n');
        file.write_all(&encoded)
            .await
            .map_err(|err| AuditError::Io(err.to_string()))
    }
}
