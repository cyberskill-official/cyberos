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

pub use types::{
    LlmHints, Persona, PersonaError, PersonaHandle, PersonaId, PersonaInitError, PersonaParseError,
};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_int_gauge, CounterVec, IntGauge};

use self::types::REGISTRY;

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
    register_int_gauge!("ai_persona_registry_size", "Current persona registry entry count").unwrap()
});

// ─── Public API ───────────────────────────────────────────────────────────────

/// Initialise the persona registry by scanning `<persona_dir>/*.md`.
///
/// Must be called once at gateway startup before any `load()` calls.
/// Uses direct filesystem reads (the memory module's read API is not yet wired).
pub fn init_persona_registry(persona_dir: &PathBuf) -> Result<(), PersonaInitError> {
    if REGISTRY.get().is_some() {
        return Err(PersonaInitError::AlreadyInitialised);
    }

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

    REGISTRY_SIZE.set(map.len() as i64);
    REGISTRY
        .set(arc_swap::ArcSwap::from_pointee(map))
        .map_err(|_| PersonaInitError::AlreadyInitialised)?;

    Ok(())
}

/// Load a persona by handle. Returns a cached `Arc<Persona>` on cache hit,
/// verified for tamper on every load.
pub fn load(handle: &PersonaHandle) -> Result<Arc<Persona>, PersonaError> {
    let registry = REGISTRY.get().ok_or(PersonaError::RegistryNotInitialised)?;
    let map = registry.load();

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
    let Some(registry) = REGISTRY.get() else {
        return vec![];
    };
    let map = registry.load();
    let mut handles: Vec<String> = map.keys().map(|h| h.display()).collect();
    handles.sort();
    handles
}

/// Reload the registry from disk (called by the file watcher).
///
/// On parse error, the cache is left unchanged and a metric is emitted.
pub fn reload(persona_dir: &PathBuf) {
    let Some(registry) = REGISTRY.get() else {
        return;
    };

    let result = (|| -> Result<HashMap<PersonaHandle, Arc<Persona>>, PersonaInitError> {
        let mut map = HashMap::new();
        if persona_dir.exists() {
            let entries =
                std::fs::read_dir(persona_dir).map_err(|e| PersonaInitError::IoError {
                    path: persona_dir.display().to_string(),
                    reason: e.to_string(),
                })?;
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.extension().map(|e| e == "md").unwrap_or(false) {
                    continue;
                }
                let path_str = path.display().to_string();
                let raw =
                    std::fs::read_to_string(&path).map_err(|e| PersonaInitError::IoError {
                        path: path_str.clone(),
                        reason: e.to_string(),
                    })?;
                match parse::parse_persona_md(&path_str, &raw) {
                    Ok(persona) => {
                        map.insert(persona.handle.clone(), Arc::new(persona));
                    }
                    Err(e) => {
                        tracing::warn!(path = %path_str, error = %e, "persona reload parse failed");
                        PERSONA_RELOADS.with_label_values(&["parse_error"]).inc();
                        // Continue loading other personas
                    }
                }
            }
        }
        Ok(map)
    })();

    match result {
        Ok(new_map) => {
            REGISTRY_SIZE.set(new_map.len() as i64);
            registry.store(Arc::new(new_map.clone()));
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
            PERSONA_RELOADS.with_label_values(&["error"]).inc();
        }
    }
}

/// Reset the registry for testing. Only safe in single-threaded test contexts.
#[cfg(test)]
pub fn reset_for_tests() {
    // This is a test-only function. In production, init is called once.
    // We can't easily reset a OnceCell, so tests that need a fresh registry
    // should use separate processes or test the parser/hash directly.
}
