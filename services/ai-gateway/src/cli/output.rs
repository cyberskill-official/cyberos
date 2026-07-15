//! TASK-AI-021 §1 #2 — Human-readable and JSON output formatting.

use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use serde::Serialize;

/// TASK-AI-021 §1 #8 — Versioned JSON envelope.
#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T: Serialize> {
    pub schema_version: &'static str,
    #[serde(flatten)]
    pub data: T,
}

/// Print usage data in human-readable format.
pub fn print_usage_human(
    tenant: &str,
    month: &str,
    cap: f64,
    spent: f64,
    calls: u64,
    top_models: &[(String, f64, u64)],
) {
    let pct = if cap > 0.0 {
        (spent / cap) * 100.0
    } else {
        0.0
    };
    let avg = if calls > 0 { spent / calls as f64 } else { 0.0 };

    println!("PERIOD: {month}");
    println!("TENANT: {tenant}");
    println!("CAP:    ${cap:.2}");
    println!("SPENT:  ${spent:.2}  ({pct:.1}% of cap)");
    println!("CALLS:  {calls}   (avg ${avg:.4} per call)");
    println!();

    if top_models.is_empty() {
        println!("No model data available.");
        return;
    }

    println!("Top {} models by spend:", top_models.len());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("MODEL").fg(Color::Cyan),
        Cell::new("SPEND").fg(Color::Cyan),
        Cell::new("CALLS").fg(Color::Cyan),
        Cell::new("AVG").fg(Color::Cyan),
    ]);

    for (model, spend, model_calls) in top_models {
        let model_avg = if *model_calls > 0 {
            spend / *model_calls as f64
        } else {
            0.0
        };
        table.add_row(vec![
            Cell::new(model),
            Cell::new(format!("${spend:.2}")),
            Cell::new(model_calls.to_string()),
            Cell::new(format!("${model_avg:.4}")),
        ]);
    }

    println!("{table}");
}

/// Print breaker status in human-readable format.
pub fn print_breaker_status_human(breakers: &[(String, String, String, u32, String)]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("PROVIDER").fg(Color::Cyan),
        Cell::new("MODEL").fg(Color::Cyan),
        Cell::new("STATE").fg(Color::Cyan),
        Cell::new("FAILURES").fg(Color::Cyan),
        Cell::new("NEXT_HALF_OPEN").fg(Color::Cyan),
    ]);

    for (provider, model, state, failures, next) in breakers {
        let state_cell = match state.as_str() {
            "Open" => Cell::new(state).fg(Color::Red),
            "HalfOpen" => Cell::new(state).fg(Color::Yellow),
            _ => Cell::new(state).fg(Color::Green),
        };
        table.add_row(vec![
            Cell::new(provider),
            Cell::new(model),
            state_cell,
            Cell::new(failures.to_string()),
            Cell::new(next),
        ]);
    }

    println!("{table}");
}

/// Print a list of models in human-readable format.
pub fn print_models_human(models: &[(String, String, String)]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("PROVIDER").fg(Color::Cyan),
        Cell::new("MODEL").fg(Color::Cyan),
        Cell::new("ALIAS").fg(Color::Cyan),
    ]);

    for (provider, model, alias) in models {
        table.add_row(vec![
            Cell::new(provider),
            Cell::new(model),
            Cell::new(alias),
        ]);
    }

    println!("{table}");
}

/// Print pricing in human-readable format.
pub fn print_pricing_human(pricing: &[(String, String, f64, f64)]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("PROVIDER").fg(Color::Cyan),
        Cell::new("MODEL").fg(Color::Cyan),
        Cell::new("INPUT_$/1K").fg(Color::Cyan),
        Cell::new("OUTPUT_$/1K").fg(Color::Cyan),
    ]);

    for (provider, model, input, output) in pricing {
        table.add_row(vec![
            Cell::new(provider),
            Cell::new(model),
            Cell::new(format!("${input:.4}")),
            Cell::new(format!("${output:.4}")),
        ]);
    }

    println!("{table}");
}

/// Print invoice data in human-readable format.
pub fn print_invoice_human(tenant: &str, period: &str, total_usd: f64, rows: &[InvoiceRow]) {
    println!("INVOICE for {tenant} — period {period}");
    println!("TOTAL: ${total_usd:.2}");
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("DATE").fg(Color::Cyan),
        Cell::new("MODEL").fg(Color::Cyan),
        Cell::new("CALLS").fg(Color::Cyan),
        Cell::new("COST").fg(Color::Cyan),
    ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(&row.date),
            Cell::new(&row.model),
            Cell::new(row.calls.to_string()),
            Cell::new(format!("${:.4}", row.cost_usd)),
        ]);
    }

    println!("{table}");
}

/// A single invoice line item.
#[derive(Debug, Serialize)]
pub struct InvoiceRow {
    pub date: String,
    pub model: String,
    pub calls: u64,
    pub cost_usd: f64,
}

/// Print audit trail rows in human-readable format.
pub fn print_audit_trail_human(rows: &[AuditTrailRow]) {
    if rows.is_empty() {
        println!("No audit rows found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("SEQ").fg(Color::Cyan),
        Cell::new("TIMESTAMP").fg(Color::Cyan),
        Cell::new("KIND").fg(Color::Cyan),
        Cell::new("PAYLOAD").fg(Color::Cyan),
    ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(row.seq.to_string()),
            Cell::new(&row.timestamp),
            Cell::new(&row.kind),
            Cell::new(&row.payload_brief),
        ]);
    }

    println!("{table}");
}

/// A single audit trail row.
#[derive(Debug, Serialize)]
pub struct AuditTrailRow {
    pub seq: u64,
    pub timestamp: String,
    pub kind: String,
    pub payload_brief: String,
}

/// Output JSON or human-readable.
pub fn emit_output<T: Serialize>(json: bool, data: &T, human_fn: impl FnOnce(&T)) {
    if json {
        let envelope = JsonEnvelope {
            schema_version: "v1",
            data,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).expect("json serialisation")
        );
    } else {
        human_fn(data);
    }
}
