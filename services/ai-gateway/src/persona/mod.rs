//! FR-AI-014 — Persona-version system-prompt injection.
//!
//! Loads persona definitions from `<memory-root>/memories/personas/<handle>.md`,
//! caches them via `ArcSwap`, and verifies source hashes on every load for
//! tamper detection.
//!
//! See FR-AI-014 for normative behaviour and acceptance criteria.

pub mod hash;
pub mod parse;
pub mod types;
pub mod watch;

pub use types::{
    LlmHints, Persona, PersonaError, PersonaHandle, PersonaId, PersonaInitError, PersonaParseError,
};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_int_gauge, CounterVec, IntGauge};

use crate::memory_writer::{self, MemoryEmit};
use crate::router::{ChatCompleteRequest, MadeByGenie, Message};

use self::types::{REGISTRY, REGISTRY_INITIALISED};

// ─── Metrics (FR-AI-014 §1 #15) ──────────────────────────────────────────────

static PERSONA_LOADS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_persona_loads_total",
        "Persona load outcomes",
        &["handle", "outcome"]
    )
    .unwrap()
});

static PERSONA_TAMPERED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_persona_tampered_total",
        "Persona tamper detections",
        &["handle"]
    )
    .unwrap()
});

static PERSONA_RELOADS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_persona_reload_total",
        "Persona hot-reload outcomes",
        &["outcome"]
    )
    .unwrap()
});

static REGISTRY_SIZE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "ai_persona_registry_size",
        "Current persona registry entry count"
    )
    .unwrap()
});

#[derive(Debug, Clone)]
pub struct AppliedPersona {
    pub request: ChatCompleteRequest,
    pub made_by_genie: Option<MadeByGenie>,
    pub source_hash_hex16: Option<String>,
    pub audit_row: Option<MemoryEmit>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Initialise the persona registry by scanning `<persona_dir>/*.md`.
///
/// Must be called once at gateway startup before any `load()` calls.
pub fn init_persona_registry(persona_dir: &PathBuf) -> Result<(), PersonaInitError> {
    let map = load_map(persona_dir)?;

    REGISTRY_INITIALISED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| PersonaInitError::AlreadyInitialised)?;

    REGISTRY_SIZE.set(map.len() as i64);
    REGISTRY.store(Arc::new(map));

    Ok(())
}

/// Load a persona by handle. Returns a cached `Arc<Persona>` on cache hit,
/// verified for tamper on every load.
pub fn load(handle: &PersonaHandle) -> Result<Arc<Persona>, PersonaError> {
    if !REGISTRY_INITIALISED.load(Ordering::SeqCst) {
        return Err(PersonaError::RegistryNotInitialised);
    }
    let map = REGISTRY.load();

    let Some(persona) = map.get(handle) else {
        let mut avail: Vec<String> = map.keys().map(|h| h.display()).collect();
        avail.sort();
        PERSONA_LOADS
            .with_label_values(&[&handle.display(), "unknown"])
            .inc();
        return Err(PersonaError::UnknownPersona {
            handle: handle.display(),
            available: avail,
        });
    };

    // Tamper check on every load (§1 #7)
    hash::verify_persona(persona).map_err(|e| {
        PERSONA_TAMPERED
            .with_label_values(&[&handle.display()])
            .inc();
        PERSONA_LOADS
            .with_label_values(&[&handle.display(), "tampered"])
            .inc();
        e
    })?;

    PERSONA_LOADS
        .with_label_values(&[&handle.display(), "hit"])
        .inc();
    Ok(persona.clone())
}

/// Return sorted list of available persona handles.
pub fn available_handles() -> Vec<String> {
    if !REGISTRY_INITIALISED.load(Ordering::SeqCst) {
        return vec![];
    }
    let map = REGISTRY.load();
    let mut handles: Vec<String> = map.keys().map(|h| h.display()).collect();
    handles.sort();
    handles
}

/// Apply the request persona, if present, without emitting the audit row.
///
/// The returned `audit_row` MUST be durably emitted before provider dispatch.
pub fn apply_to_request(
    req: &ChatCompleteRequest,
    tenant_id: &str,
    request_id: &str,
) -> Result<AppliedPersona, PersonaError> {
    let Some(raw_handle) = req.agent_persona.as_deref() else {
        return Ok(AppliedPersona {
            request: req.clone(),
            made_by_genie: None,
            source_hash_hex16: None,
            audit_row: None,
        });
    };

    let handle = PersonaHandle::parse(raw_handle).map_err(|_| PersonaError::UnknownPersona {
        handle: raw_handle.to_string(),
        available: available_handles(),
    })?;
    let persona = load(&handle)?;

    let mut request = req.clone();
    let mut messages = Vec::with_capacity(req.messages.len() + 1);
    messages.push(Message {
        role: "system".to_string(),
        content: persona.body.clone(),
    });
    messages.extend(req.messages.clone());
    request.messages = messages;

    if request.temperature.is_none() {
        request.temperature = persona.llm_hints.temperature;
    }
    if request.max_tokens.is_none() {
        request.max_tokens = persona.llm_hints.max_tokens;
    }

    let made_by_genie = MadeByGenie {
        id: persona.handle.id.0.clone(),
        version: persona.handle.version.to_string(),
    };
    let source_hash_hex16 = hash::hex16(&persona.source_hash);
    let audit_row = memory_writer::builders::persona_loaded(
        tenant_id,
        &persona.handle.id.0,
        &persona.handle.version.to_string(),
        &persona.handle.display(),
        &persona.source_path,
        persona.source_hash,
        request_id,
    );

    Ok(AppliedPersona {
        request,
        made_by_genie: Some(made_by_genie),
        source_hash_hex16: Some(source_hash_hex16),
        audit_row: Some(audit_row),
    })
}

/// Reload the registry from disk (called by the file watcher).
///
/// On parse error, the cache is left unchanged and a metric is emitted.
pub fn reload(persona_dir: &PathBuf) {
    if !REGISTRY_INITIALISED.load(Ordering::SeqCst) {
        tracing::warn!("persona reload skipped; registry not initialised");
        return;
    }

    let result = load_map(persona_dir);

    match result {
        Ok(new_map) => {
            REGISTRY_SIZE.set(new_map.len() as i64);
            REGISTRY.store(Arc::new(new_map.clone()));
            PERSONA_RELOADS.with_label_values(&["success"]).inc();
            for (h, p) in &new_map {
                tracing::info!(
                    handle = %h.display(),
                    source_hash = %hash::hex16(&p.source_hash),
                    registry_size = new_map.len(),
                    "persona_reloaded"
                );
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "persona reload failed; cache unchanged");
            let outcome = match &e {
                PersonaInitError::Schema { .. }
                | PersonaInitError::ForbiddenFrontmatterField { .. } => "parse_error",
                PersonaInitError::FilenameMismatch { .. } => "filename_mismatch",
                PersonaInitError::IoError { .. } | PersonaInitError::AlreadyInitialised => "error",
            };
            PERSONA_RELOADS.with_label_values(&[outcome]).inc();
        }
    }
}

/// Reset the registry for testing. Only safe in single-threaded test contexts.
pub fn reset_for_tests() {
    REGISTRY.store(Arc::new(HashMap::new()));
    REGISTRY_INITIALISED.store(false, Ordering::SeqCst);
    REGISTRY_SIZE.set(0);
}

fn load_map(
    persona_dir: &PathBuf,
) -> Result<HashMap<PersonaHandle, Arc<Persona>>, PersonaInitError> {
    let mut map: HashMap<PersonaHandle, Arc<Persona>> = HashMap::new();

    if persona_dir.exists() {
        let entries = std::fs::read_dir(persona_dir).map_err(|e| PersonaInitError::IoError {
            path: persona_dir.display().to_string(),
            reason: e.to_string(),
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.extension().map(|e| e == "md").unwrap_or(false) {
                continue;
            }
            let path_str = path.display().to_string();
            let raw = std::fs::read_to_string(&path).map_err(|e| PersonaInitError::IoError {
                path: path_str.clone(),
                reason: e.to_string(),
            })?;
            let persona = parse::parse_persona_md(&path_str, &raw)?;
            map.insert(persona.handle.clone(), Arc::new(persona));
        }
    }

    Ok(map)
}
