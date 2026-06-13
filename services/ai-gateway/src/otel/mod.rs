//! FR-AI-022 — OpenTelemetry trace + span emission for every AI Gateway call.

pub mod attributes;
pub mod init;
pub mod pii_lint;
pub mod propagation;
pub mod spans;
