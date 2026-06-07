//! FR-AI-009 §3 — Breaker state types.

use crate::policy::ProviderKind;

/// Circuit breaker state. `#[repr(u8)]` for AtomicU8 storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BreakerState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

impl BreakerState {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Closed,
            1 => Self::Open,
            2 => Self::HalfOpen,
            _ => panic!("invalid BreakerState discriminant: {v}"),
        }
    }

    /// Stable string for OBS metric labels — never use Debug-format.
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

/// Outcome of a provider call, consumed by `record_outcome`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallOutcome {
    Success,
    Failure5xx,
    Failure429,
    Timeout,
    ConnectionReset,
    Failure4xx,
}

/// Snapshot of a single breaker's current status.
#[derive(Debug, Clone)]
pub struct BreakerStatus {
    pub provider: ProviderKind,
    pub model: String,
    pub state: BreakerState,
    pub failure_count_window: u32,
    pub last_state_change: SystemTimeUnix,
    pub next_half_open_at: Option<SystemTimeUnix>,
    pub short_circuits_total: u64,
    pub probes_succeeded: u64,
    pub probes_failed: u64,
}

/// Nanoseconds since process-relative epoch (clock start).
pub type SystemTimeUnix = u64;

/// Deterministic transition event emitted whenever a breaker changes state.
///
/// This is the in-process event shape consumed by OBS/audit adapters; tests use it
/// to verify probe-pairing and deterministic transition order without writing to
/// the project BRAIN from the router hot path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakerTransitionEvent {
    pub ts_ns: SystemTimeUnix,
    pub provider: ProviderKind,
    pub model: String,
    pub from: BreakerState,
    pub to: BreakerState,
    pub row_kind: &'static str,
}
