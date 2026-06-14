//! `cyberos-obs-compliance-view` binary.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use cyberos_obs_compliance_view::auth::AuthConfig;
use cyberos_obs_compliance_view::chain_proof::ChainProofSigner;
use cyberos_obs_compliance_view::memory::JsonlBackend;
use cyberos_obs_compliance_view::{app, AppState, SERVICE_BANNER};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "cyberos_obs_compliance_view=info,tower_http=info".to_string()),
        )
        .init();

    let auth = AuthConfig {
        hs256_secret: std::env::var("OBS_COMPLIANCE_HS256_SECRET").ok(),
        rs256_public_pem: match std::env::var("OBS_COMPLIANCE_RS256_PUBLIC_PEM_FILE") {
            Ok(path) => {
                Some(std::fs::read_to_string(&path).with_context(|| format!("read {path}"))?)
            }
            Err(_) => None,
        },
        issuer: std::env::var("OBS_COMPLIANCE_JWT_ISSUER").ok(),
        audience: std::env::var("OBS_COMPLIANCE_JWT_AUDIENCE").ok(),
    };
    let auth = if auth.hs256_secret.is_none() && auth.rs256_public_pem.is_none() {
        AuthConfig::local("cyberos-local-dev")
    } else {
        auth
    };

    let seed = std::env::var("OBS_COMPLIANCE_ED25519_SEED_HEX").unwrap_or_else(|_| {
        "0707070707070707070707070707070707070707070707070707070707070707".to_string()
    });
    let signer = ChainProofSigner::from_hex_seed(&seed)?;
    let rows_path = std::env::var("OBS_COMPLIANCE_ROWS_JSONL")
        .unwrap_or_else(|_| "target/cuo-workflow/obs-compliance-view/rows.jsonl".to_string());
    let audit_path = std::env::var("OBS_COMPLIANCE_AUDIT_JSONL")
        .unwrap_or_else(|_| "target/cuo-workflow/obs-compliance-view/audit.jsonl".to_string());
    let memory = Arc::new(JsonlBackend::new(
        Path::new(&rows_path),
        Path::new(&audit_path),
    ));
    let state = AppState::new(auth, memory, signer);

    let addr: SocketAddr = std::env::var("OBS_COMPLIANCE_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7788".to_string())
        .parse()?;
    tracing::info!(%addr, "{SERVICE_BANNER}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app(state)).await?;
    Ok(())
}
