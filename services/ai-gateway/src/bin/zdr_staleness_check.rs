//! FR-AI-015 — CI staleness checker for ZDR attestations.

use std::path::PathBuf;
use std::process::ExitCode;

use cyberos_ai_gateway::zdr;

fn main() -> ExitCode {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/zdr_attestations.yaml"));

    let yaml = match std::fs::read_to_string(&path) {
        Ok(yaml) => yaml,
        Err(err) => {
            eprintln!("failed to read {}: {err}", path.display());
            return ExitCode::from(2);
        }
    };
    let table = match zdr::parse_attestations(&yaml) {
        Ok(table) => table,
        Err(err) => {
            eprintln!("failed to parse {}: {err}", path.display());
            return ExitCode::from(2);
        }
    };

    let mut stale = Vec::new();
    for ((provider, model), att) in &table {
        let status = if zdr::is_hard_stale(att) {
            "hard-stale"
        } else if zdr::is_soft_stale(att) {
            "soft-stale"
        } else {
            continue;
        };
        stale.push(format!(
            "{} {} {} verified_at={}",
            provider.as_metric_label(),
            model,
            status,
            att.verified_at
        ));
    }
    stale.sort();

    if stale.is_empty() {
        println!(
            "zdr-staleness-check: ok ({} attestation(s), {})",
            table.len(),
            path.display()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "zdr-staleness-check: {} stale attestation(s) in {}",
            stale.len(),
            path.display()
        );
        for line in stale {
            eprintln!("{line}");
        }
        ExitCode::from(1)
    }
}
