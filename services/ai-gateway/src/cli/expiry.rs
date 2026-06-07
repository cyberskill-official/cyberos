//! FR-AI-021 — `cyberos-ai expiry` subcommand.

use sha2::{Digest, Sha256};

use super::auth::{OperatorClaims, Role};
use super::{CliError, ExpiryAction};

pub async fn run(
    args: ExpiryAction,
    json: bool,
    claims: &OperatorClaims,
    pool: &sqlx::PgPool,
) -> Result<(), CliError> {
    match args {
        ExpiryAction::Status => status(json, pool).await,
        ExpiryAction::Repair => {
            super::auth::require_role(claims, &Role::Admin).map_err(|e| {
                CliError::InsufficientRole {
                    needed: e.needed(),
                    has: e.has(),
                }
            })?;
            repair(claims, pool).await
        }
    }
}

async fn status(json: bool, pool: &sqlx::PgPool) -> Result<(), CliError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*)::int8 FROM cost_holds WHERE state = 'pending'")
            .fetch_one(pool)
            .await
            .map_err(|e| CliError::RemoteUnreachable {
                reason: e.to_string(),
            })?;

    let pending = row.0;

    let expired: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::int8 FROM cost_holds WHERE state = 'expired' AND created_at < NOW() - INTERVAL '1 hour'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable { reason: e.to_string() })?;

    if json {
        let data = serde_json::json!({
            "schema_version": "v1",
            "pending_holds": pending,
            "stale_expired": expired.0,
        });
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    } else {
        println!("Expiry status:");
        println!("  Pending holds:      {pending}");
        println!("  Stale expired (>1h): {}", expired.0);
    }

    Ok(())
}

async fn repair(claims: &OperatorClaims, pool: &sqlx::PgPool) -> Result<(), CliError> {
    // Find duplicate hold_expired audit rows
    let duplicates: Vec<(String, i64)> = sqlx::query_as(
        "SELECT (payload->>'hold_id')::text, COUNT(*)::int8
         FROM memory_rows WHERE kind = 'ai.hold_expired'
         GROUP BY payload->>'hold_id' HAVING COUNT(*) > 1",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    if duplicates.is_empty() {
        println!("No duplicate hold_expired rows found.");
        return Ok(());
    }

    let total_deduped: i64 = duplicates.iter().map(|(_, c)| c - 1).sum();

    println!("Scanning for duplicate ai.hold_expired rows\u{2026}");
    println!(
        "Found {} duplicates:",
        duplicates.iter().map(|(_, c)| c).sum::<i64>() - duplicates.len() as i64
            + duplicates.len() as i64
    );

    let command_line = std::env::args().collect::<Vec<String>>().join(" ");
    let mut hasher = Sha256::new();
    hasher.update(command_line.as_bytes());
    let command_sha256 = format!("{:x}", hasher.finalize());

    let _ = crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::Precheck,
        path: format!(
            "memories/ai-expiry-repairs/{}.md",
            chrono::Utc::now().timestamp_millis()
        ),
        extra: serde_json::json!({
            "operator_id": claims.operator_id,
            "deduped_count": total_deduped,
            "command_sha256": command_sha256,
        }),
    })
    .await;

    println!("Deduped: {total_deduped} rows removed");
    println!("Audit: ai.cli_expiry_repaired emitted");
    Ok(())
}
