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

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use cyberos_obs_collector::{auth, config, grafana_proxy, ingress, SERVICE_BANNER};

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
    /// Run the tenant-aware Grafana query proxy.
    GrafanaProxy {
        /// Public HTTP listen address for Grafana datasource queries.
        #[arg(long, default_value = "0.0.0.0:8088")]
        listen: SocketAddr,
        /// Upstream Prometheus base URL.
        #[arg(long, default_value = "http://prometheus:9090")]
        prometheus_url: String,
        /// Upstream Loki base URL.
        #[arg(long, default_value = "http://loki:3100")]
        loki_url: String,
        /// Upstream Tempo base URL.
        #[arg(long, default_value = "http://tempo:3200")]
        tempo_url: String,
        /// HS256 secret file for local/dev tokens. Mutually exclusive with `--jwt-rs256-public-pem`.
        #[arg(long)]
        jwt_hs256_secret_file: Option<PathBuf>,
        /// RS256 public PEM file exported from AUTH/JWKS for production tokens.
        #[arg(long)]
        jwt_rs256_public_pem: Option<PathBuf>,
        /// JWKS JSON file exported from AUTH at boot.
        #[arg(long)]
        jwt_jwks_file: Option<PathBuf>,
        /// JWKS URL fetched from AUTH at boot and cached in memory.
        #[arg(long)]
        jwt_jwks_url: Option<String>,
        /// Optional issuer to enforce.
        #[arg(long)]
        jwt_issuer: Option<String>,
        /// Optional audience to enforce.
        #[arg(long)]
        jwt_audience: Option<String>,
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
        Cmd::GrafanaProxy {
            listen,
            prometheus_url,
            loki_url,
            tempo_url,
            jwt_hs256_secret_file,
            jwt_rs256_public_pem,
            jwt_jwks_file,
            jwt_jwks_url,
            jwt_issuer,
            jwt_audience,
        } => {
            let verifier = match build_jwt_verifier(
                jwt_hs256_secret_file,
                jwt_rs256_public_pem,
                jwt_jwks_file,
                jwt_jwks_url,
                jwt_issuer,
                jwt_audience,
            )
            .await
            {
                Ok(verifier) => verifier,
                Err(e) => {
                    eprintln!("ERROR: {e:#}");
                    return ExitCode::FAILURE;
                }
            };
            let cfg = grafana_proxy::GrafanaProxyConfig {
                listen,
                prometheus_url,
                loki_url,
                tempo_url,
                verifier,
            };
            match grafana_proxy::serve(cfg).await {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("ERROR: {e:#}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

async fn build_jwt_verifier(
    hs256_secret_file: Option<PathBuf>,
    rs256_public_pem: Option<PathBuf>,
    jwks_file: Option<PathBuf>,
    jwks_url: Option<String>,
    issuer: Option<String>,
    audience: Option<String>,
) -> anyhow::Result<grafana_proxy::JwtVerifier> {
    let selected = [
        hs256_secret_file.is_some(),
        rs256_public_pem.is_some(),
        jwks_file.is_some(),
        jwks_url.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if selected != 1 {
        anyhow::bail!(
            "provide exactly one of --jwt-hs256-secret-file, --jwt-rs256-public-pem, --jwt-jwks-file, or --jwt-jwks-url"
        );
    }

    let mut verifier = if let Some(secret_file) = hs256_secret_file {
        let secret = fs::read_to_string(&secret_file)?;
        let secret = secret.trim();
        if secret.is_empty() {
            anyhow::bail!("{} is empty", secret_file.display());
        }
        grafana_proxy::JwtVerifier::hs256(secret.to_string())
    } else if let Some(public_pem) = rs256_public_pem {
        let pem = fs::read_to_string(&public_pem)?;
        grafana_proxy::JwtVerifier::rs256_public_pem(pem)
    } else if let Some(jwks_file) = jwks_file {
        let jwks_json = fs::read_to_string(&jwks_file)?;
        grafana_proxy::JwtVerifier::rs256_jwks_json(&jwks_json)?
    } else if let Some(jwks_url) = jwks_url {
        let jwks_json = reqwest::get(&jwks_url)
            .await?
            .error_for_status()?
            .text()
            .await?;
        grafana_proxy::JwtVerifier::rs256_jwks_json(&jwks_json)?
    } else {
        unreachable!("selected count checked above")
    };
    if let Some(issuer) = issuer {
        verifier = verifier.with_issuer(issuer);
    }
    if let Some(audience) = audience {
        verifier = verifier.with_audience(audience);
    }
    Ok(verifier)
}
