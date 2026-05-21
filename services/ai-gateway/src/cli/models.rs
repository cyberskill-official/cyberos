//! FR-AI-021 — `cyberos-ai models` subcommand.

use super::{CliError, ModelsAction};
use super::auth::OperatorClaims;
use super::output;
use sqlx::PgPool;

pub async fn run(
    args: ModelsAction,
    json: bool,
    _claims: &OperatorClaims,
    pool: &PgPool,
) -> Result<(), CliError> {
    match args {
        ModelsAction::List => list_models(json, pool).await,
        ModelsAction::Pricing => show_pricing(json, pool).await,
    }
}

async fn list_models(json: bool, pool: &PgPool) -> Result<(), CliError> {
    let models: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT DISTINCT resolved_provider, resolved_model, model_alias
         FROM ai_invocations ORDER BY resolved_provider, resolved_model",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable { reason: e.to_string() })?;

    if json {
        let data = serde_json::json!({
            "schema_version": "v1",
            "models": models.iter().map(|(p, m, a)| {
                serde_json::json!({"provider": p, "model": m, "alias": a})
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    } else {
        output::print_models_human(&models);
    }

    Ok(())
}

async fn show_pricing(json: bool, pool: &PgPool) -> Result<(), CliError> {
    let pricing: Vec<(String, String, f64, f64)> = sqlx::query_as(
        "SELECT provider, model, input_cost_per_1k, output_cost_per_1k
         FROM cost_table ORDER BY provider, model",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable { reason: e.to_string() })?;

    if json {
        let data = serde_json::json!({
            "schema_version": "v1",
            "pricing": pricing.iter().map(|(p, m, i, o)| {
                serde_json::json!({"provider": p, "model": m, "input_per_1k": i, "output_per_1k": o})
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    } else {
        output::print_pricing_human(&pricing);
    }

    Ok(())
}
