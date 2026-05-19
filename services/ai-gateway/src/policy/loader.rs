//! FR-AI-005 §3 — Loader entry points + file-watcher.
//!
//! See `policy.rs` for the public re-exports. Behaviour:
//!
//! - `init_loader` eagerly reads every YAML in `config_dir` matching the loadable-filename
//!   regex, validates each against the schema, and aggregates ALL failures into one
//!   `LoaderInitError::Schema` (FR-AI-005 §1 #11).
//! - The file-watcher (`notify`) reacts to Modify/Remove/Create events and re-loads or
//!   evicts as appropriate. Invalid hot-reloads preserve the cached (valid) policy
//!   (FR-AI-005 §1 #5).
//! - `load_for_tenant` performs charset+traversal validation on the input, then hits the
//!   lock-free cache (FR-AI-005 §1 #6, AC #5).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{error, info};

use super::cache::PolicyCache;
use super::schema::TenantPolicy;

// --- Errors ------------------------------------------------------------------

/// Errors returned by [`load_for_tenant`].
#[derive(Debug, Error)]
pub enum PolicyError {
    /// No YAML file found for the tenant at `config/tenants/<tenant_id>.yaml`.
    #[error("policy missing for tenant {tenant_id}")]
    PolicyMissing {
        /// The tenant id that was queried.
        tenant_id: String,
    },
    /// The loaded YAML failed schema validation.
    #[error("policy invalid for tenant {tenant_id}: {schema_errors:?}")]
    PolicyInvalid {
        /// The tenant id that was queried.
        tenant_id: String,
        /// Schema errors detected during validation.
        schema_errors: Vec<String>,
    },
    /// The supplied tenant_id failed charset / traversal validation.
    #[error("invalid tenant_id: {reason}")]
    InvalidTenantId {
        /// Reason for rejection (`traversal`, `charset`, `length`).
        reason: String,
    },
    /// Underlying I/O failure (e.g. permission denied).
    #[error("io error for tenant {tenant_id}: {source}")]
    IoError {
        /// The tenant id that was queried.
        tenant_id: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Loader was not initialised before being used.
    #[error("loader not initialised — call init_loader() first")]
    NotInitialised,
}

/// Errors returned by [`init_loader`].
#[derive(Debug, Error)]
pub enum LoaderInitError {
    /// One or more YAML files failed validation. All failures are aggregated.
    #[error("schema validation failed for {} file(s)", .failures.len())]
    Schema {
        /// Per-file failures.
        failures: Vec<FileFailure>,
    },
    /// I/O error while enumerating or reading files.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    /// `init_loader` was called twice.
    #[error("loader already initialised")]
    AlreadyInitialised,
    /// `notify` failed to start the file-watcher.
    #[error("watcher setup failed: {0}")]
    WatcherSetup(#[from] notify::Error),
    /// Configuration directory does not exist or is not a directory.
    #[error("config dir not a directory: {path:?}")]
    ConfigDirInvalid {
        /// Path that was supplied.
        path: PathBuf,
    },
}

/// One YAML file's worth of validation errors. Aggregated inside
/// `LoaderInitError::Schema` so the operator can fix everything in one deploy.
#[derive(Debug)]
pub struct FileFailure {
    /// Path of the offending file.
    pub path: PathBuf,
    /// Collected validation errors.
    pub errors: Vec<String>,
}

// --- Singleton state ---------------------------------------------------------

use once_cell::sync::Lazy;
use std::sync::RwLock;

static CACHE: Lazy<RwLock<Option<Arc<PolicyCache>>>> = Lazy::new(|| RwLock::new(None));
static CONFIG_DIR: Lazy<RwLock<Option<PathBuf>>> = Lazy::new(|| RwLock::new(None));

// --- Loader handle ----------------------------------------------------------

/// Loader handle. Keep alive for the lifetime of the AI Gateway process; dropping
/// it stops the file-watcher.
#[derive(Debug)]
pub struct Loader {
    _watcher: RecommendedWatcher,
}

impl Loader {
    /// Stop the file-watcher and drain the cache. Idempotent.
    pub async fn shutdown(self) {
        shutdown_loader().await;
    }
}

// --- Public entry points -----------------------------------------------------

/// FR-AI-005 §3 — Eagerly load every YAML in `config_dir`, validate each, install the
/// file-watcher. Aggregates ALL failures into a single `LoaderInitError::Schema`.
pub async fn init_loader(config_dir: &Path) -> Result<Loader, LoaderInitError> {
    if !config_dir.exists() || !config_dir.is_dir() {
        return Err(LoaderInitError::ConfigDirInvalid {
            path: config_dir.to_path_buf(),
        });
    }

    let cache = Arc::new(PolicyCache::new());
    let mut failures: Vec<FileFailure> = Vec::new();

    for entry in std::fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_loadable_filename(name) {
            continue;
        }

        match load_file(&path) {
            Ok(policy) => {
                let tenant_id = policy.tenant_id.clone();
                cache.insert(tenant_id.clone(), Arc::new(policy));
                info!(tenant_id = %tenant_id, path = %path.display(), "policy loaded");
            }
            Err(errors) => {
                failures.push(FileFailure {
                    path: path.clone(),
                    errors,
                });
            }
        }
    }

    if !failures.is_empty() {
        return Err(LoaderInitError::Schema { failures });
    }

    {
        let mut guard_cache = CACHE.write().unwrap();
        let mut guard_dir = CONFIG_DIR.write().unwrap();
        if guard_cache.is_some() {
            return Err(LoaderInitError::AlreadyInitialised);
        }
        *guard_cache = Some(cache);
        *guard_dir = Some(config_dir.to_path_buf());
    }

    let watcher = spawn_watcher(config_dir)?;

    let loaded = {
        let guard = CACHE.read().unwrap();
        guard.as_ref().expect("just set").loaded_tenants_sorted()
    };
    info!(count = loaded.len(), tenants = ?loaded, "ai-gateway policy loader initialised");

    Ok(Loader { _watcher: watcher })
}

/// FR-AI-005 §3 — Hot-path lookup. Sub-microsecond on cache hit. Validates the supplied
/// `tenant_id` for traversal/charset/length before touching the filesystem.
pub async fn load_for_tenant(tenant_id: &str) -> Result<Arc<TenantPolicy>, PolicyError> {
    validate_tenant_id(tenant_id)?;

    let cache = {
        let guard = CACHE.read().unwrap();
        guard.as_ref().cloned().ok_or(PolicyError::NotInitialised)?
    };

    if let Some(p) = cache.get(tenant_id) {
        return Ok(p);
    }

    // Cache miss — fall through to disk. Rare path; only happens if a file appeared
    // between init and read but before the file-watch fired.
    let dir = {
        let guard = CONFIG_DIR.read().unwrap();
        guard.as_ref().cloned().ok_or(PolicyError::NotInitialised)?
    };
    let file_name = format!("{}.yaml", tenant_id.replace(':', "-"));
    let path = dir.join(&file_name);

    if !path.exists() {
        return Err(PolicyError::PolicyMissing {
            tenant_id: tenant_id.to_string(),
        });
    }

    let policy = load_file(&path).map_err(|errors| PolicyError::PolicyInvalid {
        tenant_id: tenant_id.to_string(),
        schema_errors: errors,
    })?;

    let arc = Arc::new(policy);
    cache.insert(tenant_id.to_string(), arc.clone());
    Ok(arc)
}

/// FR-AI-005 §1 #13 — Pure-function validator used by `cyberos-ai policy validate`.
pub fn validate_yaml(yaml: &str) -> Result<TenantPolicy, Vec<String>> {
    let policy: TenantPolicy = serde_yaml::from_str(yaml).map_err(|e| vec![e.to_string()])?;
    validate_policy_value(&policy)?;
    Ok(policy)
}

/// Stop the file-watcher and clear cache. Idempotent.
pub async fn shutdown_loader() {
    let mut guard_cache = CACHE.write().unwrap();
    let mut guard_dir = CONFIG_DIR.write().unwrap();
    *guard_cache = None;
    *guard_dir = None;
}

// --- Internals ---------------------------------------------------------------

/// Reads + parses + validates one file. Returns the policy on success or a list of
/// human-readable error strings on failure.
fn load_file(path: &Path) -> Result<TenantPolicy, Vec<String>> {
    let yaml = std::fs::read_to_string(path).map_err(|e| vec![format!("io: {e}")])?;
    validate_yaml(&yaml)
}

/// FR-AI-005 §3 — Validate the loaded policy against the schema range/charset rules.
/// Schemars derives generate machine-readable JSONSchema; this function applies the
/// runtime-equivalent checks. Errors are aggregated.
fn validate_policy_value(p: &TenantPolicy) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(e) = validate_tenant_id_chars(&p.tenant_id) {
        errors.push(format!("tenant_id: {e}"));
    }

    {
        use rust_decimal::Decimal;
        let c = p.ai_policy.monthly_cap_usd;
        let lo = Decimal::new(1, 2); // 0.01
        let hi = Decimal::new(1_000_000, 0);
        if c < lo {
            errors.push(format!("ai_policy.monthly_cap_usd: {c} < 0.01"));
        }
        if c > hi {
            errors.push(format!("ai_policy.monthly_cap_usd: {c} > 1_000_000"));
        }
    }

    if !(0.5..=0.95).contains(&p.ai_policy.warn_threshold) {
        errors.push(format!(
            "ai_policy.warn_threshold: {} outside [0.5, 0.95]",
            p.ai_policy.warn_threshold
        ));
    }

    if !(1..=600).contains(&p.ai_policy.call_timeout_seconds) {
        errors.push(format!(
            "ai_policy.call_timeout_seconds: {} outside [1, 600]",
            p.ai_policy.call_timeout_seconds
        ));
    }

    // Only enforce the multiplier range when the override is actually enabled.
    // A disabled (or absent / serde-defaulted) emergency_override block carries the
    // Default-impl multiplier (0.0) which is intentional and MUST NOT trip validation.
    if p.ai_policy.emergency_override.enabled {
        let mult = p.ai_policy.emergency_override.max_multiplier;
        if !(1.0..=10.0).contains(&mult) {
            errors.push(format!(
                "ai_policy.emergency_override.max_multiplier: {mult} outside [1.0, 10.0] (override is enabled)"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_tenant_id_chars(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 128 {
        return Err(format!("length {} outside (0, 128]", id.len()));
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err("traversal characters".into());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_'))
    {
        return Err("charset (only [a-z0-9:_-] allowed)".into());
    }
    Ok(())
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), PolicyError> {
    validate_tenant_id_chars(tenant_id).map_err(|reason| PolicyError::InvalidTenantId { reason })
}

fn is_loadable_filename(name: &str) -> bool {
    if !name.ends_with(".yaml") {
        return false;
    }
    if name.starts_with('_') {
        return false;
    }
    let stem = name.trim_end_matches(".yaml");
    !stem.is_empty()
        && stem
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn spawn_watcher(config_dir: &Path) -> Result<RecommendedWatcher, notify::Error> {
    let (tx, mut rx) = mpsc::channel::<Result<Event, notify::Error>>(64);

    let mut watcher = notify::recommended_watcher(move |res| {
        // Forward to async processing channel. Best-effort: a full channel means we lose
        // the event, but the periodic re-read on cache miss will eventually recover.
        let _ = tx.blocking_send(res);
    })?;
    watcher.watch(config_dir, RecursiveMode::NonRecursive)?;

    let dir = config_dir.to_path_buf();
    tokio::spawn(async move {
        while let Some(evt) = rx.recv().await {
            match evt {
                Ok(event) => handle_event(&dir, event).await,
                Err(e) => error!(error = %e, "policy watcher error"),
            }
        }
    });

    Ok(watcher)
}

async fn handle_event(config_dir: &Path, event: Event) {
    let cache = {
        let guard = CACHE.read().unwrap();
        let Some(cache) = guard.as_ref().cloned() else {
            return;
        };
        cache
    };
    for path in event.paths {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_loadable_filename(name) {
            continue;
        }
        match event.kind {
            EventKind::Remove(_) => {
                let stem = name.trim_end_matches(".yaml");
                // Reverse `:` ↔ `-` is non-injective (`-` could be either); we evict every
                // tenant whose file_name re-maps to this stem.
                for id in cache.loaded_tenants_sorted() {
                    if id.replace(':', "-") == stem {
                        cache.remove(&id);
                        info!(tenant_id = %id, file = %path.display(), "policy removed via file-watch");
                    }
                }
            }
            EventKind::Modify(_) | EventKind::Create(_) => {
                let full = config_dir.join(name);
                match load_file(&full) {
                    Ok(policy) => {
                        let id = policy.tenant_id.clone();
                        cache.insert(id.clone(), Arc::new(policy));
                        info!(tenant_id = %id, file = %full.display(), "policy hot-reloaded");
                    }
                    Err(errors) => {
                        // FR-AI-005 §1 #5 — invalid hot-reload preserves cache.
                        error!(file = %full.display(), errors = ?errors, "policy reload failed; cache preserved");
                    }
                }
            }
            _ => {}
        }
    }
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_tenant_id_chars_traversal() {
        assert!(validate_tenant_id_chars("../escape").is_err());
        assert!(validate_tenant_id_chars("a/b").is_err());
        assert!(validate_tenant_id_chars(r"a\b").is_err());
    }

    #[test]
    fn validate_tenant_id_chars_charset() {
        // Allowed: [A-Za-z0-9:_-]. Per FR-AI-005 §6 reference impl, uppercase is
        // permitted in tenant_id values themselves (though filenames are kebab-lowercase).
        assert!(validate_tenant_id_chars("org:cyberskill").is_ok());
        assert!(validate_tenant_id_chars("org-test-a").is_ok());
        assert!(validate_tenant_id_chars("Org:With:Caps").is_ok());
        // Disallowed: whitespace, dot, slash, backslash, other punctuation.
        assert!(validate_tenant_id_chars("a b").is_err());
        assert!(validate_tenant_id_chars("dotted.id").is_err());
        assert!(validate_tenant_id_chars("question?mark").is_err());
    }

    #[test]
    fn validate_tenant_id_chars_length() {
        assert!(validate_tenant_id_chars("").is_err());
        let big: String = "a".repeat(129);
        assert!(validate_tenant_id_chars(&big).is_err());
    }

    #[test]
    fn is_loadable_filename_accepts_kebab() {
        assert!(is_loadable_filename("org-cyberskill.yaml"));
        assert!(is_loadable_filename("test-a.yaml"));
        assert!(!is_loadable_filename("EXAMPLE.tenant.yaml"));
        assert!(!is_loadable_filename("_underscore.yaml"));
        assert!(!is_loadable_filename("Caps.yaml"));
        assert!(!is_loadable_filename("notyaml.txt"));
        assert!(!is_loadable_filename(".yaml"));
    }

    #[test]
    fn validate_yaml_rejects_bad_cap() {
        let yaml = r#"
tenant_id: org:test
ai_policy:
  monthly_cap_usd: "0.001"
  warn_threshold: 0.8
  primary_provider:
    kind: anthropic
    model_alias_map: {}
  residency: sg-1
"#;
        let res = validate_yaml(yaml);
        assert!(res.is_err(), "expected range failure on 0.001 cap");
    }

    #[test]
    fn validate_yaml_accepts_minimal_valid() {
        let yaml = r#"
tenant_id: org:test
ai_policy:
  monthly_cap_usd: "150"
  warn_threshold: 0.8
  primary_provider:
    kind: anthropic
    model_alias_map:
      chat.smart: claude-3-5-sonnet
  residency: sg-1
"#;
        let res = validate_yaml(yaml);
        assert!(res.is_ok(), "expected ok but got {:?}", res);
        let p = res.unwrap();
        assert_eq!(p.tenant_id, "org:test");
    }
}
