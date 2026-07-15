//! TASK-AI-021 — `cyberos-ai memory` subcommand.

use sha2::{Digest, Sha256};

use super::auth::{OperatorClaims, Role};
use super::output;
use super::{CliError, MemoryAction};
use sqlx::PgPool;

pub async fn run(
    args: MemoryAction,
    json: bool,
    claims: &OperatorClaims,
    pool: &PgPool,
) -> Result<(), CliError> {
    match args {
        MemoryAction::Emit { yaml_file, dry_run } => {
            super::auth::require_role(claims, &Role::Admin).map_err(|e| {
                CliError::InsufficientRole {
                    needed: e.needed(),
                    has: e.has(),
                }
            })?;
            emit(&yaml_file, dry_run).await
        }
        MemoryAction::AuditTrail { tenant, since } => {
            audit_trail(json, pool, &tenant, &since).await
        }
    }
}

async fn emit(yaml_file: &std::path::Path, dry_run: bool) -> Result<(), CliError> {
    let yaml = std::fs::read_to_string(yaml_file).map_err(|e| CliError::UserError {
        reason: format!("read {}: {e}", yaml_file.display()),
    })?;

    let payload: serde_json::Value =
        serde_yaml::from_str(&yaml).map_err(|e| CliError::SchemaViolation {
            reason: e.to_string(),
        })?;

    let kind = payload
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    if dry_run {
        println!("DRY RUN — would emit:");
        println!("  kind:   {kind}");
        println!("  payload: {payload}");
        return Ok(());
    }

    let command_line = std::env::args().collect::<Vec<String>>().join(" ");
    let mut hasher = Sha256::new();
    hasher.update(command_line.as_bytes());
    let command_sha256 = format!("{:x}", hasher.finalize());

    let _ = crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::Precheck,
        path: format!(
            "memories/ai-manual-emits/{}_{}.md",
            kind,
            chrono::Utc::now().timestamp_millis()
        ),
        extra: serde_json::json!({
            "operator_id": "cli",
            "emitted_kind": kind,
            "command_sha256": command_sha256,
        }),
    })
    .await;

    println!("Memory row emitted: {kind}");
    Ok(())
}

async fn audit_trail(json: bool, pool: &PgPool, tenant: &str, since: &str) -> Result<(), CliError> {
    let rows: Vec<(i64, String, String, serde_json::Value)> = sqlx::query_as(
        "SELECT seq, ts::text, kind, payload
         FROM memory_rows
         WHERE (payload->>'tenant_id' = $1 OR $1 = 'all')
           AND ts >= $2::timestamptz
         ORDER BY seq DESC LIMIT 100",
    )
    .bind(tenant)
    .bind(since)
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    let audit_rows: Vec<output::AuditTrailRow> = rows
        .into_iter()
        .map(|(seq, ts, kind, payload)| {
            let brief = payload
                .as_object()
                .map(|o| {
                    let mut parts: Vec<String> =
                        o.iter().take(3).map(|(k, v)| format!("{k}={v}")).collect();
                    if o.len() > 3 {
                        parts.push("...".into());
                    }
                    parts.join(", ")
                })
                .unwrap_or_default();
            output::AuditTrailRow {
                seq: seq as u64,
                timestamp: ts,
                kind,
                payload_brief: brief,
            }
        })
        .collect();

    output::emit_output(json, &audit_rows, |rows| {
        output::print_audit_trail_human(rows);
    });

    Ok(())
}
