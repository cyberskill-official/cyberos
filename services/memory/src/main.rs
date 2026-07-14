//! cyberos-memory — main binary entry point.
//!
//! Wires three concerns together:
//!   * **HTTP** — axum app with `/healthz` (+ `/v1/memory/search` once TASK-MEMORY-108 lands).
//!   * **Ingest daemon** — background tokio task per tenant calling
//!     `layer2::ingest::run_batch` on `default_poll_interval()`. Tenants
//!     come from the `MEMORY_TENANTS` env var (comma-separated UUIDs) or
//!     are auto-discovered from distinct `tenant_id` values in `l1_audit_log`.
//!   * **Graceful shutdown** — SIGINT/SIGTERM stops the HTTP listener AND
//!     cancels every ingest task; in-flight batches finish their current
//!     transaction before exit.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use cyberos_cli_exit::ExitCode;
use cyberos_memory::{brain, layer2, search, state::AppState, VERSION};
use cyberos_types::TenantId;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cyberos_memory=info,info".into()),
        )
        .json()
        .init();

    let state = match AppState::connect_from_env().await {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "failed to connect to Postgres");
            return ExitCode::ConfigError;
        }
    };

    let shutdown = Arc::new(Notify::new());
    let shutting_down = Arc::new(AtomicBool::new(false));

    // Spawn the ingest daemon. It keeps running until `shutdown` fires.
    let ingest_handle = tokio::spawn({
        let pool = state.pg.clone();
        let shutdown = shutdown.clone();
        let shutting_down = shutting_down.clone();
        async move {
            if let Err(e) = run_ingest_daemon(pool, shutdown, shutting_down).await {
                error!(error = %e, "ingest daemon exited with error");
            }
        }
    });

    // TASK-MEMORY-123 — spawn the BRAIN daemon alongside the Layer-2 loop: consume TASK-MEMORY-121 events ->
    // embed via the ai-gateway -> rolling summaries -> hot/warm/cold tiering, per tenant. Shares the same
    // shutdown signal; on SIGTERM the in-flight ingest tx commits and the cursor + tier watermark are saved
    // (§1 #19). Disabled with BRAIN_INGEST_ENABLED=0 (keeps the recall API + migrations available without the
    // background worker, e.g. in a deploy that ingests on a separate node).
    let brain_handle = tokio::spawn({
        let pool = state.pg.clone();
        let shutdown = shutdown.clone();
        let shutting_down = shutting_down.clone();
        async move {
            if std::env::var("BRAIN_INGEST_ENABLED").as_deref() == Ok("0") {
                info!("brain daemon disabled (BRAIN_INGEST_ENABLED=0) — recall API still served");
                return;
            }
            if let Err(e) = run_brain_daemon(pool, shutdown, shutting_down).await {
                error!(error = %e, "brain daemon exited with error");
            }
        }
    });

    // MEM-001 (R73) — pick up a rotated auth signing key without a restart. Runs only when the key set came
    // from a URL (the production JWKS path); interval is MEMORY_JWKS_REFRESH_SECS (default 300s, floored 30s).
    if state.authenticator.has_jwks_url() {
        let refresher = state.authenticator.clone();
        let secs = std::env::var("MEMORY_JWKS_REFRESH_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300)
            .max(30);
        tokio::spawn(async move {
            let mut tick = time::interval(Duration::from_secs(secs));
            tick.tick().await; // consume the immediate first tick (the boot fetch already loaded the keys)
            loop {
                tick.tick().await;
                match refresher.refresh().await {
                    Ok(()) => tracing::debug!(target: "cyberos_memory::auth", "jwks refreshed"),
                    Err(e) => {
                        tracing::warn!(target: "cyberos_memory::auth", error = %e, "jwks refresh failed")
                    }
                }
            }
        });
    }

    // TASK-OBS-003 - build the RED instruments off the global meter before serving.
    cyberos_obs_sdk::init("memory", VERSION);

    // MEM-001 (R73) — identity comes from a verified JWT, never from headers. `/v1/memory/*` sits behind the
    // require_auth middleware (it stamps the Caller into the request extensions); `/healthz` and `/metrics`
    // stay unauthenticated for liveness + scraping.
    let protected = Router::new()
        .route("/v1/memory/search", post(search::search))
        // TASK-MEMORY-123 §1 #7 — access-scoped, provenance-carrying BRAIN recall.
        .route("/v1/memory/recall", post(brain::handler::recall_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            cyberos_memory::auth::require_auth,
        ));

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .merge(protected)
        // tenant_ctx (route_layer, inner) stamps the request's tenant onto the response; red_mw (layer,
        // outer) reads it for the metric's tenant_id label (ADR-OBS-003-001).
        .route_layer(axum::middleware::from_fn(tenant_ctx))
        .layer(axum::middleware::from_fn_with_state(
            cyberos_obs_sdk::RedState::new("memory"),
            cyberos_obs_sdk::red_mw,
        ))
        .with_state(state.clone());

    let addr: SocketAddr = std::env::var("MEMORY_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7800".into())
        .parse()
        .expect("MEMORY_LISTEN_ADDR must be a valid socket address");

    info!(%addr, version = VERSION, "cyberos-memory starting");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!(error = %e, %addr, "failed to bind");
            shutdown.notify_waiters();
            ingest_handle.abort();
            return ExitCode::NetworkError;
        }
    };

    // Coordinate axum + ingest under one shutdown signal.
    let serve = axum::serve(listener, app).with_graceful_shutdown({
        let shutdown = shutdown.clone();
        let shutting_down = shutting_down.clone();
        async move {
            tokio::select! {
                _ = signal_ctrl_c() => info!("received ctrl-c — shutting down"),
                _ = signal_sigterm() => info!("received SIGTERM — shutting down"),
            }
            shutting_down.store(true, Ordering::SeqCst);
            shutdown.notify_waiters();
        }
    });

    let result = serve.await;
    // Wait briefly for the daemons to drain (in-flight ingest tx commits, cursor + tier watermark saved).
    let _ = tokio::time::timeout(Duration::from_secs(5), ingest_handle).await;
    let _ = tokio::time::timeout(Duration::from_secs(5), brain_handle).await;

    match result {
        Ok(()) => ExitCode::Ok,
        Err(e) => {
            error!(error = %e, "axum serve failed");
            ExitCode::Generic
        }
    }
}

/// Per-tenant ingest daemon. Loops forever until `shutdown` fires.
async fn run_ingest_daemon(
    pool: PgPool,
    shutdown: Arc<Notify>,
    shutting_down: Arc<AtomicBool>,
) -> Result<(), sqlx::Error> {
    let poll_interval = layer2::binlog_tail::default_poll_interval();
    let batch_size: i32 = std::env::var("MEMORY_INGEST_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256);

    info!(?poll_interval, batch_size, "ingest daemon starting");

    loop {
        if shutting_down.load(Ordering::SeqCst) {
            break;
        }
        // On each tick we re-discover tenants so newly-onboarded ones get
        // picked up without restart.
        let tenants = match discover_tenants(&pool).await {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "discover_tenants failed — sleeping and retrying");
                wait_or_shutdown(poll_interval, &shutdown).await;
                continue;
            }
        };

        for tenant in &tenants {
            if shutting_down.load(Ordering::SeqCst) {
                break;
            }
            match layer2::ingest::run_batch(&pool, *tenant, batch_size).await {
                Ok(s) if s.rows_processed > 0 => {
                    info!(?tenant, rows = s.rows_processed, "ingest batch ok");
                }
                Ok(_) => { /* quiet: nothing to do this tick */ }
                Err(e) => {
                    warn!(?tenant, error = %e, "ingest batch failed — will retry next tick");
                }
            }
        }

        wait_or_shutdown(poll_interval, &shutdown).await;
    }

    info!("ingest daemon stopped");
    Ok(())
}

/// TASK-MEMORY-123 — the BRAIN daemon. Per tick, for each tenant: run an ingest pass (consume new events ->
/// embed -> UPSERT), retry any rows left pending by an earlier gateway/cap failure, and — on a slower
/// cadence — run the summarise + tiering passes. Loops until `shutdown` fires; on shutdown it stops between
/// passes so an in-flight ingest transaction always commits (§1 #19).
async fn run_brain_daemon(
    pool: PgPool,
    shutdown: Arc<Notify>,
    shutting_down: Arc<AtomicBool>,
) -> Result<(), sqlx::Error> {
    // The brain polls a touch slower than the raw Layer-2 tail (the embedding round-trip dominates).
    let poll_ms: u64 = std::env::var("BRAIN_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let poll = Duration::from_millis(poll_ms);
    // Run the summarise gauge + tiering pass every N ingest ticks (cheap to skip; expensive to run hot).
    let maintenance_every: u64 = std::env::var("BRAIN_MAINTENANCE_EVERY_TICKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let gw = brain::EmbedClient::from_env();
    let cfg = brain::BrainConfig::from_env();

    info!(?poll, maintenance_every, "brain daemon starting");

    let mut tick: u64 = 0;
    loop {
        if shutting_down.load(Ordering::SeqCst) {
            break;
        }
        let tenants = match discover_tenants(&pool).await {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "brain: discover_tenants failed — sleeping and retrying");
                wait_or_shutdown(poll, &shutdown).await;
                continue;
            }
        };

        let run_maintenance = maintenance_every > 0 && tick % maintenance_every == 0;

        for tenant in &tenants {
            if shutting_down.load(Ordering::SeqCst) {
                break;
            }
            let tid = tenant.as_uuid();

            match brain::ingest_worker::ingest_one_tenant(tid, &pool, &gw).await {
                Ok(s) if s.embedded > 0 || s.pending > 0 => {
                    info!(
                        ?tenant,
                        embedded = s.embedded,
                        pending = s.pending,
                        "brain ingest batch"
                    );
                }
                Ok(_) => { /* quiet: nothing new this tick */ }
                Err(e) => warn!(?tenant, error = %e, "brain ingest batch failed — retry next tick"),
            }

            // Re-embed a bounded slice of rows left pending by an earlier gateway/cap failure.
            if let Err(e) = brain::ingest_worker::retry_pending(tid, &pool, &gw, 64).await {
                warn!(?tenant, error = %e, "brain retry_pending failed");
            }

            if run_maintenance {
                if let Err(e) = brain::summarize::run_summary_pass(&pool, tid).await {
                    warn!(?tenant, error = %e, "brain summary pass failed");
                }
                if let Err(e) = brain::tiering::run_tier_pass(&pool, tid, &cfg).await {
                    warn!(?tenant, error = %e, "brain tier pass failed");
                }
            }
        }

        tick = tick.wrapping_add(1);
        wait_or_shutdown(poll, &shutdown).await;
    }

    info!("brain daemon stopped");
    Ok(())
}

/// Discover tenants to ingest. `MEMORY_TENANTS` env var (comma-separated UUIDs)
/// wins if set; otherwise fall back to distinct tenants present in
/// `l1_audit_log` whose cursor hasn't caught up yet.
async fn discover_tenants(pool: &PgPool) -> Result<Vec<TenantId>, sqlx::Error> {
    if let Ok(raw) = std::env::var("MEMORY_TENANTS") {
        let mut out = Vec::new();
        for s in raw.split(',') {
            let s = s.trim();
            if s.is_empty() {
                continue;
            }
            if let Ok(uuid) = uuid::Uuid::parse_str(s) {
                out.push(TenantId(uuid));
            }
        }
        return Ok(out);
    }

    let rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT l.tenant_id
             FROM l1_audit_log l
        LEFT JOIN l2_ingest_cursor c ON c.tenant_id = l.tenant_id
            WHERE COALESCE(c.last_seq, 0) < (
                SELECT MAX(seq) FROM l1_audit_log WHERE tenant_id = l.tenant_id
            )",
    )
    .fetch_all(pool)
    .await?;

    // Defensive dedupe in case the query somehow returns repeats.
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(rows.len());
    for (u,) in rows {
        if seen.insert(u) {
            out.push(TenantId(u));
        }
    }
    Ok(out)
}

async fn healthz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let pg_ok = sqlx::query("SELECT 1").fetch_one(&state.pg).await.is_ok();
    let status = if pg_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(json!({
            "service": "cyberos-memory",
            "version": VERSION,
            "postgres": if pg_ok { "ok" } else { "down" },
        })),
    )
}

/// Cheap text-format metrics endpoint for Prometheus scraping. Returns a
/// minimal Prometheus exposition: ingest cursor lag per tenant + row counts.
/// RED metrics (TASK-OBS-003) are emitted separately via the obs-sdk middleware over OTLP; this endpoint
/// keeps the memory-specific ingest-cursor gauges.
async fn metrics(State(state): State<AppState>) -> Result<String, (StatusCode, String)> {
    let rows: Vec<(uuid::Uuid, i64, i64)> =
        sqlx::query_as("SELECT tenant_id, last_seq, last_lag_ms FROM l2_ingest_cursor")
            .fetch_all(&state.pg)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut out = String::new();
    out.push_str("# HELP cyberos_memory_l2_cursor_seq Highest L1 seq materialised per tenant\n");
    out.push_str("# TYPE cyberos_memory_l2_cursor_seq counter\n");
    for (t, seq, _) in &rows {
        out.push_str(&format!(
            "cyberos_memory_l2_cursor_seq{{tenant=\"{t}\"}} {seq}\n"
        ));
    }
    out.push_str(
        "# HELP cyberos_memory_l2_last_batch_lag_ms Lag observed on the last ingest batch (ms)\n",
    );
    out.push_str("# TYPE cyberos_memory_l2_last_batch_lag_ms gauge\n");
    for (t, _, lag) in &rows {
        out.push_str(&format!(
            "cyberos_memory_l2_last_batch_lag_ms{{tenant=\"{t}\"}} {lag}\n"
        ));
    }
    Ok(out)
}

/// TASK-OBS-003 - hand the request's tenant (the `x-tenant-id` header memory scopes by) to the RED
/// middleware via the response extensions, so the metric's tenant_id label is real rather than
/// "unknown". Runs as a route_layer (inner of the RED layer); the tenant header is absent on
/// `/healthz` and `/metrics`, which is fine - those are not tenant-scoped.
async fn tenant_ctx(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let tenant = req
        .headers()
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .map(std::string::ToString::to_string);
    let mut response = next.run(req).await;
    if let Some(t) = tenant {
        response
            .extensions_mut()
            .insert(cyberos_obs_sdk::TenantCtx(t));
    }
    response
}

async fn wait_or_shutdown(d: Duration, shutdown: &Notify) {
    tokio::select! {
        _ = time::sleep(d) => {}
        _ = shutdown.notified() => {}
    }
}

async fn signal_ctrl_c() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(unix)]
async fn signal_sigterm() {
    use tokio::signal::unix::{signal, SignalKind};
    if let Ok(mut s) = signal(SignalKind::terminate()) {
        let _ = s.recv().await;
    } else {
        // If we can't install the handler, sleep forever — ctrl-c handler still works.
        std::future::pending::<()>().await;
    }
}

#[cfg(not(unix))]
async fn signal_sigterm() {
    std::future::pending::<()>().await;
}
