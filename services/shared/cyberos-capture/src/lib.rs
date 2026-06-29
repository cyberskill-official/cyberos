//! `cyberos-capture` (FR-MEMORY-122) — the shared mechanism that turns a module's domain event into an
//! FR-MEMORY-121 interaction-event and records it on the BRAIN's hash-chained audit log, consent-gated and
//! best-effort. AUTH and CHAT depend on this today; PROJ/EMAIL/APP/MCP add their own `capture.rs` against
//! the same contract as they come online (DEC-2714).
//!
//! What this crate is:
//!   * [`Capturer`] — holds the brain audit pool + the consent gate; `capture(ev)` writes a built event via
//!     FR-MEMORY-121 `emit()`. The ONE write path; nothing constructs an audit row by hand.
//!   * [`gate::SqlConsentGate`] — the REAL consent gate (DEC-2712): a SQL read of FR-EVAL-001's
//!     `monitoring_notice` + `subject_acknowledgment` tables. Wrapped in FR-MEMORY-121's `CachingGate` by
//!     [`gate::build_default`]. Deliberately queries the eval TABLES, not the eval binary, so AUTH/CHAT
//!     never link eval (§1 #16).
//!   * [`CaptureEmitter`] — the per-module convention (each module owns a thin `capture.rs`).
//!
//! ## The default-OFF safety flag (FR-MEMORY-122 hard rule)
//!
//! Every emitter is gated behind the process env flag `CAPTURE_ENABLED`. [`capture_enabled`] is the single
//! reader: capture is ON only when it is set to a truthy value (`1`/`true`/`yes`/`on`, case-insensitive).
//! It defaults to **false**. When false, modules build no `Capturer` (`AppState.capturer = None`) and every
//! emitter helper is a no-op that touches nothing — so deploying this code during a live team test changes
//! NOTHING. Even when ON, a row is written only if the subject has also acknowledged the notice (the
//! consent gate). Both conditions must hold: `CAPTURE_ENABLED=true` AND an acknowledgment on file.
//!
//! ## Building the module-side capturer
//!
//! A module calls [`maybe_capturer`] at startup with its audit pool (the brain DB). It returns
//! `Some(Capturer)` only when BOTH `CAPTURE_ENABLED` is truthy AND a pool is present; otherwise `None`.
//! That `Option<Capturer>` lives in the module's `AppState`; the per-module helpers take `Option<&Capturer>`
//! and return early on `None`.

pub mod emitter;
pub mod gate;

pub use emitter::{outcome_label, CaptureEmitter, Capturer};
pub use gate::{build_default as build_default_gate, SqlConsentGate};

// Re-export the FR-MEMORY-121 event surface emitters need, so a module's `capture.rs` imports from
// `cyberos_capture` alone and never has to also reach into `cyberos_memory::interaction` for the builder.
pub use cyberos_memory::interaction::{
    ContentRef, EmitError, EmitOutcome, EventClass, InteractionEvent, Module, SourceChannel, TargetRef,
};

use sqlx::PgPool;

/// The default-OFF capture flag (FR-MEMORY-122). Reads the `CAPTURE_ENABLED` env var; returns `true` ONLY
/// for an explicit truthy value (`1`, `true`, `yes`, `on`, any case). Absent / empty / anything else =>
/// `false`. This is the single source of truth for "is capture on in this process?" so AUTH and CHAT agree.
pub fn capture_enabled() -> bool {
    matches!(
        std::env::var("CAPTURE_ENABLED")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Build the module's optional capturer. Returns `Some(Capturer)` iff capture is enabled
/// ([`capture_enabled`]) AND a brain audit pool is available; otherwise `None`. A module stores the result
/// in its `AppState` and hands `Option<&Capturer>` to its emitter helpers. When this is `None` — the
/// default, and the state during the team load-test — capture is a complete no-op.
///
/// `audit_pool` is the brain/governance Postgres (the one holding `l1_audit_log`, `monitoring_notice`, and
/// `subject_acknowledgment`). For AUTH that is its own `pg` pool (auth + memory share a DB); for CHAT it is
/// the pool opened from `CHAT_AUDIT_DATABASE_URL` (the chat->brain link, DEC-2713).
pub fn maybe_capturer(audit_pool: Option<PgPool>) -> Option<Capturer> {
    match (capture_enabled(), audit_pool) {
        (true, Some(pool)) => {
            tracing::info!(
                target: "cyberos_capture",
                "CAPTURE_ENABLED is on and an audit pool is configured — interaction capture is ACTIVE \
                 (subjects still gated on notice acknowledgment)"
            );
            Some(Capturer::new(pool))
        }
        (true, None) => {
            tracing::info!(
                target: "cyberos_capture",
                "CAPTURE_ENABLED is on but no audit pool is configured — capture stays OFF (no brain link)"
            );
            None
        }
        (false, _) => {
            tracing::info!(
                target: "cyberos_capture",
                "CAPTURE_ENABLED is off (default) — interaction capture is a no-op this process"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // capture_enabled reads a process-global env var; these cases mutate and restore it. Kept in one test
    // so they do not race each other under the default multi-threaded test runner.
    #[test]
    fn capture_enabled_defaults_off_and_honours_truthy_values() {
        let prev = std::env::var("CAPTURE_ENABLED").ok();

        std::env::remove_var("CAPTURE_ENABLED");
        assert!(!capture_enabled(), "absent => off (the safe default)");

        for v in ["1", "true", "TRUE", "Yes", "on"] {
            std::env::set_var("CAPTURE_ENABLED", v);
            assert!(capture_enabled(), "{v:?} must read as on");
        }
        for v in ["0", "false", "no", "off", "", "maybe"] {
            std::env::set_var("CAPTURE_ENABLED", v);
            assert!(!capture_enabled(), "{v:?} must read as off");
        }

        // maybe_capturer is None whenever the flag is off, regardless of pool presence (no pool here).
        std::env::remove_var("CAPTURE_ENABLED");
        assert!(maybe_capturer(None).is_none());

        match prev {
            Some(v) => std::env::set_var("CAPTURE_ENABLED", v),
            None => std::env::remove_var("CAPTURE_ENABLED"),
        }
    }
}
