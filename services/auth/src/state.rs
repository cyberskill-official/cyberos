//! Shared application state.

use crate::geoip::{self, GeoIpResolver};
use crate::oidc::PendingState;
use crate::rate_limit::RateLimiter;
use crate::rbac::RoleMatrix;
use crate::travel_policy::{PolicyCache, StickySuppress};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub pg: PgPool,
    /// Issuer URL — included as the `iss` claim in every JWT.
    pub jwt_issuer: String,
    /// In-memory RBAC matrix (FR-AUTH-101 §1 #9, #21). Loaded at boot;
    /// 60s refresher swaps via `Arc<RwLock>`.
    pub role_matrix: Arc<RwLock<RoleMatrix>>,
    /// FR-AUTH-104 OIDC PKCE pending-state map (state_token → verifier).
    /// 10-minute TTL enforced at callback time; sweeper deferred to slice 2.
    pub oidc_pending: Arc<RwLock<HashMap<String, PendingState>>>,
    /// FR-AUTH-106 slice-2 — GeoIP resolver. Either MaxMindResolver (when
    /// `AUTH_GEOIP_DB` is set + readable) or NullResolver (degradation matches
    /// slice-1). The resolver is consulted on every `record_login_and_assess`
    /// call; it must be cheap to invoke and Send + Sync (the MaxMind reader is).
    pub geoip: Arc<dyn GeoIpResolver>,
    /// FR-AUTH-106 slice-3 — per-tenant policy cache (60s TTL).
    pub travel_policy: PolicyCache,
    /// FR-AUTH-106 slice-3 — sticky-challenge suppression LRU. Shared across
    /// all login flows so a passed MFA from one flow suppresses re-challenge
    /// in another flow (within the configured window).
    pub sticky_suppress: Arc<StickySuppress>,
    /// FR-AUTH-004 §1 #5 — dual rate-limiter for `POST /v1/auth/token`
    /// (per-IP 10/min + per-account 5/min). In-memory; multi-instance prod
    /// will swap to Redis when FR-OBS-002 ships.
    pub rate_limit: Arc<RateLimiter>,
    /// FR-AUTH-005 §1 #3 + #11 + G-011 — JWT jti deny-list consulted by
    /// `verify_jwt`. Populated by revoke handler with the subject's active
    /// jtis. In-memory per DEC-DENY-LIST-001 slice-1 (Redis lift is FR-AUTH-110).
    pub deny_list: crate::deny_list::DenyList,
}

impl AppState {
    pub async fn connect_from_env() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| sqlx::Error::Configuration("DATABASE_URL env var must be set".into()))?;

        let pg = PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(3))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Switch every connection to the least-privilege app role
                    // so RLS policies apply. Root-tenant ops use SET ROLE in
                    // the bootstrap path; the default for everyday queries
                    // must be cyberos_app.
                    sqlx::query("SET ROLE cyberos_app").execute(conn).await.ok();
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

        // FR-AUTH-106 slice-2 — load GeoIP resolver. Honours AUTH_GEOIP_DB
        // and AUTH_GEOIP_REQUIRED. Failure to read the DB is sticky when
        // _REQUIRED=1; otherwise the service falls back to NullResolver and
        // logs once at startup so ops sees the kind-2/3 detectors are inactive.
        let geoip = match geoip::from_env() {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "GeoIP init failed — refusing to start");
                return Err(sqlx::Error::Configuration(e.to_string().into()));
            }
        };

        Ok(Self {
            pg,
            jwt_issuer,
            role_matrix,
            oidc_pending: Arc::new(RwLock::new(HashMap::new())),
            geoip,
            travel_policy: PolicyCache::new(),
            sticky_suppress: StickySuppress::new(),
            rate_limit: Arc::new(RateLimiter::new()),
            deny_list: crate::deny_list::DenyList::new(),
        })
    }

    async fn ensure_signing_key(
        pg: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
