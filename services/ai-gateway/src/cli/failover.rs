//! FR-AI-021 — `cyberos-ai failover` subcommand.

use sha2::{Digest, Sha256};

use super::auth::OperatorClaims;
use super::{CliError, FailoverAction};

pub async fn run(
    args: FailoverAction,
    _json: bool,
    claims: &OperatorClaims,
    _pool: &sqlx::PgPool,
) -> Result<(), CliError> {
    match args {
        FailoverAction::Drill {
            target,
            duration,
            prod_confirmed_aware,
        } => drill(claims, &target, duration, prod_confirmed_aware).await,
    }
}

async fn drill(
    claims: &OperatorClaims,
    target: &str,
    duration: u32,
    prod_confirmed_aware: bool,
) -> Result<(), CliError> {
    let tier = std::env::var("CYBEROS_DEPLOYMENT_TIER").unwrap_or_else(|_| "staging".into());

    if tier == "production" && !prod_confirmed_aware {
        return Err(CliError::DestructiveWithoutConfirm);
    }

    // Parse target
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        return Err(CliError::UserError {
            reason: format!("invalid target format '{target}', expected 'provider:model'"),
        });
    }

    let command_line = std::env::args().collect::<Vec<String>>().join(" ");
    let mut hasher = Sha256::new();
    hasher.update(command_line.as_bytes());
    let command_sha256 = format!("{:x}", hasher.finalize());

    let _ = crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::Precheck,
        path: format!(
            "memories/ai-failover-drills/{}_{}.md",
            target.replace(':', "-"),
            chrono::Utc::now().timestamp_millis()
        ),
        extra: serde_json::json!({
            "operator_id": claims.operator_id,
            "target": target,
            "duration_s": duration,
            "deployment_tier": tier,
            "command_sha256": command_sha256,
        }),
    })
    .await;

    println!("Failover drill initiated:");
    println!("  target:     {target}");
    println!("  duration:   {duration}s");
    println!("  tier:       {tier}");
    println!("  operator:   {}", claims.operator_id);

    Ok(())
}
