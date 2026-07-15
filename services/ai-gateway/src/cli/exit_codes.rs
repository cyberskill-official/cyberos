//! TASK-AI-021 §1 #7 — Re-export shared exit codes.
//!
//! Codes 0-7 are stable cross-CLI contract. Module-specific extensions start at 100 (AI).

pub use cyberos_cli_exit::ExitCode;
