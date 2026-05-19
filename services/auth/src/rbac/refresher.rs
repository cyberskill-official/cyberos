//! FR-AUTH-101 §1 #9 + DEC-126 — background `RoleMatrix` refresher.
//!
//! Spawns a tokio task that calls `RoleMatrix::load_from_db` every
//! `MEMORY_RBAC_REFRESH_SECS` (default 60s) and atomically swaps the snapshot
//! via the shared `Arc<RwLock<RoleMatrix>>`. Failures are logged but never
//! kill the task — the previous snapshot keeps serving.
//!
//! The 60s cadence is the documented design assertion (DEC-126): revocations
//! are honoured within 60s. Time-critical revocations (terminated employee)
//! go via a future per-tenant CRL-flush endpoint that targets the in-memory
//! matrix directly (FR-AUTH-111 placeholder).

use crate::rbac::RoleMatrix;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock};
use tracing::{info, warn};

pub fn default_refresh_interval() -> Duration {
    let s: u64 = std::env::var("AUTH_RBAC_REFRESH_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    Duration::from_secs(s.max(5)) // floor at 5s to avoid pathological churn
}

/// Spawn the refresher. Returns immediately. The task lives until `shutdown`
/// fires. Caller is responsible for awaiting the JoinHandle if it wants
/// drain semantics.
pub fn spawn(
    pool: PgPool,
    matrix: Arc<RwLock<RoleMatrix>>,
    shutdown: Arc<Notify>,
) -> tokio::task::JoinHandle<()> {
    let interval = default_refresh_interval();
    tokio::spawn(async move {
        info!(?interval, "RBAC matrix refresher started");
        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = shutdown.notified() => {
                    info!("RBAC matrix refresher stopping");
                    return;
                }
            }
            match RoleMatrix::load_from_db(&pool).await {
                Ok(fresh) => {
                    let mut guard = matrix.write().await;
                    let prev_v = guard.version();
                    let prev_n = guard.len();
                    let new_v = fresh.version();
                    let new_n = fresh.len();
                    *guard = fresh;
                    if new_v != prev_v {
                        info!(
                            old_version = prev_v,
                            new_version = new_v,
                            grants = new_n,
                            "RBAC matrix swapped — catalogue version changed"
                        );
                    } else if new_n != prev_n {
                        info!(
                            grants_delta = (new_n as i64 - prev_n as i64),
                            "RBAC matrix swapped — grants changed"
                        );
                    }
                }
                Err(e) => {
                    warn!(error = %e, "RBAC matrix refresh failed — keeping previous snapshot");
                }
            }
        }
    })
}
