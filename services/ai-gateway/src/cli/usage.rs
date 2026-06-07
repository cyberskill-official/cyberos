//! FR-AI-021 — `cyberos-ai usage` subcommand.

use super::auth::OperatorClaims;
use super::output;
use super::{CliError, UsageArgs};
use sqlx::PgPool;

#[derive(serde::Serialize)]
struct UsageOutput {
    schema_version: &'static str,
    tenant: String,
    month: String,
    cap_usd: f64,
    spent_usd: f64,
    spent_pct: f64,
    calls: u64,
    top_models_by_spend: Vec<ModelSpend>,
}

#[derive(serde::Serialize)]
struct ModelSpend {
    model: String,
    spend_usd: f64,
    calls: u64,
}

impl std::fmt::Display for UsageOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PERIOD: {}", self.month)?;
        writeln!(f, "TENANT: {}", self.tenant)?;
        writeln!(f, "CAP:    ${:.2}", self.cap_usd)?;
        writeln!(
            f,
            "SPENT:  ${:.2}  ({:.1}% of cap)",
            self.spent_usd, self.spent_pct
        )?;
        writeln!(
            f,
            "CALLS:  {}   (avg ${:.4} per call)",
            self.calls,
            if self.calls > 0 {
                self.spent_usd / self.calls as f64
            } else {
                0.0
            }
        )
    }
}

pub async fn run(
    args: UsageArgs,
    json: bool,
    _claims: &OperatorClaims,
    pool: &PgPool,
) -> Result<(), CliError> {
    let tenant = args.tenant.as_deref().unwrap_or("all");
    let default_month = chrono::Utc::now().format("%Y-%m").to_string();
    let month = args.month.as_deref().unwrap_or(&default_month);

    let (cap, spent, calls, top_models) = if tenant == "all" {
        query_all_usage(pool, month).await?
    } else {
        query_tenant_usage(pool, tenant, month).await?
    };

    let spent_pct = if cap > 0.0 {
        (spent / cap) * 100.0
    } else {
        0.0
    };

    let data = UsageOutput {
        schema_version: "v1",
        tenant: tenant.to_string(),
        month: month.to_string(),
        cap_usd: cap,
        spent_usd: spent,
        spent_pct,
        calls,
        top_models_by_spend: top_models
            .into_iter()
            .map(|(model, spend, model_calls)| ModelSpend {
                model,
                spend_usd: spend,
                calls: model_calls,
            })
            .collect(),
    };

    output::emit_output(json, &data, |d| {
        output::print_usage_human(
            &d.tenant,
            &d.month,
            d.cap_usd,
            d.spent_usd,
            d.calls,
            &d.top_models_by_spend
                .iter()
                .map(|m| (m.model.clone(), m.spend_usd, m.calls))
                .collect::<Vec<_>>(),
        );
    });

    Ok(())
}

async fn query_all_usage(
    pool: &PgPool,
    month: &str,
) -> Result<(f64, f64, u64, Vec<(String, f64, u64)>), CliError> {
    let row: (Option<f64>, Option<f64>, Option<i64>) = sqlx::query_as(
        "SELECT SUM(ai_policy->>'monthly_cap_usd')::float8, SUM(actual_usd)::float8, COUNT(*)::int8
         FROM ai_invocations inv
         LEFT JOIN tenant_policies tp ON inv.tenant_id = tp.tenant_id
         WHERE TO_CHAR(inv.created_at, 'YYYY-MM') = $1",
    )
    .bind(month)
    .fetch_optional(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?
    .unwrap_or((None, None, None));

    let cap = row.0.unwrap_or(0.0);
    let spent = row.1.unwrap_or(0.0);
    let calls = row.2.unwrap_or(0) as u64;

    let models: Vec<(String, f64, i64)> = sqlx::query_as(
        "SELECT resolved_model, SUM(actual_usd)::float8, COUNT(*)::int8
         FROM ai_invocations
         WHERE TO_CHAR(created_at, 'YYYY-MM') = $1
         GROUP BY resolved_model ORDER BY SUM(actual_usd) DESC LIMIT 5",
    )
    .bind(month)
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    let top_models = models
        .into_iter()
        .map(|(m, s, c)| (m, s, c as u64))
        .collect();

    Ok((cap, spent, calls, top_models))
}

async fn query_tenant_usage(
    pool: &PgPool,
    tenant: &str,
    month: &str,
) -> Result<(f64, f64, u64, Vec<(String, f64, u64)>), CliError> {
    let row: (Option<f64>, Option<f64>, Option<i64>) = sqlx::query_as(
        "SELECT (ai_policy->>'monthly_cap_usd')::float8, SUM(actual_usd)::float8, COUNT(*)::int8
         FROM ai_invocations inv
         JOIN tenant_policies tp ON inv.tenant_id = tp.tenant_id
         WHERE inv.tenant_id = $1 AND TO_CHAR(inv.created_at, 'YYYY-MM') = $2",
    )
    .bind(tenant)
    .bind(month)
    .fetch_optional(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?
    .unwrap_or((None, None, None));

    let cap = row.0.unwrap_or(0.0);
    let spent = row.1.unwrap_or(0.0);
    let calls = row.2.unwrap_or(0) as u64;

    let models: Vec<(String, f64, i64)> = sqlx::query_as(
        "SELECT resolved_model, SUM(actual_usd)::float8, COUNT(*)::int8
         FROM ai_invocations
         WHERE tenant_id = $1 AND TO_CHAR(created_at, 'YYYY-MM') = $2
         GROUP BY resolved_model ORDER BY SUM(actual_usd) DESC LIMIT 5",
    )
    .bind(tenant)
    .bind(month)
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    let top_models = models
        .into_iter()
        .map(|(m, s, c)| (m, s, c as u64))
        .collect();

    Ok((cap, spent, calls, top_models))
}
