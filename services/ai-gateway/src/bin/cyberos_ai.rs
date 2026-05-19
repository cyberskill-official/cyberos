//! `cyberos-ai` operator CLI — slice 1 surface (FR-AI-005 init test; full surface in FR-AI-021).
//!
//! Slice-1 subcommands:
//! - `policy validate <file>` — runs the schema validator on a YAML file without loading it.
//! - `policy list` — lists loaded tenant policies from the live `config/tenants/` directory.
//! - `serve` — boots the AI Gateway listener (placeholder; HTTP surface lands with FR-AI-008).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use cyberos_ai_gateway::{policy, SERVICE_BANNER};

#[derive(Debug, Parser)]
#[command(name = "cyberos-ai", version, about = "CyberOS AI Gateway operator CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Policy management.
    Policy {
        #[command(subcommand)]
        op: PolicyOp,
    },
    /// Boot the gateway HTTP listener (placeholder; FR-AI-008 lands the real router).
    Serve {
        /// Path to the tenant-policy config directory.
        #[arg(long, default_value = "services/ai-gateway/config/tenants")]
        config: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum PolicyOp {
    /// Validate a tenant-policy YAML file without loading it.
    Validate {
        /// Path to the YAML file.
        file: PathBuf,
    },
    /// List the loaded tenant policies from the supplied config dir.
    List {
        /// Config directory.
        #[arg(long, default_value = "services/ai-gateway/config/tenants")]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Policy { op } => match op {
            PolicyOp::Validate { file } => match std::fs::read_to_string(&file) {
                Ok(yaml) => match policy::validate_yaml(&yaml) {
                    Ok(p) => {
                        println!("OK tenant_id={} (cap=${})", p.tenant_id, p.ai_policy.monthly_cap_usd);
                        ExitCode::SUCCESS
                    }
                    Err(errs) => {
                        for e in errs {
                            eprintln!("ERROR {e}");
                        }
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("ERROR: read {}: {e}", file.display());
                    ExitCode::FAILURE
                }
            },
            PolicyOp::List { config } => match policy::init_loader(&config).await {
                Ok(_loader) => {
                    println!("Loaded policies from {}:", config.display());
                    // Pull the cache's sorted snapshot via load_for_tenant on each known id.
                    // For now we just emit the path; FR-AI-021 adds a richer surface.
                    for entry in std::fs::read_dir(&config).unwrap_or_else(|_| panic!("read_dir {}", config.display())) {
                        let Ok(entry) = entry else { continue };
                        if entry.path().extension().map(|e| e == "yaml").unwrap_or(false) {
                            println!("  {}", entry.path().display());
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("ERROR: init_loader: {e}");
                    ExitCode::FAILURE
                }
            },
        },
        Cmd::Serve { config } => {
            println!("{SERVICE_BANNER}");
            match policy::init_loader(&config).await {
                Ok(_loader) => {
                    println!("Policy loader initialised; HTTP listener placeholder.");
                    println!("FR-AI-008 lands the real router; this binary's serve mode currently no-ops.");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("FATAL: init_loader: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
