//! FR-AI-021 — `cyberos-ai failover` subcommand.

use std::io::{self, IsTerminal, Write};

use super::auth::{OperatorClaims, Role};
use super::{CliError, FailoverAction};

pub async fn run(
    args: FailoverAction,
    _json: bool,
    confirm: bool,
    claims: &OperatorClaims,
    _pool: &sqlx::PgPool,
) -> Result<(), CliError> {
    match args {
        FailoverAction::Drill {
            target,
            duration,
            prod_confirmed_aware,
        } => {
            super::auth::require_role(claims, &Role::Admin).map_err(|e| {
                CliError::InsufficientRole {
                    needed: e.needed(),
                    has: e.has(),
                }
            })?;
            drill(claims, &target, duration, confirm, prod_confirmed_aware).await
        }
    }
}

async fn drill(
    claims: &OperatorClaims,
    target: &str,
    duration: u32,
    confirm: bool,
    prod_confirmed_aware: bool,
) -> Result<(), CliError> {
    let tier = std::env::var("CYBEROS_DEPLOYMENT_TIER").unwrap_or_else(|_| "staging".into());

    if !confirm {
        println!("Failover drill preview:");
        println!("  target:     {target}");
        println!("  duration:   {duration}s");
        println!("  tier:       {tier}");
        eprintln!("To apply, re-run with --confirm");
        return Err(CliError::DestructiveWithoutConfirm);
    }

    if tier == "production" && !prod_confirmed_aware {
        eprintln!(
            "production drill requires --prod-confirmed-aware AND interactive Y confirmation"
        );
        return Err(CliError::DestructiveWithoutConfirm);
    }

    if tier == "production" {
        confirm_production_drill(&tier)?;
    }

    // Parse target.
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        return Err(CliError::UserError {
            reason: format!("invalid target format '{target}', expected 'provider:model'"),
        });
    }

    let command_line = super::current_command_line();
    let command_sha256 = super::command_sha256(&command_line);
    let request_id = super::request_id();

    crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::CliFailoverDrill,
        path: super::cli_audit_path("failover-drills", target),
        extra: serde_json::json!({
            "operator_id": claims.operator_id,
            "command": "failover drill",
            "args": {
                "target": target,
                "duration_s": duration,
                "prod_confirmed_aware": prod_confirmed_aware,
            },
            "target": target,
            "duration_s": duration,
            "deployment_tier": tier,
            "command_sha256": command_sha256,
            "request_id": request_id,
            "outcome": "confirmed",
        }),
    })
    .await
    .map_err(super::memory_writer_error)?;

    println!("Failover drill initiated:");
    println!("  target:     {target}");
    println!("  duration:   {duration}s");
    println!("  tier:       {tier}");
    println!("  operator:   {}", claims.operator_id);

    Ok(())
}

fn confirm_production_drill(tier: &str) -> Result<(), CliError> {
    if !io::stdin().is_terminal() {
        eprintln!("production drill requires interactive Y confirmation on a terminal");
        return Err(CliError::DestructiveWithoutConfirm);
    }

    print!("Type Y to run failover drill against deployment tier '{tier}': ");
    io::stdout().flush().map_err(|e| CliError::InternalError {
        reason: format!("flush prompt: {e}"),
    })?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|e| CliError::InternalError {
            reason: format!("read confirmation: {e}"),
        })?;
    if answer.trim() != "Y" {
        return Err(CliError::DestructiveWithoutConfirm);
    }
    Ok(())
}
