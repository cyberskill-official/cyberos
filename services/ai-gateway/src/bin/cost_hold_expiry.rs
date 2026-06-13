//! FR-AI-004 — Standalone cost-hold expiry cleanup binary.
//!
//! Runs as a long-lived process, scanning for expired holds every 30 seconds
//! (configurable via CYBEROS_AI_EXPIRY_TICK_SECONDS).
//!
//! See FR-AI-004 for normative behaviour and acceptance criteria.

use std::time::Duration;

use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .init();

    let tick_seconds = std::env::var("CYBEROS_AI_EXPIRY_TICK_SECONDS")
        .unwrap_or_else(|_| "30".into())
        .parse::<u64>()
        .unwrap_or(30)
        .clamp(5, 300);

    let mut shutdown = signal(SignalKind::terminate())?;
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let Some(pool) = connect_with_backoff(&database_url, &mut shutdown).await? else {
        return Ok(());
    };
    let mut tick = tokio::time::interval(Duration::from_secs(tick_seconds));
    let mut consecutive_failures: u32 = 0;

    info!(tick_seconds, "expiry_job_started");

    loop {
        tokio::select! {
            _ = tick.tick() => {
                match cyberos_ai_gateway::cost_hold_expiry::run_tick(&pool).await {
                    Ok(report) => {
                        consecutive_failures = 0;
                        info!(
                            holds_processed = report.holds_processed,
                            holds_succeeded = report.holds_succeeded,
                            holds_failed = report.holds_failed,
                            duration_ms = report.duration_ms,
                            "expiry_tick_complete"
                        );
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        tracing::error!(?e, consecutive_failures, "expiry_tick_failed");
                        if consecutive_failures >= 10 {
                            tracing::error!(
                                consecutive_failures,
                                "expiry_consecutive_failures_threshold"
                            );
                        }
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("process_shutdown");
                break;
            }
        }
    }

    Ok(())
}

async fn connect_with_backoff(
    database_url: &str,
    shutdown: &mut tokio::signal::unix::Signal,
) -> anyhow::Result<Option<sqlx::PgPool>> {
    let mut backoff = Duration::from_secs(1);
    loop {
        let connect_attempt =
            tokio::time::timeout(Duration::from_secs(1), sqlx::PgPool::connect(database_url));
        match tokio::select! {
            result = connect_attempt => Some(result),
            _ = shutdown.recv() => None,
        } {
            Some(Ok(Ok(pool))) => return Ok(Some(pool)),
            Some(Ok(Err(e))) => {
                error!(
                    ?e,
                    backoff_seconds = backoff.as_secs(),
                    "expiry_db_connect_failed"
                );
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {
                        backoff = (backoff * 2).min(Duration::from_secs(30));
                    }
                    _ = shutdown.recv() => {
                        info!("process_shutdown");
                        return Ok(None);
                    }
                }
            }
            Some(Err(_elapsed)) => {
                error!(
                    backoff_seconds = backoff.as_secs(),
                    "expiry_db_connect_timeout"
                );
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {
                        backoff = (backoff * 2).min(Duration::from_secs(30));
                    }
                    _ = shutdown.recv() => {
                        info!("process_shutdown");
                        return Ok(None);
                    }
                }
            }
            None => {
                info!("process_shutdown");
                return Ok(None);
            }
        }
    }
}
