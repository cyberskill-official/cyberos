//! FR-AI-021 — `cyberos-ai invoice` subcommand.

use super::{CliError, InvoiceAction};
use super::auth::OperatorClaims;
use super::output::{self, InvoiceRow};
use sqlx::PgPool;

pub async fn run(
    args: InvoiceAction,
    json: bool,
    _claims: &OperatorClaims,
    pool: &PgPool,
) -> Result<(), CliError> {
    match args {
        InvoiceAction::Export { tenant, period, format } => {
            export(pool, &tenant, &period, &format, json).await
        }
    }
}

async fn export(
    pool: &PgPool,
    tenant: &str,
    period: &str,
    format: &str,
    json_output: bool,
) -> Result<(), CliError> {
    let rows: Vec<(String, String, i64, f64)> = sqlx::query_as(
        "SELECT TO_CHAR(created_at, 'YYYY-MM-DD') as date, resolved_model, COUNT(*)::int8, SUM(actual_usd)::float8
         FROM ai_invocations
         WHERE tenant_id = $1 AND TO_CHAR(created_at, 'YYYY-MM') = $2
         GROUP BY date, resolved_model ORDER BY date, resolved_model",
    )
    .bind(tenant)
    .bind(period)
    .fetch_all(pool)
    .await
    .map_err(|e| CliError::RemoteUnreachable { reason: e.to_string() })?;

    let total: f64 = rows.iter().map(|(_, _, _, cost)| cost).sum();

    let invoice_rows: Vec<InvoiceRow> = rows
        .into_iter()
        .map(|(date, model, calls, cost)| InvoiceRow {
            date,
            model,
            calls: calls as u64,
            cost_usd: cost,
        })
        .collect();

    match format {
        "json" => {
            let data = serde_json::json!({
                "schema_version": "v1",
                "tenant": tenant,
                "period": period,
                "total_usd": total,
                "rows": invoice_rows,
            });
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
        "csv" => {
            println!("date,model,calls,cost_usd");
            for row in &invoice_rows {
                println!("{},{},{},{}", row.date, row.model, row.calls, row.cost_usd);
            }
        }
        "pdf" => {
            if json_output {
                let data = serde_json::json!({
                    "schema_version": "v1",
                    "tenant": tenant,
                    "period": period,
                    "total_usd": total,
                    "format": "pdf",
                    "rows": invoice_rows,
                });
                println!("{}", serde_json::to_string_pretty(&data).unwrap());
            } else {
                output::print_invoice_human(tenant, period, total, &invoice_rows);
                eprintln!("PDF generation requires wkhtmltopdf. Use --format json or csv for machine output.");
            }
        }
        _ => {
            return Err(CliError::UserError {
                reason: format!("unsupported format: {format} (use json, csv, or pdf)"),
            });
        }
    }

    Ok(())
}
