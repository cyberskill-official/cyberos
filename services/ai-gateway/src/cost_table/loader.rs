//! FR-AI-007 — Cost-table loader with hot-reload.
//!
//! Loads `config/cost_rates.yaml` at startup, validates all entries, caches
//! them lock-free via `ArcSwap`, and hot-reloads on file change via `notify`.
//!
//! See FR-AI-007 §1 for normative behaviour, §4 for acceptance criteria.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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

/// Serialises init/re-init publication so tests and CLIs can reload fixtures
/// without racing the global ArcSwap cells.
static INIT_LOCK: Mutex<()> = Mutex::new(());

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
/// Used by FR-AI-021 operator CLI.
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
    let loaded_at = Utc::now();

    {
        let _guard = INIT_LOCK
            .lock()
            .expect("cost-table init mutex should not be poisoned");
        if let Some(cell) = TABLE.get() {
            cell.store(Arc::new(table));
        } else {
            TABLE
                .set(ArcSwap::from_pointee(table))
                .map_err(|_| LoaderInitError::AlreadyInitialised)?;
        }
        if let Some(cell) = LOADED_AT.get() {
            cell.store(Arc::new(Some(loaded_at)));
        } else {
            LOADED_AT
                .set(ArcSwap::from_pointee(Some(loaded_at)))
                .map_err(|_| LoaderInitError::AlreadyInitialised)?;
        }

        metrics::ENTRY_COUNT.set(count as i64);
        metrics::LOADED_AT_TS.set(loaded_at.timestamp());
    }

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

    if raw.version != 1 {
        failures.push(FileFailure {
            path: path.to_path_buf(),
            provider: None,
            model: None,
            errors: vec![format!("version must be 1, got {}", raw.version)],
        });
    }
    let age_days = Utc::now()
        .date_naive()
        .signed_duration_since(raw.last_updated)
        .num_days();
    if age_days > 90 {
        tracing::warn!(
            ?path,
            last_updated = %raw.last_updated,
            age_days,
            "cost_rates_yaml_older_than_90_days"
        );
    }
    if raw.source.trim().is_empty() {
        tracing::warn!(?path, "cost_rates_yaml_source_empty");
    }

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

            let input_per_1k_usd = match rate.input_per_1k_usd {
                Some(value) => {
                    if value < Decimal::ZERO {
                        model_errors.push(format!(
                            "input_per_1k_usd must be non-negative, got {}",
                            value
                        ));
                    }
                    Some(value)
                }
                None => {
                    model_errors.push("input_per_1k_usd is required".to_string());
                    None
                }
            };
            let output_per_1k_usd = match rate.output_per_1k_usd {
                Some(value) => {
                    if value < Decimal::ZERO {
                        model_errors.push(format!(
                            "output_per_1k_usd must be non-negative, got {}",
                            value
                        ));
                    }
                    Some(value)
                }
                None => {
                    model_errors.push("output_per_1k_usd is required".to_string());
                    None
                }
            };
            if let Some(output_per_1k_usd) = output_per_1k_usd {
                // FR-AI-007 §1 #12: is_embedding ⇒ output_per_1k_usd == 0.0
                if rate.is_embedding && output_per_1k_usd > Decimal::ZERO {
                    model_errors.push(format!(
                        "is_embedding: true requires output_per_1k_usd == 0.0, got {}",
                        output_per_1k_usd
                    ));
                }
            }
            if model.is_empty() || model.len() > 256 {
                model_errors.push(format!(
                    "model name length must be 1..=256, got {}",
                    model.len()
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
                    input_per_1k_usd: input_per_1k_usd.expect("validated input rate exists"),
                    output_per_1k_usd: output_per_1k_usd.expect("validated output rate exists"),
                    is_embedding: rate.is_embedding,
                },
            );
        }
    }

    // FR-AI-007 §4 #16: deterministic failure order
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
        other => Err(format!(
            "unknown provider '{}'; supported: bedrock|anthropic|openai|vertex|bge",
            other
        )),
    }
}

fn has_yaml_parse_failure(failures: &[FileFailure]) -> bool {
    failures.iter().any(|failure| {
        failure
            .errors
            .iter()
            .any(|error| error.starts_with("yaml parse:"))
    })
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
    if tokio::runtime::Handle::try_current().is_err() {
        tracing::warn!(
            ?path,
            "cost table watcher registered without a Tokio runtime; hot reload disabled"
        );
        return Ok(watcher);
    }

    tokio::spawn(async move {
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
                LoaderInitError::Schema { failures } if has_yaml_parse_failure(failures) => {
                    "parse_error"
                }
                LoaderInitError::Schema { .. } => "validation_error",
                LoaderInitError::IoError { .. } => "io_error",
                _ => "unknown",
            };
            metrics::RELOAD_FAILURES.with_label_values(&[reason]).inc();
            tracing::error!(?path, ?e, "cost_table_reload_failed");
        }
    }
}
