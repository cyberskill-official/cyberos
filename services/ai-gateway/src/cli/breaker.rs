//! FR-AI-021 — `cyberos-ai breaker` subcommand.

use super::auth::{OperatorClaims, Role};
use super::output;
use super::{BreakerAction, CliError};

pub async fn run(
    args: BreakerAction,
    json: bool,
    confirm: bool,
    claims: &OperatorClaims,
    pool: &sqlx::PgPool,
) -> Result<(), CliError> {
    match args {
        BreakerAction::Status => status(json, pool).await,
        BreakerAction::Reset { target } => {
            super::auth::require_role(claims, &Role::Mutate).map_err(|e| {
                CliError::InsufficientRole {
                    needed: e.needed(),
                    has: e.has(),
                }
            })?;
            reset(claims, &target, confirm, pool).await
        }
    }
}

async fn status(json: bool, pool: &sqlx::PgPool) -> Result<(), CliError> {
    let breakers: Vec<(String, String, String, i32, Option<String>)> = sqlx::query_as(
        "SELECT provider, model, state, failure_count, next_half_open::text
         FROM circuit_breakers ORDER BY provider, model",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    let breaker_data: Vec<(String, String, String, u32, String)> = breakers
        .into_iter()
        .map(|(p, m, s, f, n)| (p, m, s, f as u32, n.unwrap_or_default()))
        .collect();

    if json {
        let data = serde_json::json!({
            "schema_version": "v1",
            "breakers": breaker_data.iter().map(|(p, m, s, f, n)| {
                serde_json::json!({"provider": p, "model": m, "state": s, "failures": f, "next_half_open": n})
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    } else {
        output::print_breaker_status_human(&breaker_data);
    }

    Ok(())
}

async fn reset(
    claims: &OperatorClaims,
    target: &str,
    confirm: bool,
    pool: &sqlx::PgPool,
) -> Result<(), CliError> {
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        return Err(CliError::UserError {
            reason: format!("invalid target format '{target}', expected 'provider:model'"),
        });
    }

    if !confirm {
        println!("Breaker reset preview:");
        println!("  target: {target}");
        println!("  after:  Closed");
        eprintln!("To apply, re-run with --confirm");
        return Err(CliError::DestructiveWithoutConfirm);
    }

    let command_line = super::current_command_line();
    let command_sha256 = super::command_sha256(&command_line);
    let request_id = super::request_id();

    crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::CliBreakerReset,
        path: super::cli_audit_path("breaker-resets", target),
        extra: serde_json::json!({
            "operator_id": claims.operator_id,
            "command": "breaker reset",
            "args": {"target": target},
            "target": target,
            "command_sha256": command_sha256,
            "request_id": request_id,
            "outcome": "confirmed",
        }),
    })
    .await
    .map_err(super::memory_writer_error)?;

    // Force-close the breaker
    sqlx::query(
        "UPDATE circuit_breakers SET state = 'Closed', failure_count = 0, last_failure_at = NULL
         WHERE provider = $1 AND model = $2",
    )
    .bind(parts[0])
    .bind(parts[1])
    .execute(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    println!("Breaker reset: {target} \u{2192} Closed");
    Ok(())
}
