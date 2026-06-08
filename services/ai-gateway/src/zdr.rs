//! FR-AI-015 — Zero Data Retention attestation table + enforcement.
//!
//! Maintains an authoritative ZDR attestation table loaded from
//! `config/zdr_attestations.yaml`. Exposes `is_zdr()` for gate checks
//! and `attestation_for()` for audit-trail access.
//!
//! See FR-AI-015 for normative behaviour and acceptance criteria.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration as StdDuration, Instant};

use arc_swap::ArcSwap;
use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_gauge, CounterVec, Gauge};
use tracing::{error, info, warn};
use url::Url;

use crate::policy::ProviderKind;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Soft-stale threshold: 90 days per FR-AI-015 §1 #9.
pub const SOFT_STALE_DAYS: i64 = 90;

/// Hard-stale threshold: 365 days per FR-AI-015 §1 #9.
pub const HARD_STALE_DAYS: i64 = 365;

/// Approved attestor domains per FR-AI-015 §1 #11.
const APPROVED_AUDITOR_DOMAINS: &[&str] =
    &["cyberos.world", "kpmg.com.vn", "ey.com", "deloitte.com"];

// ─── Metrics (FR-AI-015 §4 #14) ──────────────────────────────────────────────

static ZDR_LOOKUPS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_zdr_lookups_total",
        "ZDR lookups by provider, model, and outcome",
        &["provider", "model", "outcome"]
    )
    .unwrap()
});

static ZDR_VIOLATIONS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_zdr_violations_total",
        "ZDR violations by tenant",
        &["tenant_id"]
    )
    .unwrap()
});

static ZDR_REVOKED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_zdr_attestations_revoked_total",
        "ZDR attestations revoked (true→false)",
        &["provider", "model"]
    )
    .unwrap()
});

static ZDR_STALE: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_zdr_attestations_stale_total",
        "ZDR attestations soft-stale (>90 days)",
        &["provider", "model"]
    )
    .unwrap()
});

static ZDR_EXPIRED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_zdr_attestations_expired_total",
        "ZDR attestations hard-stale (>365 days, forced false)",
        &["provider", "model"]
    )
    .unwrap()
});

static ZDR_TABLE_SIZE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("ai_zdr_table_size", "Current ZDR attestation count").unwrap());

// ─── Public types ─────────────────────────────────────────────────────────────

/// ZDR attestation for a single (provider, model) pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZdrAttestation {
    pub is_zdr: bool,
    pub verified_at: NaiveDate,
    pub source_url: String,
    pub attested_by: String,
    pub notes: Option<String>,
}

/// Error from ZDR table initialization.
#[derive(Debug, thiserror::Error)]
pub enum ZdrInitError {
    #[error("zdr_attestations.yaml malformed: {reason}")]
    Schema { reason: String },

    #[error("invalid source_url at {provider}/{model}: must be https://, got {url}")]
    InvalidSourceUrl {
        provider: String,
        model: String,
        url: String,
    },

    #[error("invalid attested_by at {provider}/{model}: {value}")]
    InvalidAttestor {
        provider: String,
        model: String,
        value: String,
    },

    #[error("zdr table already initialised")]
    AlreadyInitialised,

    #[error("io error reading config: {0}")]
    Io(#[from] std::io::Error),
}

// ─── Global state ─────────────────────────────────────────────────────────────

pub type AttestationTable = HashMap<(ProviderKind, String), ZdrAttestation>;

static TABLE: Lazy<ArcSwap<AttestationTable>> = Lazy::new(|| ArcSwap::from_pointee(HashMap::new()));
static TABLE_INITIALISED: AtomicBool = AtomicBool::new(false);

const WATCH_DEBOUNCE: StdDuration = StdDuration::from_millis(250);
const WATCH_POLL: StdDuration = StdDuration::from_millis(25);

/// Keeps the notify watcher and debounce worker alive.
#[derive(Debug)]
pub struct ZdrWatcher {
    _watcher: RecommendedWatcher,
    _worker: JoinHandle<()>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// FR-AI-015 §1 #2 — Check if a (provider, model) pair is ZDR-attested.
///
/// Returns `false` for missing entries (fail-closed, §1 #3).
/// Hard-stale entries (>365 days) are forced to `false` (§1 #9).
pub fn is_zdr(provider: &ProviderKind, model: &str) -> bool {
    ensure_default_table_loaded();
    if !TABLE_INITIALISED.load(Ordering::SeqCst) {
        return false;
    }
    let table = TABLE.load();
    let key = (*provider, model.to_string());
    match table.get(&key) {
        None => {
            ZDR_LOOKUPS
                .with_label_values(&[provider.as_metric_label(), model, "missing"])
                .inc();
            false
        }
        Some(att) if is_hard_stale(att) => {
            ZDR_LOOKUPS
                .with_label_values(&[provider.as_metric_label(), model, "expired"])
                .inc();
            ZDR_EXPIRED
                .with_label_values(&[provider.as_metric_label(), model])
                .inc();
            error!(
                provider = ?provider,
                model = %model,
                verified_at = %att.verified_at,
                "zdr attestation HARD-stale (>365d); forcing is_zdr=false"
            );
            false
        }
        Some(att) => {
            if is_soft_stale(att) {
                ZDR_STALE
                    .with_label_values(&[provider.as_metric_label(), model])
                    .inc();
            }
            ZDR_LOOKUPS
                .with_label_values(&[
                    provider.as_metric_label(),
                    model,
                    if att.is_zdr { "attested" } else { "missing" },
                ])
                .inc();
            att.is_zdr
        }
    }
}

/// FR-AI-015 §1 #2 — Get the full attestation for a (provider, model) pair.
pub fn attestation_for(provider: &ProviderKind, model: &str) -> Option<ZdrAttestation> {
    ensure_default_table_loaded();
    if !TABLE_INITIALISED.load(Ordering::SeqCst) {
        return None;
    }
    let table = TABLE.load();
    table.get(&(*provider, model.to_string())).cloned()
}

/// FR-AI-015 §1 #1 — Load the ZDR attestation table from YAML.
pub fn init_zdr_table(config_path: &Path) -> Result<(), ZdrInitError> {
    let yaml = std::fs::read_to_string(config_path)?;
    let parsed = parse_attestations(&yaml)?;

    TABLE_INITIALISED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| ZdrInitError::AlreadyInitialised)?;

    ZDR_TABLE_SIZE.set(parsed.len() as f64);
    info!(count = parsed.len(), "zdr_table_loaded");
    TABLE.store(Arc::new(parsed));

    Ok(())
}

/// Reload the ZDR table and atomically swap it into the read path.
pub fn reload_zdr_table(config_path: &Path) -> Result<(), ZdrInitError> {
    if !TABLE_INITIALISED.load(Ordering::SeqCst) {
        return init_zdr_table(config_path);
    }

    let yaml = std::fs::read_to_string(config_path)?;
    let parsed = parse_attestations(&yaml)?;
    let previous = TABLE.load();

    for ((provider, model), old) in previous.iter() {
        let revoked = old.is_zdr
            && match parsed.get(&(*provider, model.clone())) {
                Some(new) => !new.is_zdr,
                None => true,
            };
        if revoked {
            warn!(
                provider = provider.as_metric_label(),
                model = %model,
                "zdr attestation revoked"
            );
            ZDR_REVOKED
                .with_label_values(&[provider.as_metric_label(), model])
                .inc();
        }
    }

    ZDR_TABLE_SIZE.set(parsed.len() as f64);
    TABLE.store(Arc::new(parsed));
    info!(count = TABLE.load().len(), "zdr_table_reloaded");

    Ok(())
}

/// Watch `config_path` and reload after a 250ms debounce window.
pub fn start_watcher(config_path: PathBuf) -> notify::Result<ZdrWatcher> {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

    let worker_path = config_path.clone();
    let worker = std::thread::spawn(move || {
        let mut pending_since: Option<Instant> = None;
        loop {
            match rx.recv_timeout(WATCH_POLL) {
                Ok(()) => pending_since = Some(Instant::now()),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
            }

            if pending_since
                .map(|started| started.elapsed() >= WATCH_DEBOUNCE)
                .unwrap_or(false)
            {
                if let Err(err) = reload_zdr_table(&worker_path) {
                    warn!(?err, path = %worker_path.display(), "zdr table reload failed; cache unchanged");
                }
                pending_since = None;
            }
        }
    });

    Ok(ZdrWatcher {
        _watcher: watcher,
        _worker: worker,
    })
}

/// Record a ZDR violation metric for the tenant.
pub fn record_violation(tenant_id: &str) {
    ZDR_VIOLATIONS.with_label_values(&[tenant_id]).inc();
}

/// Reset global table state for integration tests.
#[doc(hidden)]
pub fn reset_for_tests() {
    TABLE.store(Arc::new(HashMap::new()));
    TABLE_INITIALISED.store(false, Ordering::SeqCst);
    ZDR_TABLE_SIZE.set(0.0);
}

/// Replace global table state for integration tests.
#[doc(hidden)]
pub fn replace_for_tests(table: AttestationTable) {
    ZDR_TABLE_SIZE.set(table.len() as f64);
    TABLE.store(Arc::new(table));
    TABLE_INITIALISED.store(true, Ordering::SeqCst);
}

fn ensure_default_table_loaded() {
    if TABLE_INITIALISED.load(Ordering::SeqCst) {
        return;
    }

    let candidates = [
        PathBuf::from("config/zdr_attestations.yaml"),
        PathBuf::from("services/ai-gateway/config/zdr_attestations.yaml"),
        PathBuf::from("../config/zdr_attestations.yaml"),
    ];
    let Some(path) = candidates.iter().find(|p| p.exists()) else {
        return;
    };

    if let Err(err) = init_zdr_table(path) {
        if !matches!(err, ZdrInitError::AlreadyInitialised) {
            tracing::warn!(?path, ?err, "default zdr table load failed");
        }
    }
}

// ─── Staleness helpers ────────────────────────────────────────────────────────

/// Check if an attestation is soft-stale (>90 days).
pub fn is_soft_stale(att: &ZdrAttestation) -> bool {
    Utc::now().date_naive() - att.verified_at > ChronoDuration::days(SOFT_STALE_DAYS)
}

/// Check if an attestation is hard-stale (>365 days).
pub fn is_hard_stale(att: &ZdrAttestation) -> bool {
    Utc::now().date_naive() - att.verified_at > ChronoDuration::days(HARD_STALE_DAYS)
}

// ─── Parser ───────────────────────────────────────────────────────────────────

/// Parse and validate a ZDR attestation YAML document.
pub fn parse_attestations(yaml: &str) -> Result<AttestationTable, ZdrInitError> {
    let raw: serde_yaml::Value = serde_yaml::from_str(yaml).map_err(|e| ZdrInitError::Schema {
        reason: e.to_string(),
    })?;

    let attestations = raw
        .get("attestations")
        .ok_or_else(|| ZdrInitError::Schema {
            reason: "missing 'attestations' root key".into(),
        })?;

    let mut out = HashMap::new();
    for (provider_yaml, models) in
        attestations
            .as_mapping()
            .ok_or_else(|| ZdrInitError::Schema {
                reason: "'attestations' must be a mapping".into(),
            })?
    {
        let provider_str = provider_yaml.as_str().ok_or_else(|| ZdrInitError::Schema {
            reason: format!("provider key must be a string, got: {:?}", provider_yaml),
        })?;
        let provider = parse_provider_kind(provider_str).ok_or_else(|| ZdrInitError::Schema {
            reason: format!("unknown provider: {}", provider_str),
        })?;

        for (model_yaml, fields) in models.as_mapping().ok_or_else(|| ZdrInitError::Schema {
            reason: format!("{}/models must be a mapping", provider_str),
        })? {
            let model = model_yaml
                .as_str()
                .ok_or_else(|| ZdrInitError::Schema {
                    reason: format!("model key must be a string, got: {:?}", model_yaml),
                })?
                .to_string();
            let att = parse_one_attestation(provider_str, &model, fields)?;
            out.insert((provider, model), att);
        }
    }

    Ok(out)
}

fn parse_one_attestation(
    provider: &str,
    model: &str,
    fields: &serde_yaml::Value,
) -> Result<ZdrAttestation, ZdrInitError> {
    let map = fields.as_mapping().ok_or_else(|| ZdrInitError::Schema {
        reason: format!("{}/{}: not a mapping", provider, model),
    })?;

    let is_zdr = map
        .get(&serde_yaml::Value::String("is_zdr".into()))
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ZdrInitError::Schema {
            reason: format!("{}/{}: missing or non-bool is_zdr", provider, model),
        })?;

    let verified_at_s = map
        .get(&serde_yaml::Value::String("verified_at".into()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ZdrInitError::Schema {
            reason: format!("{}/{}: missing verified_at", provider, model),
        })?;
    let verified_at =
        NaiveDate::parse_from_str(verified_at_s, "%Y-%m-%d").map_err(|e| ZdrInitError::Schema {
            reason: format!("{}/{}: bad verified_at: {}", provider, model, e),
        })?;

    let source_url = map
        .get(&serde_yaml::Value::String("source_url".into()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ZdrInitError::Schema {
            reason: format!("{}/{}: missing source_url", provider, model),
        })?
        .to_string();
    validate_source_url(provider, model, &source_url)?;

    let attested_by = map
        .get(&serde_yaml::Value::String("attested_by".into()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ZdrInitError::Schema {
            reason: format!("{}/{}: missing attested_by", provider, model),
        })?
        .to_string();
    validate_attested_by(provider, model, &attested_by)?;

    let notes = map
        .get(&serde_yaml::Value::String("notes".into()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(ZdrAttestation {
        is_zdr,
        verified_at,
        source_url,
        attested_by,
        notes,
    })
}

fn validate_source_url(provider: &str, model: &str, url: &str) -> Result<(), ZdrInitError> {
    let parsed = Url::parse(url).map_err(|_| ZdrInitError::InvalidSourceUrl {
        provider: provider.into(),
        model: model.into(),
        url: url.into(),
    })?;
    if parsed.scheme() != "https" {
        return Err(ZdrInitError::InvalidSourceUrl {
            provider: provider.into(),
            model: model.into(),
            url: url.into(),
        });
    }
    let Some(host) = parsed.host_str() else {
        return Err(ZdrInitError::InvalidSourceUrl {
            provider: provider.into(),
            model: model.into(),
            url: url.into(),
        });
    };
    if !is_valid_dns_host(host) {
        return Err(ZdrInitError::InvalidSourceUrl {
            provider: provider.into(),
            model: model.into(),
            url: url.into(),
        });
    }
    Ok(())
}

fn validate_attested_by(provider: &str, model: &str, value: &str) -> Result<(), ZdrInitError> {
    let Some((local, domain)) = value.split_once('@') else {
        return Err(ZdrInitError::InvalidAttestor {
            provider: provider.into(),
            model: model.into(),
            value: value.into(),
        });
    };
    if local.is_empty()
        || !local
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        || !APPROVED_AUDITOR_DOMAINS.contains(&domain)
    {
        return Err(ZdrInitError::InvalidAttestor {
            provider: provider.into(),
            model: model.into(),
            value: value.into(),
        });
    }
    Ok(())
}

fn is_valid_dns_host(host: &str) -> bool {
    if host.is_empty() || host.len() > 253 {
        return false;
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

fn parse_provider_kind(s: &str) -> Option<ProviderKind> {
    match s {
        "bedrock" => Some(ProviderKind::Bedrock),
        "anthropic" => Some(ProviderKind::Anthropic),
        "openai" => Some(ProviderKind::Openai),
        "vertex" => Some(ProviderKind::Vertex),
        "bge" => Some(ProviderKind::Bge),
        _ => None,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_table() {
        // Tests need to re-initialize; we can't reset a OnceCell, so tests
        // that need fresh state should use parse_attestations directly.
    }

    #[test]
    fn parse_valid_yaml() {
        let yaml = r#"
version: 1
attestations:
  bedrock:
    "anthropic.claude-3-5-sonnet-20241022-v2:0":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
"#;
        let table = parse_attestations(yaml).unwrap();
        assert_eq!(table.len(), 1);
        let att = table
            .get(&(
                ProviderKind::Bedrock,
                "anthropic.claude-3-5-sonnet-20241022-v2:0".into(),
            ))
            .unwrap();
        assert!(att.is_zdr);
        assert_eq!(
            att.verified_at,
            NaiveDate::from_ymd_opt(2026, 5, 21).unwrap()
        );
    }

    #[test]
    fn parse_rejects_http_source_url() {
        let yaml = r#"
version: 1
attestations:
  openai:
    "gpt-4o":
      is_zdr: false
      verified_at: 2026-05-21
      source_url: "http://platform.openai.com/policy"
      attested_by: "stephen@cyberos.world"
"#;
        let err = parse_attestations(yaml).unwrap_err();
        assert!(matches!(err, ZdrInitError::InvalidSourceUrl { .. }));
    }

    #[test]
    fn parse_rejects_bare_attestor() {
        let yaml = r#"
version: 1
attestations:
  openai:
    "gpt-4o":
      is_zdr: false
      verified_at: 2026-05-21
      source_url: "https://platform.openai.com/policy"
      attested_by: "alice"
"#;
        let err = parse_attestations(yaml).unwrap_err();
        assert!(matches!(err, ZdrInitError::InvalidAttestor { .. }));
    }

    #[test]
    fn parse_rejects_unapproved_domain() {
        let yaml = r#"
version: 1
attestations:
  openai:
    "gpt-4o":
      is_zdr: false
      verified_at: 2026-05-21
      source_url: "https://platform.openai.com/policy"
      attested_by: "alice@gmail.com"
"#;
        let err = parse_attestations(yaml).unwrap_err();
        assert!(matches!(err, ZdrInitError::InvalidAttestor { .. }));
    }

    #[test]
    fn parse_rejects_missing_source_url() {
        let yaml = r#"
version: 1
attestations:
  openai:
    "gpt-4o":
      is_zdr: false
      verified_at: 2026-05-21
      attested_by: "stephen@cyberos.world"
"#;
        let err = parse_attestations(yaml).unwrap_err();
        match err {
            ZdrInitError::Schema { reason } => assert!(reason.contains("source_url")),
            e => panic!("wrong variant: {:?}", e),
        }
    }

    #[test]
    fn parse_rejects_missing_attested_by() {
        let yaml = r#"
version: 1
attestations:
  openai:
    "gpt-4o":
      is_zdr: false
      verified_at: 2026-05-21
      source_url: "https://platform.openai.com/policy"
"#;
        let err = parse_attestations(yaml).unwrap_err();
        match err {
            ZdrInitError::Schema { reason } => assert!(reason.contains("attested_by")),
            e => panic!("wrong variant: {:?}", e),
        }
    }

    #[test]
    fn parse_rejects_missing_is_zdr() {
        let yaml = r#"
version: 1
attestations:
  openai:
    "gpt-4o":
      verified_at: 2026-05-21
      source_url: "https://platform.openai.com/policy"
      attested_by: "stephen@cyberos.world"
"#;
        let err = parse_attestations(yaml).unwrap_err();
        match err {
            ZdrInitError::Schema { reason } => assert!(reason.contains("is_zdr")),
            e => panic!("wrong variant: {:?}", e),
        }
    }

    #[test]
    fn soft_stale_at_91_days() {
        let att = ZdrAttestation {
            is_zdr: true,
            verified_at: Utc::now().date_naive() - ChronoDuration::days(91),
            source_url: "https://x".into(),
            attested_by: "stephen@cyberos.world".into(),
            notes: None,
        };
        assert!(is_soft_stale(&att));
        assert!(!is_hard_stale(&att));
    }

    #[test]
    fn hard_stale_at_366_days() {
        let att = ZdrAttestation {
            is_zdr: true,
            verified_at: Utc::now().date_naive() - ChronoDuration::days(366),
            source_url: "https://x".into(),
            attested_by: "stephen@cyberos.world".into(),
            notes: None,
        };
        assert!(is_hard_stale(&att));
    }

    #[test]
    fn not_stale_at_30_days() {
        let att = ZdrAttestation {
            is_zdr: true,
            verified_at: Utc::now().date_naive() - ChronoDuration::days(30),
            source_url: "https://x".into(),
            attested_by: "stephen@cyberos.world".into(),
            notes: None,
        };
        assert!(!is_soft_stale(&att));
        assert!(!is_hard_stale(&att));
    }

    #[test]
    fn parse_provider_kind_variants() {
        assert_eq!(parse_provider_kind("bedrock"), Some(ProviderKind::Bedrock));
        assert_eq!(
            parse_provider_kind("anthropic"),
            Some(ProviderKind::Anthropic)
        );
        assert_eq!(parse_provider_kind("openai"), Some(ProviderKind::Openai));
        assert_eq!(parse_provider_kind("vertex"), Some(ProviderKind::Vertex));
        assert_eq!(parse_provider_kind("bge"), Some(ProviderKind::Bge));
        assert_eq!(parse_provider_kind("unknown"), None);
    }
}
