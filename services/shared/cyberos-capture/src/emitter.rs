//! TASK-MEMORY-122 §1 #1, #2, #12 — the shared capture mechanism.
//!
//! [`Capturer`] is the one path every module uses to record a work-interaction: it holds the brain audit
//! pool + the consent gate, and `capture(ev)` builds nothing of its own — it takes an already-built
//! TASK-MEMORY-121 [`InteractionEvent`] and writes it via TASK-MEMORY-121 `emit()` (validated, consent-gated,
//! chained as an aux row on `l1_audit_log`). No module ever constructs an `l1_audit_log` row directly
//! (disallowed_tools); `emit()` is the only door, so the consent gate cannot be skipped (DEC-2712).
//!
//! **Best-effort, never on the critical path (§1 #7).** `capture` returns the TASK-MEMORY-121
//! `EmitOutcome`/`EmitError`; callers match it for metrics and swallow errors. A capture failure (pool
//! down, validation error, gated subject) MUST NOT fail or delay the sign-in / message send. Modules wire
//! the emit so it cannot delay the response (after the response is built, or in a spawned task).
//!
//! **Reconciliation with the task skeleton.** TASK-MEMORY-122 §3 sketched `emit(&self.pool, ev)` (two args),
//! but the shipped TASK-MEMORY-121 `emit(pool, ev, gate)` takes the gate explicitly so the gate is a visible
//! dependency, not a hidden global. The `Capturer` therefore *owns* the gate (built once, wrapped in the
//! TASK-MEMORY-121 `CachingGate`) and passes it into `emit` on every call — same effect, gate dependency
//! made explicit. See [`crate::gate::build_default`].

use std::sync::Arc;

use cyberos_memory::interaction::{emit, ConsentGate, EmitError, EmitOutcome, InteractionEvent};
use sqlx::PgPool;

/// The per-emitter outcome label (TASK-MEMORY-122 §1 #12): the value of `outcome` on
/// `memory_capture_emitter_calls_total{module, event_type, outcome}`. Derived from the TASK-MEMORY-121
/// `EmitOutcome`/`EmitError` so a per-module view of live capture (recorded vs gated vs error) is available
/// in addition to TASK-MEMORY-121's own counters.
pub fn outcome_label(result: &Result<EmitOutcome, EmitError>) -> &'static str {
    match result {
        Ok(EmitOutcome::Recorded { .. }) => "recorded",
        Ok(EmitOutcome::Skipped { .. }) => "skipped_consent",
        Err(EmitError::Invalid(_)) => "invalid",
        Err(EmitError::Db(_)) => "emit_error",
    }
}

/// The shared capture mechanism. Cheaply clonable (`PgPool` + `Arc` gate), so it sits in each module's
/// `AppState` and is cloned per request without cost.
#[derive(Clone)]
pub struct Capturer {
    pool: PgPool,
    gate: Arc<dyn ConsentGate>,
}

impl Capturer {
    /// Build a `Capturer` over `pool` (the brain audit DB — the one holding `l1_audit_log`) with the
    /// production consent gate ([`crate::gate::build_default`], the SQL ledger read wrapped in the
    /// TASK-MEMORY-121 `CachingGate`). Both the audit writes and the gate reads go to the same governance
    /// deployment, so one pool serves both.
    pub fn new(pool: PgPool) -> Self {
        let gate = crate::gate::build_default(pool.clone());
        Self {
            pool,
            gate: Arc::new(gate),
        }
    }

    /// Build a `Capturer` with an explicit, already-wrapped gate. Used by tests (e.g. an `AllowAll`/
    /// `DenyAll` gate, or a custom-TTL `CachingGate`) so the happy path can be exercised without standing
    /// up the governance tables.
    pub fn with_gate(pool: PgPool, gate: Arc<dyn ConsentGate>) -> Self {
        Self { pool, gate }
    }

    /// Record a built interaction-event. Best-effort: returns the TASK-MEMORY-121 outcome so the caller can
    /// emit the `memory_capture_emitter_calls_total` metric, then swallow. The underlying interaction
    /// (sign-in, message send) MUST proceed regardless of what this returns. The consent gate is consulted
    /// inside `emit` for subject events (system actors are exempt there); there is no way to capture a
    /// subject who has not acknowledged.
    pub async fn capture(&self, ev: &InteractionEvent) -> Result<EmitOutcome, EmitError> {
        emit(&self.pool, ev, self.gate.as_ref()).await
    }

    /// Record a built event and emit the per-emitter metric in one call — the convenience every module's
    /// `capture.rs` uses so the metric label set (`module`, `event_type`, `outcome`) is identical
    /// everywhere. Always swallows the result (logs on error); never returns it, never blocks meaningfully.
    pub async fn capture_metered(&self, ev: &InteractionEvent) {
        let module = ev.module_str();
        let event_type = ev.event_type.clone();
        let result = self.capture(ev).await;
        let outcome = outcome_label(&result);
        // §1 #12 — memory_capture_emitter_calls_total{module, event_type, outcome}. Structured tracing
        // event (this workspace's metrics path is OTel via cyberos-obs-sdk, matching emit.rs); the obs
        // pipeline derives the counter from these fields.
        match &result {
            Ok(EmitOutcome::Recorded { .. }) | Ok(EmitOutcome::Skipped { .. }) => {
                tracing::info!(
                    target: "cyberos_capture::emitter",
                    metric = "memory_capture_emitter_calls_total",
                    module = module,
                    event_type = %event_type,
                    outcome = outcome,
                    "capture emitter call"
                );
            }
            Err(e) => {
                // Best-effort: a capture failure is logged, never propagated to the interaction.
                tracing::warn!(
                    target: "cyberos_capture::emitter",
                    metric = "memory_capture_emitter_calls_total",
                    module = module,
                    event_type = %event_type,
                    outcome = outcome,
                    error = %e,
                    "capture emitter call failed (best-effort; interaction unaffected)"
                );
            }
        }
    }
}

/// TASK-MEMORY-122 §1 #2 / DEC-2714 — the per-module emitter convention. A module that captures holds a
/// `Capturer` (directly in its `AppState`, or reachable from it) and exposes a thin `capture.rs` of typed
/// helpers (`emit_signed_in(..)`, `emit_message_created(..)`, ...) that translate that module's domain
/// event into a built `InteractionEvent` and call [`Capturer::capture_metered`]. This trait names that
/// "the module can reach its `Capturer`" contract so the shape is stated once; new modules (PROJ, EMAIL,
/// APP, MCP) implement it and add a `capture.rs`, not a new pattern.
pub trait CaptureEmitter {
    /// The capturer to record through. `None` when capture is disabled for this process (e.g.
    /// `CAPTURE_ENABLED` unset, or no audit pool configured) — helpers see `None` and no-op.
    fn capturer(&self) -> Option<&Capturer>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use cyberos_memory::interaction::{EmitError, EmitOutcome};

    #[test]
    fn outcome_label_maps_every_arm() {
        assert_eq!(
            outcome_label(&Ok(EmitOutcome::Recorded { seq: 1 })),
            "recorded"
        );
        assert_eq!(
            outcome_label(&Ok(EmitOutcome::Skipped {
                reason: cyberos_memory::interaction::SkipReason::ConsentNotAcknowledged
            })),
            "skipped_consent"
        );
        assert_eq!(
            outcome_label(&Err(EmitError::Invalid("x".into()))),
            "invalid"
        );
        assert_eq!(
            outcome_label(&Err(EmitError::Db(sqlx::Error::PoolClosed))),
            "emit_error"
        );
    }
}
