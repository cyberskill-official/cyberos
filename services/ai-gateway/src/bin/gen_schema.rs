//! `gen-schema` — emit the `TenantPolicy` JSONSchema for CI gate (FR-AI-005 §5).
//!
//! Run via:
//! ```bash
//! cargo run -p cyberos-ai-gateway --bin gen-schema -- --out services/ai-gateway/config/tenants/SCHEMA.json
//! ```
//!
//! CI then runs `git diff --exit-code services/ai-gateway/config/tenants/SCHEMA.json`
//! to catch silent schema drift.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cyberos_ai_gateway::policy::TenantPolicy;

#[derive(Debug, Parser)]
#[command(name = "gen-schema", about = "Emit TenantPolicy JSONSchema")]
struct Cli {
    /// Output path.
    #[arg(long, default_value = "services/ai-gateway/config/tenants/SCHEMA.json")]
    out: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let schema = schemars::schema_for!(TenantPolicy);
    let json = match serde_json::to_string_pretty(&schema) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("ERROR: serialise: {e}");
            return ExitCode::FAILURE;
        }
    };
    match std::fs::write(&cli.out, format!("{json}\n")) {
        Ok(()) => {
            println!("wrote {} ({} bytes)", cli.out.display(), json.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ERROR: write {}: {e}", cli.out.display());
            ExitCode::FAILURE
        }
    }
}
