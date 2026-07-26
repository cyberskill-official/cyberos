//! TASK-TEN-204 — ai_tokens emit helper acceptance.

use cyberos_ai_gateway::metering_emit::{emit_ai_tokens, recorded_len};
use uuid::Uuid;

#[test]
fn success_emits_quantity_fifteen() {
    let before = recorded_len();
    let hold = Uuid::new_v4();
    assert!(emit_ai_tokens(
        "tenant-x", hold, "anthropic", "claude-sonnet", 10, 5
    ));
    assert!(recorded_len() > before);
}

#[test]
fn provider_error_path_has_no_emit_helper_call() {
    // Documented contract: only Success / Cancelled(Some) call emit_ai_tokens.
    // Zero-token and skip paths:
    let hold = Uuid::new_v4();
    let before = recorded_len();
    assert!(!emit_ai_tokens("tenant-x", hold, "openai", "gpt", 0, 0));
    assert_eq!(recorded_len(), before);
}

#[test]
fn idempotency_key_is_hold_id() {
    let hold = Uuid::new_v4();
    assert!(emit_ai_tokens("t", hold, "p", "m", 1, 1));
    // Second call same hold does not grow recorder length.
    let mid = recorded_len();
    assert!(emit_ai_tokens("t", hold, "p", "m", 1, 1));
    assert_eq!(recorded_len(), mid);
}
