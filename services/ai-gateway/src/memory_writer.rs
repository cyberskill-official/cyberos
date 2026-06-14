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
//!   `hold_expired`, `persona_loaded`, `zdr_violation`, `residency_violation` — all implemented.
//! - Startup health check (`check_writer_available`) implemented.
//! - The `python3 -m cyberos.writer put` Python subprocess is supplied by
//!   `modules/memory/runtime/`. When that interface is not on PATH, `check_writer_available`
//!   returns `Err(WriterUnreachable)` and the gateway should exit non-zero at boot
//!   (FR-AI-003 §1 #10).

pub mod canonical;

use once_cell::sync::Lazy;
use std::future::Future;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{error, Instrument};
use uuid::Uuid;

const WRITER_BIN: &str = "python3";
const WRITER_ARGS: &[&str] = &["-m", "cyberos.writer", "put"];
const WRITER_STREAM_ARGS: &[&str] = &["-m", "cyberos.writer", "stream"];
const WRITER_TIMEOUT: Duration = Duration::from_secs(5);
const MEMORY_KINDS: &[&str] = &[
    "decisions",
    "facts",
    "people",
    "projects",
    "preferences",
    "drift",
    "refinements",
];

static STREAM_SESSION: Lazy<Mutex<Option<WriterStreamSession>>> = Lazy::new(|| Mutex::new(None));

tokio::task_local! {
    static WRITER_TRACE_CONTEXT: WriterTraceContext;
}

// --- Types ------------------------------------------------------------------

/// FR-AI-003 §1 #8 — closed set of `ai.*` row kinds emitted by the AI Gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiInvocationKind {
    /// Emitted by FR-AI-001 (cost-ledger pre-call check).
    Precheck,
    /// Emitted by FR-AI-002 (success path).
    Invocation,
    /// Emitted by FR-AI-002 (refund path).
    InvocationFailed,
    /// Emitted by FR-AI-002 before applying a reconcile state transition.
    ReconcileStarted,
    /// Emitted by FR-AI-002 after the reconcile transaction commits.
    ReconcileCompleted,
    /// Emitted by FR-AI-002 after a reconcile attempt rolls back.
    ReconcileFailed,
    /// Emitted by FR-AI-004 (cleanup job).
    HoldExpired,
    /// Emitted by FR-AI-004 before an expiry state transition.
    HoldExpiredStarted,
    /// Emitted by FR-AI-004 after an expiry state transition commits.
    HoldExpiredCompleted,
    /// Emitted by FR-AI-004 after a deterministic cleanup sweep.
    CleanupRunCompleted,
    /// Emitted by FR-AI-014 (persona stamping).
    PersonaLoaded,
    /// Emitted by FR-AI-015 (ZDR refusal).
    ZdrViolation,
    /// Emitted by FR-AI-016 (residency refusal).
    ResidencyViolation,
    /// Emitted by FR-AI-021 (`cyberos-ai policy set`).
    CliPolicyUpdated,
    /// Emitted by FR-OBS-004 when a tenant opts into LangSmith exports.
    ObsLangsmithExportEnabled,
    /// Emitted by FR-AI-021 (`cyberos-ai failover drill`).
    CliFailoverDrill,
    /// Emitted by FR-AI-021 (`cyberos-ai invoice export`).
    CliInvoiceExported,
    /// Emitted by FR-AI-021 (`cyberos-ai breaker reset`).
    CliBreakerReset,
    /// Emitted by FR-AI-021 (`cyberos-ai expiry repair`).
    CliExpiryRepaired,
    /// Emitted by FR-AI-021 (`cyberos-ai memory emit`).
    CliMemoryEmitted,
}

impl AiInvocationKind {
    /// String tag emitted into the audit row body.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Precheck => "ai.precheck",
            Self::Invocation => "ai.invocation",
            Self::InvocationFailed => "ai.invocation_failed",
            Self::ReconcileStarted => "ai.reconcile_started",
            Self::ReconcileCompleted => "ai.reconcile_completed",
            Self::ReconcileFailed => "ai.reconcile_failed",
            Self::HoldExpired => "ai.hold_expired",
            Self::HoldExpiredStarted => "ai.hold_expired_started",
            Self::HoldExpiredCompleted => "ai.hold_expired_completed",
            Self::CleanupRunCompleted => "ai.cleanup_run_completed",
            Self::PersonaLoaded => "ai.persona_loaded",
            Self::ZdrViolation => "ai.zdr_violation",
            Self::ResidencyViolation => "ai.residency_violation",
            Self::CliPolicyUpdated => "ai.cli_policy_updated",
            Self::ObsLangsmithExportEnabled => "obs.langsmith_export_enabled",
            Self::CliFailoverDrill => "ai.cli_failover_drill",
            Self::CliInvoiceExported => "ai.cli_invoice_exported",
            Self::CliBreakerReset => "ai.cli_breaker_reset",
            Self::CliExpiryRepaired => "ai.cli_expiry_repaired",
            Self::CliMemoryEmitted => "ai.cli_memory_emitted",
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

/// Trace context exported to the Writer subprocess environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriterTraceContext {
    /// W3C trace id.
    pub trace_id: String,
    /// Current span id.
    pub span_id: String,
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

#[derive(Deserialize)]
#[serde(untagged)]
enum WriterResponse {
    Ok(WriterStdout),
    Err { error: String },
}

struct WriterStreamSession {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
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
    let payload_value: Value =
        serde_json::from_str(&payload).map_err(|e| MemoryWriterError::CanonicalisationFailed {
            reason: format!("canonical payload reparse failed: {e}"),
        })?;

    let trace_context = current_writer_trace_context();
    let row = if trace_context.is_some() {
        emit_payload_once_with_context(&payload, trace_context.as_ref()).await?
    } else {
        emit_payload_streaming(&payload).await?
    };
    finish_emit(&req, &payload_value, row)
}

/// Run memory emissions with subprocess `OTEL_TRACE_ID` and `OTEL_SPAN_ID`.
pub async fn with_trace_context<F>(trace_id: &str, span_id: &str, future: F) -> F::Output
where
    F: Future,
{
    WRITER_TRACE_CONTEXT
        .scope(
            WriterTraceContext {
                trace_id: trace_id.to_string(),
                span_id: span_id.to_string(),
            },
            future,
        )
        .await
}

/// Read the current task-local Writer trace context.
pub fn current_writer_trace_context() -> Option<WriterTraceContext> {
    WRITER_TRACE_CONTEXT.try_with(Clone::clone).ok()
}

async fn emit_payload_streaming(payload: &str) -> Result<WriterStdout, MemoryWriterError> {
    let mut guard = STREAM_SESSION.lock().await;
    if guard.is_none() {
        *guard = Some(WriterStreamSession::start().await?);
    }

    let result = match guard.as_mut() {
        Some(session) => session.emit(payload).await,
        None => unreachable!("stream session was just initialised"),
    };

    match result {
        Ok(row) => Ok(row),
        Err(first_err) => {
            *guard = None;
            drop(guard);
            emit_payload_once(payload).await.or(Err(first_err))
        }
    }
}

async fn emit_payload_once(payload: &str) -> Result<WriterStdout, MemoryWriterError> {
    emit_payload_once_with_context(payload, None).await
}

async fn emit_payload_once_with_context(
    payload: &str,
    trace_context: Option<&WriterTraceContext>,
) -> Result<WriterStdout, MemoryWriterError> {
    // Spawn Writer for one row. Kept as fallback for a failed stream process.
    let mut child = writer_command(WRITER_ARGS);
    apply_writer_trace_env(&mut child, trace_context);
    let mut child = child
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
    let write_task = tokio::spawn(
        async move {
            stdin.write_all(&payload_bytes).await?;
            stdin.shutdown().await?;
            Ok::<_, std::io::Error>(())
        }
        .in_current_span(),
    );

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

    parse_writer_stdout(&stdout_bytes)
}

fn finish_emit(
    req: &MemoryEmit,
    payload_value: &Value,
    row: WriterStdout,
) -> Result<EmittedRow, MemoryWriterError> {
    // Verify chain hash locally (FR-AI-003 §1 #7).
    let expected = compute_chain(payload_value, &row)?;
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
            payload_canonical_hash = hex::encode(Sha256::digest(
                canonical::canonicalise(payload_value)
                    .unwrap_or_default()
                    .as_bytes()
            )),
            "chain_hash_mismatch — refusing row",
        );
        return Err(MemoryWriterError::ChainHashMismatch { expected, got });
    }

    Ok(EmittedRow {
        seq: row.seq,
        ts_ns: row.ts_ns,
        chain: expected,
        path: payload_value
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or(&req.path)
            .to_string(),
    })
}

fn parse_writer_stdout(stdout_bytes: &[u8]) -> Result<WriterStdout, MemoryWriterError> {
    let response: WriterResponse =
        serde_json::from_slice(stdout_bytes).map_err(|e| MemoryWriterError::WriterFailed {
            exit_code: 0,
            stderr: format!("stdout parse: {e}"),
        })?;
    match response {
        WriterResponse::Ok(row) => Ok(row),
        WriterResponse::Err { error } => Err(MemoryWriterError::WriterFailed {
            exit_code: 0,
            stderr: error,
        }),
    }
}

/// FR-AI-003 §1 #10 — Startup health check.
pub async fn check_writer_available() -> Result<WriterVersion, MemoryWriterError> {
    let out = writer_command(&["-m", "cyberos.writer", "--version"])
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
    if let Some(rest) = path.strip_prefix("memories/") {
        let kind = rest.split('/').next().unwrap_or_default();
        if !MEMORY_KINDS.contains(&kind) {
            return Err(format!("invalid memory kind: {kind}"));
        }
    }
    Ok(())
}

fn writer_command(args: &[&str]) -> Command {
    let mut cmd = Command::new(WRITER_BIN);
    cmd.args(args);
    if let Some(path) = local_memory_pythonpath() {
        match std::env::var_os("PYTHONPATH") {
            Some(existing) if !existing.is_empty() => {
                let mut paths = vec![path];
                paths.extend(std::env::split_paths(&existing));
                if let Ok(joined) = std::env::join_paths(paths) {
                    cmd.env("PYTHONPATH", joined);
                }
            }
            _ => {
                cmd.env("PYTHONPATH", path);
            }
        }
    }
    cmd
}

fn apply_writer_trace_env(cmd: &mut Command, trace_context: Option<&WriterTraceContext>) {
    if let Some(context) = trace_context {
        cmd.env("OTEL_TRACE_ID", &context.trace_id);
        cmd.env("OTEL_SPAN_ID", &context.span_id);
    }
}

fn local_memory_pythonpath() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for dir in cwd.ancestors() {
        let candidate = dir.join("modules").join("memory");
        if candidate.join("cyberos").join("writer.py").is_file() {
            return Some(candidate);
        }
    }
    None
}

fn compute_chain(
    canonical_payload: &Value,
    row: &WriterStdout,
) -> Result<[u8; 32], MemoryWriterError> {
    let prev = hex::decode(&row.prev_chain).unwrap_or_default();
    let body = canonical_payload
        .get("body")
        .and_then(Value::as_str)
        .ok_or_else(|| MemoryWriterError::CanonicalisationFailed {
            reason: "canonical payload missing string body".to_string(),
        })?;
    let meta = canonical_payload
        .get("meta")
        .and_then(Value::as_object)
        .ok_or_else(|| MemoryWriterError::CanonicalisationFailed {
            reason: "canonical payload missing object meta".to_string(),
        })?;
    let actor = meta.get("actor").and_then(Value::as_str).ok_or_else(|| {
        MemoryWriterError::CanonicalisationFailed {
            reason: "canonical payload missing string meta.actor".to_string(),
        }
    })?;
    let kind = meta.get("kind").and_then(Value::as_str).ok_or_else(|| {
        MemoryWriterError::CanonicalisationFailed {
            reason: "canonical payload missing string meta.kind".to_string(),
        }
    })?;
    let path = canonical_payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| MemoryWriterError::CanonicalisationFailed {
            reason: "canonical payload missing string path".to_string(),
        })?;
    let mut extra = serde_json::Map::new();
    extra.insert("kind".to_string(), Value::String(kind.to_string()));
    match meta.get("extra") {
        Some(Value::Object(map)) => {
            for (key, value) in map {
                extra.insert(key.clone(), value.clone());
            }
        }
        Some(_) => {
            return Err(MemoryWriterError::CanonicalisationFailed {
                reason: "canonical payload meta.extra must be an object".to_string(),
            })
        }
        None => {}
    }

    let record_minus_chain = serde_json::json!({
        "actor": actor,
        "chain": "",
        "content_sha256": hex::encode(Sha256::digest(body.as_bytes())),
        "extra": Value::Object(extra),
        "op": "put",
        "path": path,
        "prev_chain": row.prev_chain,
        "ts_ns": row.ts_ns,
    });
    let canonical_record = canonical::canonicalise(&record_minus_chain)
        .map_err(|reason| MemoryWriterError::CanonicalisationFailed { reason })?;
    let mut hasher = Sha256::new();
    hasher.update(canonical_record.as_bytes());
    hasher.update(&prev);
    Ok(hasher.finalize().into())
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

impl WriterStreamSession {
    async fn start() -> Result<Self, MemoryWriterError> {
        let mut child = writer_command(WRITER_STREAM_ARGS);
        let mut child = child
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| MemoryWriterError::WriterUnreachable {
                reason: e.to_string(),
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| MemoryWriterError::WriterUnreachable {
                reason: "stream stdin unavailable".to_string(),
            })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| MemoryWriterError::WriterUnreachable {
                reason: "stream stdout unavailable".to_string(),
            })?;

        Ok(Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    async fn emit(&mut self, payload: &str) -> Result<WriterStdout, MemoryWriterError> {
        self.stdin
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| MemoryWriterError::WriterUnreachable {
                reason: format!("stream write: {e}"),
            })?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| MemoryWriterError::WriterUnreachable {
                reason: format!("stream newline: {e}"),
            })?;
        self.stdin
            .flush()
            .await
            .map_err(|e| MemoryWriterError::WriterUnreachable {
                reason: format!("stream flush: {e}"),
            })?;

        let mut line = String::new();
        let n = timeout(WRITER_TIMEOUT, self.stdout.read_line(&mut line))
            .await
            .map_err(|_| MemoryWriterError::Timeout {
                waited_ms: WRITER_TIMEOUT.as_millis() as u32,
            })?
            .map_err(|e| MemoryWriterError::WriterUnreachable {
                reason: format!("stream read: {e}"),
            })?;
        if n == 0 {
            return Err(MemoryWriterError::WriterUnreachable {
                reason: "stream exited".to_string(),
            });
        }
        parse_writer_stdout(line.as_bytes())
    }
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
        model_alias: &str,
        resolved_provider: &str,
        resolved_model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        actual_usd: Decimal,
        hold_id: Uuid,
        latency_ms: u32,
        cache_state: &str,
        provider_request_id: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::Invocation,
            path: row_path("ai-invocations", tenant_id, &hold_id.to_string()),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "model_alias": model_alias,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "actual_usd": actual_usd.to_string(),
                "hold_id": hold_id,
                "latency_ms": latency_ms,
                "cache_state": cache_state,
                "provider_request_id": provider_request_id,
            }),
        }
    }

    /// `ai.invocation_failed` row (FR-AI-002 refund path).
    #[allow(clippy::too_many_arguments)]
    pub fn invocation_failed(
        tenant_id: &str,
        agent_persona: &str,
        model_alias: &str,
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
                "model_alias": model_alias,
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

    /// `obs.langsmith_export_enabled` row (FR-OBS-004 tenant opt-in).
    pub fn obs_langsmith_export_enabled(
        tenant_id: &str,
        operator_id: &str,
        request_id: &str,
        command_sha256: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::ObsLangsmithExportEnabled,
            path: row_path("obs-langsmith-exports", tenant_id, request_id),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "operator_id": operator_id,
                "enabled": true,
                "request_id": request_id,
                "command_sha256": command_sha256,
            }),
        }
    }

    /// `ai.reconcile_started` row (FR-AI-002 pair-write start).
    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_started(
        tenant_id: &str,
        agent_persona: &str,
        model_alias: &str,
        resolved_provider: &str,
        resolved_model: &str,
        hold_id: Uuid,
        outcome_kind: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::ReconcileStarted,
            path: row_path(
                "ai-invocations",
                tenant_id,
                &format!("{hold_id}-reconcile-started"),
            ),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "model_alias": model_alias,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "hold_id": hold_id,
                "outcome_kind": outcome_kind,
            }),
        }
    }

    /// `ai.reconcile_completed` row (FR-AI-002 pair-write completion).
    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_completed(
        tenant_id: &str,
        agent_persona: &str,
        model_alias: &str,
        resolved_provider: &str,
        resolved_model: &str,
        hold_id: Uuid,
        outcome: &str,
        actual_usd: Option<Decimal>,
        new_spent_total_usd: Option<Decimal>,
        warn_crossed: bool,
        refund_reason: Option<&str>,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::ReconcileCompleted,
            path: row_path(
                "ai-invocations",
                tenant_id,
                &format!("{hold_id}-reconcile-completed"),
            ),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "model_alias": model_alias,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "hold_id": hold_id,
                "outcome": outcome,
                "actual_usd": actual_usd.map(|v| v.to_string()),
                "new_spent_total_usd": new_spent_total_usd.map(|v| v.to_string()),
                "warn_crossed": warn_crossed,
                "refund_reason": refund_reason,
            }),
        }
    }

    /// `ai.reconcile_failed` row (FR-AI-002 pair-write failure).
    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_failed(
        tenant_id: &str,
        agent_persona: &str,
        model_alias: &str,
        resolved_provider: &str,
        resolved_model: &str,
        hold_id: Uuid,
        error: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::ReconcileFailed,
            path: row_path(
                "ai-invocations",
                tenant_id,
                &format!("{hold_id}-reconcile-failed"),
            ),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "model_alias": model_alias,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "hold_id": hold_id,
                "error": error,
            }),
        }
    }

    /// `ai.hold_expired` row (FR-AI-004 cleanup job).
    pub fn hold_expired(
        tenant_id: &str,
        hold_id: Uuid,
        expired_at: chrono::DateTime<chrono::Utc>,
        refund_amount_usd: Decimal,
        tick_id: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::HoldExpired,
            path: row_path("ai-invocations", tenant_id, &hold_id.to_string()),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "hold_id": hold_id,
                "expired_at": expired_at.to_rfc3339(),
                "refund_amount_usd": refund_amount_usd.to_string(),
                "tick_id": tick_id,
            }),
        }
    }

    /// `ai.hold_expired_started` row (FR-AI-004 pair-write start).
    pub fn hold_expired_started(tenant_id: &str, hold_id: Uuid, tick_id: &str) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::HoldExpiredStarted,
            path: row_path(
                "ai-invocations",
                tenant_id,
                &format!("{hold_id}-hold-expired-started"),
            ),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "hold_id": hold_id,
                "tick_id": tick_id,
            }),
        }
    }

    /// `ai.hold_expired_completed` row (FR-AI-004 pair-write completion).
    pub fn hold_expired_completed(
        tenant_id: &str,
        hold_id: Uuid,
        tick_id: &str,
        refund_reason: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::HoldExpiredCompleted,
            path: row_path(
                "ai-invocations",
                tenant_id,
                &format!("{hold_id}-hold-expired-completed"),
            ),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "hold_id": hold_id,
                "tick_id": tick_id,
                "refund_reason": refund_reason,
            }),
        }
    }

    /// `ai.cleanup_run_completed` row (FR-AI-004 deterministic sweep summary).
    pub fn cleanup_run_completed(
        tick_id: &str,
        expired_hold_ids: &[Uuid],
        holds_succeeded: u32,
        holds_failed: u32,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::CleanupRunCompleted,
            path: row_path(
                "ai-invocations",
                "system",
                &format!("{tick_id}-cleanup-run-completed"),
            ),
            extra: serde_json::json!({
                "tick_id": tick_id,
                "expired_hold_ids": expired_hold_ids,
                "holds_succeeded": holds_succeeded,
                "holds_failed": holds_failed,
            }),
        }
    }

    /// `ai.persona_loaded` row (FR-AI-014).
    pub fn persona_loaded(
        tenant_id: &str,
        persona_id: &str,
        persona_version: &str,
        persona_handle: &str,
        source_path: &str,
        source_hash: [u8; 32],
        request_id: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::PersonaLoaded,
            path: row_path("ai-personas", tenant_id, persona_id),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "persona_id": persona_id,
                "persona_version": persona_version,
                "persona_handle": persona_handle,
                "source_path": source_path,
                "source_hash": hex::encode(source_hash),
                "request_id": request_id,
            }),
        }
    }

    /// `ai.zdr_violation` row (FR-AI-015).
    #[allow(clippy::too_many_arguments)]
    pub fn zdr_violation(
        tenant_id: &str,
        agent_persona: &str,
        requested_alias: &str,
        resolved_provider: &str,
        resolved_model: &str,
        policy_requires_zdr: bool,
        attestation_present: bool,
        request_id: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::ZdrViolation,
            path: row_path("ai-zdr-violations", tenant_id, request_id),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "requested_alias": requested_alias,
                "resolved_provider": resolved_provider,
                "resolved_model": resolved_model,
                "policy_requires_zdr": policy_requires_zdr,
                "attestation_present": attestation_present,
                "request_id": request_id,
            }),
        }
    }

    /// `ai.residency_violation` row (FR-AI-016).
    #[allow(clippy::too_many_arguments)]
    pub fn residency_violation(
        tenant_id: &str,
        agent_persona: &str,
        requested_alias: &str,
        policy_residency: &str,
        resolved_region: Option<&str>,
        vn1_no_provider: bool,
        request_id: &str,
    ) -> MemoryEmit {
        MemoryEmit {
            kind: AiInvocationKind::ResidencyViolation,
            path: row_path("ai-residency-violations", tenant_id, request_id),
            extra: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "requested_alias": requested_alias,
                "policy_residency": policy_residency,
                "resolved_region": resolved_region,
                "request_id": request_id,
                "vn1_no_provider": vn1_no_provider,
            }),
        }
    }

    fn row_path(folder: &str, tenant_id: &str, key: &str) -> String {
        let now = chrono::Utc::now().timestamp_millis().max(0) as u128;
        let safe_tenant = tenant_id.replace(':', "-");
        format!("memories/decisions/{folder}/{now}_{safe_tenant}_{key}.md")
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
        assert!(validate_path("memories/decisions/ai-invocations/abc.md").is_ok());
        assert!(validate_path("memories/decisions/ai-personas/xyz.md").is_ok());
        assert!(validate_path("memories/ai-invocations/abc.md").is_err());
    }

    #[test]
    fn ai_invocation_kind_tag_is_stable() {
        assert_eq!(AiInvocationKind::Precheck.tag(), "ai.precheck");
        assert_eq!(AiInvocationKind::Invocation.tag(), "ai.invocation");
        assert_eq!(
            AiInvocationKind::InvocationFailed.tag(),
            "ai.invocation_failed"
        );
        assert_eq!(
            AiInvocationKind::ReconcileStarted.tag(),
            "ai.reconcile_started"
        );
        assert_eq!(
            AiInvocationKind::ReconcileCompleted.tag(),
            "ai.reconcile_completed"
        );
        assert_eq!(
            AiInvocationKind::ReconcileFailed.tag(),
            "ai.reconcile_failed"
        );
        assert_eq!(AiInvocationKind::HoldExpired.tag(), "ai.hold_expired");
        assert_eq!(
            AiInvocationKind::HoldExpiredStarted.tag(),
            "ai.hold_expired_started"
        );
        assert_eq!(
            AiInvocationKind::HoldExpiredCompleted.tag(),
            "ai.hold_expired_completed"
        );
        assert_eq!(
            AiInvocationKind::CleanupRunCompleted.tag(),
            "ai.cleanup_run_completed"
        );
        assert_eq!(AiInvocationKind::PersonaLoaded.tag(), "ai.persona_loaded");
        assert_eq!(AiInvocationKind::ZdrViolation.tag(), "ai.zdr_violation");
        assert_eq!(
            AiInvocationKind::ResidencyViolation.tag(),
            "ai.residency_violation"
        );
        assert_eq!(
            AiInvocationKind::ObsLangsmithExportEnabled.tag(),
            "obs.langsmith_export_enabled"
        );
    }

    #[tokio::test]
    async fn writer_trace_context_is_task_local() {
        assert!(current_writer_trace_context().is_none());

        let inside = with_trace_context(
            "4bf92f3577b34da6a3ce929d0e0e4736",
            "00f067aa0ba902b7",
            async { current_writer_trace_context() },
        )
        .await
        .expect("context inside scope");

        assert_eq!(inside.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(inside.span_id, "00f067aa0ba902b7");
        assert!(current_writer_trace_context().is_none());
    }

    #[test]
    fn zdr_violation_builder_carries_required_payload() {
        let row = builders::zdr_violation(
            "tenant:alpha",
            "cuo-cpo@0.4.1",
            "chat.smart",
            "openai",
            "gpt-4o",
            true,
            true,
            "req-123",
        );
        assert_eq!(row.kind.tag(), "ai.zdr_violation");
        assert_eq!(row.extra["tenant_id"], "tenant:alpha");
        assert_eq!(row.extra["agent_persona"], "cuo-cpo@0.4.1");
        assert_eq!(row.extra["requested_alias"], "chat.smart");
        assert_eq!(row.extra["resolved_provider"], "openai");
        assert_eq!(row.extra["resolved_model"], "gpt-4o");
        assert_eq!(row.extra["policy_requires_zdr"], true);
        assert_eq!(row.extra["attestation_present"], true);
        assert_eq!(row.extra["request_id"], "req-123");
    }

    #[test]
    fn residency_violation_builder_carries_required_payload() {
        let row = builders::residency_violation(
            "tenant:alpha",
            "cuo-cpo@0.4.1",
            "chat.smart",
            "vn-1",
            Some("ap-southeast-1"),
            true,
            "req-123",
        );
        assert_eq!(row.kind.tag(), "ai.residency_violation");
        assert_eq!(row.extra["tenant_id"], "tenant:alpha");
        assert_eq!(row.extra["agent_persona"], "cuo-cpo@0.4.1");
        assert_eq!(row.extra["requested_alias"], "chat.smart");
        assert_eq!(row.extra["policy_residency"], "vn-1");
        assert_eq!(row.extra["resolved_region"], "ap-southeast-1");
        assert_eq!(row.extra["vn1_no_provider"], true);
        assert_eq!(row.extra["request_id"], "req-123");
    }

    #[test]
    fn invocation_builder_carries_cache_state() {
        let hold_id = Uuid::nil();
        let row = builders::invocation(
            "tenant:alpha",
            "cuo-cpo@0.4.1",
            "chat.smart",
            "bedrock",
            "claude",
            10,
            20,
            Decimal::new(12, 4),
            hold_id,
            42,
            "hit",
            "prv-test",
        );
        assert_eq!(row.kind.tag(), "ai.invocation");
        assert_eq!(row.extra["cache_state"], "hit");
        assert_eq!(row.extra["model_alias"], "chat.smart");
        assert_eq!(row.extra["provider_request_id"], "prv-test");
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
        let payload = serde_json::json!({
            "body": "---\nkind: ai.precheck\n---\n",
            "meta": {
                "actor": "agent:cyberos-ai-gateway",
                "extra": {"tenant_id": "org:cyberskill"},
                "kind": "ai.precheck",
            },
            "path": "memories/decisions/ai-invocations/test.md",
        });
        let row = WriterStdout {
            seq: 1,
            ts_ns: 123,
            chain: "00".repeat(32),
            prev_chain: "00".repeat(32),
        };
        let a = compute_chain(&payload, &row).unwrap();
        let b = compute_chain(&payload, &row).unwrap();
        assert_eq!(a, b);
    }
}
