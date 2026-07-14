//! TASK-AI-007 — Cost-table loader with hot-reload.
//!
//! Loads `config/cost_rates.yaml` at startup, validates all entries, caches
//! them lock-free via `ArcSwap`, and hot-reloads on file change via `notify`.
//!
//! See TASK-AI-007 §1 for normative behaviour, §4 for acceptance criteria.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

use super::schema::{CostRate, CostTableHandle, FileFailure, LoaderInitError, RawCostTable};
use crate::policy::ProviderKind;

/// In-memory cost table: (ProviderKind, model_name) → CostRate.
static TABLE: OnceCell<ArcSwap<HashMap<(ProviderKind, String), CostRate>>> = OnceCell::new();

/// Timestamp of last successful load.
static LOADED_AT: OnceCell<ArcSwap<Option<DateTime<Utc>>>> = OnceCell::new();

/// Debounce interval for file-watch events (milliseconds).
const DEBOUNCE_MS: u64 = 100;

// ─── Metrics ──────────────────────────────────────────────────────────────────

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{register_counter_vec, register_int_gauge, CounterVec, Histogram, IntGauge};

    pub static LOOKUPS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_cost_table_lookups_total",
            "Cost-table lookups by provider and outcome",
            &["provider", "outcome"]
        )
        .unwrap()
    });

    pub static RELOAD_FAILURES: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_cost_table_reload_failures_total",
            "Cost-table reload failures by reason",
            &["reason"]
        )
        .unwrap()
    });

    pub static ENTRY_COUNT: Lazy<IntGauge> = Lazy::new(|| {
        register_int_gauge!(
            "ai_cost_table_entries_total",
            "Current count of (provider, model) entries"
        )
        .unwrap()
    });

    pub static LOADED_AT_TS: Lazy<IntGauge> = Lazy::new(|| {
        register_int_gauge!(
            "ai_cost_table_loaded_at_ts",
            "UNIX timestamp of last successful load"
        )
        .unwrap()
    });

    pub static LOOKUP_LATENCY: Lazy<Histogram> = Lazy::new(|| {
        prometheus::register_histogram!(
            "ai_cost_table_lookup_latency_ns",
            "Cost-table lookup latency in nanoseconds",
            vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0]
        )
        .unwrap()
    });
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Synchronous lookup against the in-memory cost table.
///
/// Returns `None` on miss; never panics; lock-free read via `ArcSwap`.
pub fn lookup(provider: &ProviderKind, model: &str) -> Option<CostRate> {
    let started = Instant::now();
    let result = TABLE
        .get()
        .and_then(|s| s.load().get(&(*provider, model.to_string())).copied());
    let outcome = if result.is_some() { "hit" } else { "miss" };
    metrics::LOOKUPS
        .with_label_values(&[provider.as_metric_label(), outcome])
        .inc();
    metrics::LOOKUP_LATENCY.observe(started.elapsed().as_nanos() as f64);
    result
}

/// Returns the UNIX timestamp of the last successful load (or `None` if never loaded).
///
/// Used by TASK-AI-021 operator CLI.
pub fn loaded_at() -> Option<DateTime<Utc>> {
    LOADED_AT.get().and_then(|s| **s.load())
}

/// Returns the current entry count (gauge value).
pub fn entry_count() -> usize {
    TABLE.get().map(|s| s.load().len()).unwrap_or(0)
}

/// Initialise the cost table at gateway startup.
///
/// MUST be called before the HTTP listener is bound.
pub async fn init_cost_table(config_path: &Path) -> Result<CostTableHandle, LoaderInitError> {
    let table = load_and_validate(config_path).await?;
    let count = table.len();

    // Idempotent init: create each cell once, then atomically swap in the freshly loaded
    // table. Re-calling init_cost_table reloads (the same lock-free swap the hot-reload
    // watcher uses) instead of failing with AlreadyInitialised. ArcSwap exists precisely so a
    // reload never blocks readers; a single-shot OnceCell::set defeated that and made every
    // call after the first (each integration test, any re-init) error.
    TABLE
        .get_or_init(|| ArcSwap::from_pointee(HashMap::new()))
        .store(Arc::new(table));
    LOADED_AT
        .get_or_init(|| ArcSwap::from_pointee(None))
        .store(Arc::new(Some(Utc::now())));

    metrics::ENTRY_COUNT.set(count as i64);
    metrics::LOADED_AT_TS.set(Utc::now().timestamp());

    let watcher = spawn_watcher(config_path).await?;
    Ok(CostTableHandle { _watcher: watcher })
}

// ─── Internal: load + validate ────────────────────────────────────────────────

async fn load_and_validate(
    path: &Path,
) -> Result<HashMap<(ProviderKind, String), CostRate>, LoaderInitError> {
    let yaml = std::fs::read_to_string(path).map_err(|source| LoaderInitError::IoError {
        path: path.to_path_buf(),
        source,
    })?;
    let raw: RawCostTable = serde_yaml::from_str(&yaml).map_err(|e| LoaderInitError::Schema {
        failures: vec![FileFailure {
            path: path.to_path_buf(),
            model: None,
            provider: None,
            errors: vec![format!("yaml parse: {}", e)],
        }],
    })?;
    validate_and_flatten(raw, path)
}

fn validate_and_flatten(
    raw: RawCostTable,
    path: &Path,
) -> Result<HashMap<(ProviderKind, String), CostRate>, LoaderInitError> {
    let mut failures: Vec<FileFailure> = Vec::new();
    let mut out = HashMap::new();

    for (provider_str, models) in raw.rates {
        let kind = match parse_provider(&provider_str) {
            Ok(k) => k,
            Err(e) => {
                failures.push(FileFailure {
                    path: path.to_path_buf(),
                    provider: Some(provider_str.clone()),
                    model: None,
                    errors: vec![e],
                });
                continue;
            }
        };

        for (model, rate) in models {
            let mut model_errors: Vec<String> = Vec::new();

            if rate.input_per_1k_usd < Decimal::ZERO {
                model_errors.push(format!(
                    "input_per_1k_usd must be non-negative, got {}",
                    rate.input_per_1k_usd
                ));
            }
            if rate.output_per_1k_usd < Decimal::ZERO {
                model_errors.push(format!(
                    "output_per_1k_usd must be non-negative, got {}",
                    rate.output_per_1k_usd
                ));
            }
            if model.is_empty() || model.len() > 256 {
                model_errors.push(format!(
                    "model name length must be 1..=256, got {}",
                    model.len()
                ));
            }
            // TASK-AI-007 §1 #12: is_embedding ⇒ output_per_1k_usd == 0.0
            if rate.is_embedding && rate.output_per_1k_usd > Decimal::ZERO {
                model_errors.push(format!(
                    "is_embedding: true requires output_per_1k_usd == 0.0, got {}",
                    rate.output_per_1k_usd
                ));
            }

            if !model_errors.is_empty() {
                failures.push(FileFailure {
                    path: path.to_path_buf(),
                    provider: Some(provider_str.clone()),
                    model: Some(model.clone()),
                    errors: model_errors,
                });
                continue;
            }

            out.insert(
                (kind, model),
                CostRate {
                    input_per_1k_usd: rate.input_per_1k_usd,
                    output_per_1k_usd: rate.output_per_1k_usd,
                    is_embedding: rate.is_embedding,
                },
            );
        }
    }

    // TASK-AI-007 §4 #16: deterministic failure order
    failures.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.provider.cmp(&b.provider))
            .then_with(|| a.model.cmp(&b.model))
    });

    if !failures.is_empty() {
        return Err(LoaderInitError::Schema { failures });
    }
    Ok(out)
}

fn parse_provider(s: &str) -> Result<ProviderKind, String> {
    match s {
        "bedrock" => Ok(ProviderKind::Bedrock),
        "anthropic" => Ok(ProviderKind::Anthropic),
        "openai" => Ok(ProviderKind::Openai),
        "vertex" => Ok(ProviderKind::Vertex),
        "bge" => Ok(ProviderKind::Bge),
        "ollama" => Ok(ProviderKind::Ollama),
        "local_openai" => Ok(ProviderKind::LocalOpenai),
        other => Err(format!(
            "unknown provider '{}'; supported: bedrock|anthropic|openai|vertex|bge|ollama|local_openai",
            other
        )),
    }
}

// ─── Internal: hot-reload watcher ─────────────────────────────────────────────

async fn spawn_watcher(path: &Path) -> Result<RecommendedWatcher, LoaderInitError> {
    let (tx, mut rx) = mpsc::channel::<Event>(16);
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ev) = res {
            let _ = tx.blocking_send(ev);
        }
    })
    .map_err(LoaderInitError::WatcherSetup)?;

    let watch_dir: PathBuf = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => {
            tracing::warn!(
                ?path,
                "cost_rates.yaml has no parent dir; watching CWD instead"
            );
            PathBuf::from(".")
        }
    };
    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .map_err(LoaderInitError::WatcherSetup)?;

    let path = path.to_path_buf();
    // The debounced reload loop needs a Tokio runtime. In the server, init runs inside the
    // main runtime; sync callers (block_on-based tests) have none, where `tokio::spawn` would
    // panic with "there is no reactor running". Spawn only when a runtime is present - the table
    // is already loaded by this point, so the only thing skipped without a runtime is live
    // hot-reload, which those contexts do not need.
    match tokio::runtime::Handle::try_current() {
        Ok(rt) => {
            rt.spawn(async move {
                let mut last_event_at: Option<Instant> = None;
                loop {
                    tokio::select! {
                        Some(_event) = rx.recv() => {
                            last_event_at = Some(Instant::now());
                        }
                        _ = tokio::time::sleep(std::time::Duration::from_millis(DEBOUNCE_MS)) => {
                            if let Some(t) = last_event_at {
                                if t.elapsed() >= std::time::Duration::from_millis(DEBOUNCE_MS) {
                                    last_event_at = None;
                                    apply_reload(&path).await;
                                }
                            }
                        }
                    }
                }
            });
        }
        Err(_) => {
            tracing::warn!("init_cost_table called outside a Tokio runtime; hot-reload disabled");
        }
    }

    Ok(watcher)
}

async fn apply_reload(path: &Path) {
    match load_and_validate(path).await {
        Ok(new_table) => {
            if let Some(s) = TABLE.get() {
                let count = new_table.len();
                s.store(Arc::new(new_table));
                metrics::ENTRY_COUNT.set(count as i64);
            }
            if let Some(s) = LOADED_AT.get() {
                s.store(Arc::new(Some(Utc::now())));
                metrics::LOADED_AT_TS.set(Utc::now().timestamp());
            }
            tracing::info!(?path, "cost_table_reloaded");
        }
        Err(e) => {
            let reason = match &e {
                LoaderInitError::Schema { .. } => "validation_error",
                LoaderInitError::IoError { .. } => "io_error",
                _ => "unknown",
            };
            metrics::RELOAD_FAILURES.with_label_values(&[reason]).inc();
            tracing::error!(?path, ?e, "cost_table_reload_failed");
        }
    }
}
