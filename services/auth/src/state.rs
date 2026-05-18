//! Shared application state.

use crate::rbac::RoleMatrix;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub pg: PgPool,
    /// Issuer URL — included as the `iss` claim in every JWT.
    pub jwt_issuer: String,
    /// In-memory RBAC matrix (FR-AUTH-101 §1 #9, #21). Loaded at boot;
    /// 60s refresher lands as a follow-up.
    pub role_matrix: Arc<RwLock<RoleMatrix>>,
}

impl AppState {
    pub async fn connect_from_env() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").map_err(|_| {
            sqlx::Error::Configuration("DATABASE_URL env var must be set".into())
        })?;

        let pg = PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(3))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Switch every connection to the least-privilege app role
                    // so RLS policies apply. Root-tenant ops use SET ROLE in
                    // the bootstrap path; the default for everyday queries
                    // must be cyberos_app.
                    sqlx::query("SET ROLE cyberos_app")
                        .execute(conn)
                        .await
                        .ok();
                    Ok(())
                })
            })
            .connect(&url)
            .await?;

        let jwt_issuer = std::env::var("AUTH_JWT_ISSUER")
            .unwrap_or_else(|_| "https://auth.cyberos.local".into());

        // Auto-bootstrap a signing key on first boot. In production the
        // operator runs `cyberos-auth bootstrap` separately + KMS-wraps the
        // private key. For dev + first-boot we generate inline so the
        // service can issue tokens immediately.
        Self::ensure_signing_key(&pg).await.ok();

        // Load the RBAC permission matrix. If the migration hasn't been
        // applied yet (fresh dev DB), fall back to an empty matrix — the
        // service still boots; admin endpoints will 403 until migrate.
        let role_matrix = match RoleMatrix::load_from_db(&pg).await {
            Ok(m) => {
                tracing::info!(
                    grants = m.len(),
                    version = m.version(),
                    "RBAC matrix loaded"
                );
                m
            }
            Err(e) => {
                tracing::warn!(error = %e, "RBAC matrix load failed — starting with empty matrix");
                RoleMatrix::empty()
            }
        };
        let role_matrix = Arc::new(RwLock::new(role_matrix));

        Ok(Self {
            pg,
            jwt_issuer,
            role_matrix,
        })
    }

    async fn ensure_signing_key(pg: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM auth_signing_keys WHERE status = 'active' AND expires_at > NOW()",
        )
        .fetch_optional(pg)
        .await?;
        if matches!(row, Some((n,)) if n > 0) {
            return Ok(());
        }

        tracing::info!("no active signing key found — bootstrapping a fresh RSA-2048 key");
        let key = crate::keygen::generate_rsa_2048()?;
        let kid = format!("auth-{}", chrono::Utc::now().format("%Y-%m-%d"));
        let expires = chrono::Utc::now() + chrono::Duration::days(90);
        sqlx::query(
            "INSERT INTO auth_signing_keys
                    (kid, algorithm, public_pem, private_pem, status, expires_at)
             VALUES ($1, 'RS256', $2, $3, 'active', $4)
             ON CONFLICT (kid) DO NOTHING",
        )
        .bind(&kid)
        .bind(&key.public_pem)
        .bind(&key.private_pem)
        .bind(expires)
        .execute(pg)
        .await?;
        tracing::info!(%kid, "signing key bootstrapped — 90 day TTL");
        Ok(())
    }
}
