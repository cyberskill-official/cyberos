//! `cyberos-obs` — supervisor binary around the upstream `otelcol-contrib`.
//!
//! Slice-1 surface:
//! - `validate-config <path>` — parse + validate the collector YAML against
//!   FR-OBS-001 §3 (CI gate).
//! - `validate-tokens <path>` — parse + validate the bearer-token file.
//!
//! The actual otelcol process supervision (spawn, health-check polling on
//! `:13133`, SIGHUP for token rotation, log forwarding) lands when the deploy
//! pipeline is wired in next session. The Cargo bin's slice-1 job is the
//! pre-flight validation that catches misconfiguration at deploy time.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use cyberos_obs_collector::{auth, config, ingress, SERVICE_BANNER};

#[derive(Debug, Parser)]
#[command(
    name = "cyberos-obs",
    version,
    about = "CyberOS observability supervisor"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Validate an `otel-collector-config.yaml` against the FR-OBS-001 §3 contract.
    ValidateConfig {
        /// Path to the collector config.
        path: PathBuf,
    },
    /// Parse + validate a bearer-token file.
    ValidateTokens {
        /// Path to the token file.
        path: PathBuf,
    },
    /// Print the banner and exit (smoke test for the binary itself).
    Banner,
    /// Run the CyberOS service-token OTLP ingress gate.
    Ingress {
        /// Public HTTP listen address for OTLP/HTTP and `/ready`.
        #[arg(long, default_value = "0.0.0.0:4318")]
        http_listen: SocketAddr,
        /// Public gRPC listen address for OTLP/gRPC.
        #[arg(long, default_value = "0.0.0.0:4317")]
        grpc_listen: SocketAddr,
        /// CyberOS token map: `<service-name> <bearer-token>`.
        #[arg(long)]
        tokens: PathBuf,
        /// Internal single-token file used between this gate and otelcol.
        #[arg(long)]
        collector_token: PathBuf,
        /// Upstream collector OTLP/HTTP base URL.
        #[arg(long, default_value = "http://collector:4318")]
        upstream_http: String,
        /// Upstream collector OTLP/gRPC endpoint.
        #[arg(long, default_value = "http://collector:4317")]
        upstream_grpc: String,
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
        Cmd::ValidateConfig { path } => match config::validate(&path) {
            Ok(()) => {
                println!("OK config valid: {}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("ERROR: {e}");
                ExitCode::FAILURE
            }
        },
        Cmd::ValidateTokens { path } => match auth::TokenFile::load(&path) {
            Ok(tf) => {
                println!(
                    "OK tokens loaded: {} entries from {}",
                    tf.tokens.len(),
                    path.display()
                );
                let mut services: Vec<_> = tf.tokens.keys().collect();
                services.sort();
                for s in services {
                    println!("  {s}");
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("ERROR: {e}");
                ExitCode::FAILURE
            }
        },
        Cmd::Banner => {
            println!("{SERVICE_BANNER}");
            ExitCode::SUCCESS
        }
        Cmd::Ingress {
            http_listen,
            grpc_listen,
            tokens,
            collector_token,
            upstream_http,
            upstream_grpc,
        } => {
            let cfg = ingress::IngressConfig {
                http_listen,
                grpc_listen,
                token_file: tokens,
                collector_token_file: collector_token,
                upstream_http,
                upstream_grpc,
            };
            match ingress::serve(cfg).await {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("ERROR: {e:#}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
