//! FR-AI-007 §3 — Cost table schema types.
//!
//! Defines the public `CostRate` struct and internal YAML deserialization types.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::policy::ProviderKind;

/// A single cost-rate entry for a (provider, model) pair.
///
/// All rates are in USD per 1,000 tokens. Embedding models have
/// `output_per_1k_usd == 0.0` and `is_embedding == true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostRate {
    /// Cost per 1,000 input (prompt) tokens in USD.
    pub input_per_1k_usd: Decimal,
    /// Cost per 1,000 output (completion) tokens in USD.
    /// Zero for embedding models.
    pub output_per_1k_usd: Decimal,
    /// Whether this model produces embeddings (no output tokens).
    pub is_embedding: bool,
}

/// Error returned by [`super::init_cost_table`].
#[derive(Debug)]
pub enum LoaderInitError {
    /// One or more YAML entries failed validation; ALL failures reported.
    Schema { failures: Vec<FileFailure> },
    /// IO error reading the file (missing, permission denied, etc.).
    IoError { path: PathBuf, source: std::io::Error },
    /// Loader already initialised (programmer error — init called twice).
    AlreadyInitialised,
    /// `notify` watcher setup failed.
    WatcherSetup(notify::Error),
}

/// A single validation failure within a YAML file.
#[derive(Debug, Clone)]
pub struct FileFailure {
    /// Path to the offending YAML file.
    pub path: PathBuf,
    /// Provider key (None if the failure is structural, e.g. YAML parse error).
    pub provider: Option<String>,
    /// Model key (None if the failure is per-provider, e.g. unknown provider name).
    pub model: Option<String>,
    /// Human-readable error messages.
    pub errors: Vec<String>,
}

/// Opaque handle that keeps the file watcher alive. Drop = stop watching.
pub struct CostTableHandle {
    pub(crate) _watcher: notify::RecommendedWatcher,
}

impl std::fmt::Debug for CostTableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CostTableHandle").finish_non_exhaustive()
    }
}

/// Top-level YAML structure before validation.
#[derive(Debug, Deserialize)]
pub(crate) struct RawCostTable {
    pub version: u32,
    pub last_updated: NaiveDate,
    #[serde(default)]
    pub source: String,
    pub rates: HashMap<String, HashMap<String, RawCostRate>>,
}

/// Per-model YAML entry before validation.
#[derive(Debug, Deserialize)]
pub(crate) struct RawCostRate {
    pub input_per_1k_usd: Decimal,
    pub output_per_1k_usd: Decimal,
    #[serde(default)]
    pub is_embedding: bool,
}
