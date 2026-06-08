//! FR-AI-021 — `cyberos-ai invoice` subcommand.

use super::auth::OperatorClaims;
use super::output::{self, InvoiceRow};
use super::{CliError, InvoiceAction};
use sqlx::PgPool;

pub async fn run(
    args: InvoiceAction,
    json: bool,
    claims: &OperatorClaims,
    pool: &PgPool,
) -> Result<(), CliError> {
    match args {
        InvoiceAction::Export {
            tenant,
            period,
            format,
        } => export(pool, claims, &tenant, &period, &format, json).await,
    }
}

async fn export(
    pool: &PgPool,
    claims: &OperatorClaims,
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

    let command_line = super::current_command_line();
    let command_sha256 = super::command_sha256(&command_line);
    let request_id = super::request_id();

    crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::CliInvoiceExported,
        path: super::cli_audit_path("invoice-exports", tenant),
        extra: serde_json::json!({
            "operator_id": claims.operator_id,
            "command": "invoice export",
            "args": {
                "tenant": tenant,
                "period": period,
                "format": format,
            },
            "tenant": tenant,
            "period": period,
            "format": format,
            "row_count": invoice_rows.len(),
            "total_usd": total,
            "command_sha256": command_sha256,
            "request_id": request_id,
            "outcome": "exported",
        }),
    })
    .await
    .map_err(super::memory_writer_error)?;

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
