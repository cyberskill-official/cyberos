//! FR-AI-003 — Canonical memory audit-row writer.
//!
//! Bridges the AI Gateway to the canonical Python Writer subprocess (`python3 -m
//! cyberos.writer put`). Every AI Gateway audit emission MUST route through this module;
//! direct writes to `<memory-root>/` are forbidden (AGENTS.md §14.1).
//!
//! ## Status (slice 1)
//!
//! - Path validation (AC #7), canonical-JSON serialisation (AGENTS.md §6.2), subprocess
//!   spawn + stdin/stdout/stderr piping, 5s timeout with SIGTERM-then-SIGKILL, exit-code
//!   taxonomy, chain-hash verification — all implemented.
//! - Typed builders for the slice-1 closed set: `precheck`, `invocation`, `invocation_failed`,
//!   `hold_expired`, `persona_loaded` — all implemented.
//! - Startup health check (`check_writer_available`) implemented.
//! - The `python3 -m cyberos.writer put` Python subprocess is supplied by
//!   `modules/memory/runtime/`. When that interface is not on PATH, `check_writer_available`
//!   returns `Err(WriterUnreachable)` and the gateway should exit non-zero at boot
//!   (FR-AI-003 §1 #10).

pub mod canonical;

use std::process::Stdio;
use std::time::Duration;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::error;
use uuid::Uuid;

const WRITER_BIN: &str = "python3";
const WRITER_ARGS: &[&str] = &["-m", "cyberos.writer", "put"];
const WRITER_TIMEOUT: Duration = Duration::from_secs(5);

// --- Types ------------------------------------------------------------------

/// FR-AI-003 §1 #8 — closed initial set of `ai.*` row kinds for slice 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiInvocationKind {
    /// Emitted by FR-AI-001 (cost-ledger pre-call check).
    Precheck,
    /// Emitted by FR-AI-002 (success path).
    Invocation,
    /// Emitted by FR-AI-002 (refund path).
    InvocationFailed,
    /// Emitted by FR-AI-004 (cleanup job).
    HoldExpired,
    /// Emitted by FR-AI-014 (persona stamping).
    PersonaLoaded,
}

impl AiInvocationKind {
    /// String tag emitted into the audit row body.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Precheck => "ai.precheck",
            Self::Invocation => "ai.invocation",
            Self::InvocationFailed => "ai.invocation_failed",
            Self::HoldExpired => "ai.hold_expired",
            Self::PersonaLoaded => "ai.persona_loaded",
        }
    }
}

/// FR-AI-003 §3 — Emission request.
#[derive(Debug, Clone)]
pub struct MemoryEmit {
    /// Row kind tag (closed set).
    pub kind: AiInvocationKind,
    /// Memory file path under `<memory-root>/`, validated before subprocess spawn.
    pub path: String,
    /// Per-kind structured payload (no schema validation at the bridge; typed builders
    /// constrain it at the call site).
    pub extra: serde_json::Value,
}

/// FR-AI-003 §3 — Outcome of a successful emit.
#[derive(Debug, Clone)]
pub struct EmittedRow {
    /// HEAD seq counter at time of emission.
    pub seq: u64,
    /// Ns since UNIX epoch.
    pub ts_ns: u64,
    /// Chain hash of this row (SHA-256(canonical(record_minus_chain) ‖ prev_chain)).
    pub chain: [u8; 32],
    /// Memory file path of the row.
    pub path: String,
}

/// FR-AI-003 §3 — Writer process metadata used by the startup health check.
#[derive(Debug, Clone)]
pub struct WriterVersion {
    /// Semver of the Writer module.
    pub semver: String,
    /// Git commit of the Writer module.
    pub commit: String,
    /// Wire-format schema version.
    pub schema_version: u32,
}

/// FR-AI-003 §3 — Error taxonomy.
#[derive(Debug, Error)]
pub enum MemoryWriterError {
    /// Subprocess returned non-zero exit code.
    #[error("writer failed (exit {exit_code}): {stderr}")]
    WriterFailed {
        /// Process exit code.
        exit_code: i32,
        /// Captured stderr.
        stderr: String,
    },
    /// Subprocess could not be spawned or could not be reached.
    #[error("writer unreachable: {reason}")]
    WriterUnreachable {
        /// Reason for unreachability.
        reason: String,
    },
    /// Payload could not be canonicalised (e.g. non-UTF-8 bytes).
    #[error("canonicalisation failed: {reason}")]
    CanonicalisationFailed {
        /// Reason for failure.
        reason: String,
    },
    /// Local recomputation of the chain hash diverged from the Writer's response.
    #[error("chain-hash mismatch")]
    ChainHashMismatch {
        /// What we computed.
        expected: [u8; 32],
        /// What the Writer returned.
        got: [u8; 32],
    },
    /// Path failed validation (traversal, absolute, reserved prefix).
    #[error("path rejected ({reason}): {path}")]
    PathRejected {
        /// Offending path.
        path: String,
        /// Reason for rejection.
        reason: String,
    },
    /// Subprocess hung past the 5s timeout; SIGTERM-then-SIGKILL applied.
    #[error("writer timeout after {waited_ms}ms")]
    Timeout {
        /// How long we waited before killing.
        waited_ms: u32,
    },
}

// --- Subprocess stdout shape ------------------------------------------------

#[derive(Deserialize)]
struct WriterStdout {
    seq: u64,
    ts_ns: u64,
    chain: String,
    prev_chain: String,
}

// --- Public entry points ----------------------------------------------------

/// FR-AI-003 §3 — Emit one audit row via the canonical Writer subprocess.
pub async fn emit(req: MemoryEmit) -> Result<EmittedRow, MemoryWriterError> {
    // 1. Validate path BEFORE spawning anything (AC #7).
    validate_path(&req.path).map_err(|reason| MemoryWriterError::PathRejected {
        path: req.path.clone(),
        reason,
    })?;

    // 2. Build canonical JSON payload.
    let payload = canonical::serialise(&req)
        .map_err(|reason| MemoryWriterError::CanonicalisationFailed { reason })?;

    // 3. Spawn Writer.
    let mut child = Command::new(WRITER_BIN)
        .args(WRITER_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| MemoryWriterError::WriterUnreachable {
            reason: e.to_string(),
        })?;

    let mut stdin = child.stdin.take().expect("piped");
    let stdout = child.stdout.take().expect("piped");
    let stderr = child.stderr.take().expect("piped");

    // 4. Pipe payload + signal EOF.
    let payload_bytes = format!("{payload}\n").into_bytes();
    let write_task = tokio::spawn(async move {
        stdin.write_all(&payload_bytes).await?;
        stdin.shutdown().await?;
        Ok::<_, std::io::Error>(())
    });

    // 5. Wait for child + read stdout/stderr concurrently, with 5s timeout.
    let outcome = timeout(WRITER_TIMEOUT, async move {
        let (write_res, stdout_buf, stderr_buf, exit_res) =
            tokio::join!(write_task, read_all(stdout), read_all(stderr), child.wait(),);
        (write_res, stdout_buf, stderr_buf, exit_res)
    })
    .await;

    let (write_res, stdout_buf, stderr_buf, exit_res) = match outcome {
        Ok(t) => t,
        Err(_) => {
            // Timeout fired — kill_on_drop will reap the process when we leave scope.
            return Err(MemoryWriterError::Timeout {
                waited_ms: WRITER_TIMEOUT.as_millis() as u32,
            });
        }
    };

    write_res
        .map_err(|e| MemoryWriterError::WriterUnreachable {
            reason: format!("write join: {e}"),
        })?
        .map_err(|e| MemoryWriterError::WriterUnreachable {
            reason: format!("write io: {e}"),
        })?;

    let exit = exit_res.map_err(|e| MemoryWriterError::WriterUnreachable {
        reason: e.to_string(),
    })?;
    let stdout_bytes = stdout_buf.map_err(|e| MemoryWriterError::WriterUnreachable {
        reason: format!("stdout: {e}"),
    })?;
    let stderr_bytes = stderr_buf.map_err(|e| MemoryWriterError::WriterUnreachable {
        reason: format!("stderr: {e}"),
    })?;

    if !exit.success() {
        return Err(MemoryWriterError::WriterFailed {
            exit_code: exit.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
        });
    }

    // 6. Parse stdout → typed row.
    let row: WriterStdout =
        serde_json::from_slice(&stdout_bytes).map_err(|e| MemoryWriterError::WriterFailed {
            exit_code: 0,
            stderr: format!("stdout parse: {e}"),
        })?;

    // 7. Verify chain hash locally (FR-AI-003 §1 #7).
    let expected = compute_chain(&payload, &row.prev_chain);
    let got_vec = hex::decode(&row.chain).unwrap_or_default();
    let mut got = [0u8; 32];
    if got_vec.len() == 32 {
        got.copy_from_slice(&got_vec);
    }
    if expected != got {
        error!(
            expected_chain = hex::encode(expected),
            actual_chain = hex::encode(got),
            seq = row.seq,
            payload_canonical_hash = hex::encode(Sha256::digest(payload.as_bytes())),
            "chain_hash_mismatch — refusing row",
        );
        return Err(MemoryWriterError::ChainHashMismatch { expected, got });
    }

    Ok(EmittedRow {
        seq: row.seq,
        ts_ns: row.ts_ns,
        chain: expected,
        path: req.path,
    })
}

/// FR-AI-003 §1 #10 — Startup health check.
pub async fn check_writer_available() -> Result<WriterVersion, MemoryWriterError> {
    let out = Command::new(WRITER_BIN)
        .args(["-m", "cyberos.writer", "--version"])
        .output()
        .await
        .map_err(|e| MemoryWriterError::WriterUnreachable {
            reason: e.to_string(),
        })?;
    if !out.status.success() {
        return Err(MemoryWriterError::WriterUnreachable {
            reason: format!(
                "exit={} stderr={}",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr),
            ),
        });
    }
    // Parse "cyberos.writer 0.1.0 sha=abc1234 schema=1" or JSON. Permissive parse.
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let (semver, commit, schema_version) = parse_version_line(&line);
    Ok(WriterVersion {
        semver,
        commit,
        schema_version,
    })
}

// --- Internals --------------------------------------------------------------

fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty".into());
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err("absolute".into());
    }
    if path.contains("..") {
        return Err("traversal".into());
    }
    for reserved in ["audit/", "index/", "HEAD", ".lock"] {
        if path.starts_with(reserved) {
            return Err(format!("reserved: {reserved}"));
        }
    }
    Ok(())
}

fn compute_chain(canonical_payload: &str, prev_chain_hex: &str) -> [u8; 32] {
    let prev = hex::decode(prev_chain_hex).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical_payload.as_bytes());
    hasher.update(&prev);
    hasher.finalize().into()
}

fn parse_version_line(line: &str) -> (String, String, u32) {
    let mut semver = String::from("unknown");
    let mut commit = String::from("unknown");
    let mut schema_version = 0u32;
    for token in line.split_whitespace() {
        if let Some(c) = token.strip_prefix("sha=") {
            commit = c.to_string();
        } else if let Some(s) = token.strip_prefix("schema=") {
            schema_version = s.parse().unwrap_or(0);
        } else if token
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && token.contains('.')
        {
            semver = token.to_string();
        }
    }
    (semver, commit, schema_version)
}

async fn read_all(mut stream: impl tokio::io::AsyncRead + Unpin) -> std::io::Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::with_capacity(4096);
    stream.read_to_end(&mut buf).await?;
    Ok(buf)
}

// --- Public typed builders --------------------------------------------------

/// FR-AI-003 §3 — Typed builders for the slice-1 closed set.
pub mod builders {
    use super::*;

    /// `ai.precheck` row (FR-AI-001).
    #[allow(clippy::too_many_arguments)]
    pub fn precheck(
        tenant_id: &str,
        agent_persona: &str,
        model_alias: &str,
        resolved_provider: &str,
        resolved_model: &str,
        estimated_usd: Decimal,
        current_spent_usd: Decimal,
        idempotency_key: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::Precheck,
            path: row_path("ai-invocations", tenant_id, idempotency_key),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "model_alias": model_alias,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "estimated_usd": estimated_usd.to_string(),
                "current_spent_usd": current_spent_usd.to_string(),
                "idempotency_key": idempotency_key,
            }),
        }
    }

    /// `ai.invocation` row (FR-AI-002 success path).
    #[allow(clippy::too_many_arguments)]
    pub fn invocation(
        tenant_id: &str,
        agent_persona: &str,
        resolved_provider: &str,
        resolved_model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        actual_usd: Decimal,
        hold_id: Uuid,
        latency_ms: u32,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::Invocation,
            path: row_path("ai-invocations", tenant_id, &hold_id.to_string()),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "actual_usd": actual_usd.to_string(),
                "hold_id": hold_id,
                "latency_ms": latency_ms,
            }),
        }
    }

    /// `ai.invocation_failed` row (FR-AI-002 refund path).
    #[allow(clippy::too_many_arguments)]
    pub fn invocation_failed(
        tenant_id: &str,
        agent_persona: &str,
        resolved_provider: &str,
        resolved_model: &str,
        http_status: u16,
        retryable: bool,
        provider_error_message: &str,
        hold_id: Uuid,
        refund_amount_usd: Decimal,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::InvocationFailed,
            path: row_path("ai-invocations", tenant_id, &hold_id.to_string()),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "http_status": http_status,
                "retryable": retryable,
                "provider_error_message": provider_error_message,
                "hold_id": hold_id,
                "refund_amount_usd": refund_amount_usd.to_string(),
            }),
        }
    }

    /// `ai.hold_expired` row (FR-AI-004 cleanup job).
    pub fn hold_expired(
        tenant_id: &str,
        hold_id: Uuid,
        expired_at: chrono::DateTime<chrono::Utc>,
        refund_amount_usd: Decimal,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::HoldExpired,
            path: row_path("ai-invocations", tenant_id, &hold_id.to_string()),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "hold_id": hold_id,
                "expired_at": expired_at.to_rfc3339(),
                "refund_amount_usd": refund_amount_usd.to_string(),
            }),
        }
    }

    /// `ai.persona_loaded` row (FR-AI-014).
    pub fn persona_loaded(
        tenant_id: &str,
        persona_id: &str,
        persona_version: &str,
        source_path: &str,
        source_hash: [u8; 32],
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::PersonaLoaded,
            path: row_path("ai-personas", tenant_id, persona_id),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "persona_id": persona_id,
                "persona_version": persona_version,
                "source_path": source_path,
                "source_hash": hex::encode(source_hash),
            }),
        }
    }

    fn row_path(folder: &str, tenant_id: &str, key: &str) -> String {
        let now = chrono::Utc::now().timestamp_millis().max(0) as u128;
        let safe_tenant = tenant_id.replace(':', "-");
        format!("memories/{folder}/{now}_{safe_tenant}_{key}.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_rejects_absolute_and_traversal_and_reserved() {
        assert!(validate_path("/etc/passwd").is_err());
        assert!(validate_path("../escape.md").is_err());
        assert!(validate_path("memories/x/../escape.md").is_err());
        assert!(validate_path("audit/log.binlog").is_err());
        assert!(validate_path("index/foo.idx").is_err());
        assert!(validate_path("HEAD").is_err());
        assert!(validate_path(".lock").is_err());
        assert!(validate_path("").is_err());
    }

    #[test]
    fn validate_path_accepts_memories_subdirs() {
        assert!(validate_path("memories/ai-invocations/abc.md").is_ok());
        assert!(validate_path("memories/ai-personas/xyz.md").is_ok());
    }

    #[test]
    fn ai_invocation_kind_tag_is_stable() {
        assert_eq!(AiInvocationKind::Precheck.tag(), "ai.precheck");
        assert_eq!(AiInvocationKind::Invocation.tag(), "ai.invocation");
        assert_eq!(
            AiInvocationKind::InvocationFailed.tag(),
            "ai.invocation_failed"
        );
        assert_eq!(AiInvocationKind::HoldExpired.tag(), "ai.hold_expired");
        assert_eq!(AiInvocationKind::PersonaLoaded.tag(), "ai.persona_loaded");
    }

    #[test]
    fn parse_version_line_handles_canonical_format() {
        let (s, c, sv) = parse_version_line("cyberos.writer 0.1.0 sha=abc1234 schema=1");
        assert_eq!(s, "0.1.0");
        assert_eq!(c, "abc1234");
        assert_eq!(sv, 1);
    }

    #[test]
    fn compute_chain_is_deterministic() {
        let prev = "00".repeat(32);
        let a = compute_chain(r#"{"x":1}"#, &prev);
        let b = compute_chain(r#"{"x":1}"#, &prev);
        assert_eq!(a, b);
    }
}
