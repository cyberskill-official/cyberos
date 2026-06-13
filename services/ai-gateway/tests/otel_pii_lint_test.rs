//! FR-AI-022 §1 #6/#15 — OTel attribute-key lint tests.

use std::path::Path;

use cyberos_ai_gateway::otel::pii_lint;

#[test]
fn otel_pii_lint_accepts_gateway_sources() {
    pii_lint::lint_no_unknown_attribute_keys(Path::new("src"))
        .expect("gateway sources should only use approved OTel attribute keys");
}

#[test]
fn otel_pii_lint_rejects_unapproved_attribute_key() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.rs");
    std::fs::write(
        &file,
        r#"fn bad() { let _ = opentelemetry::KeyValue::new("user.email", "x"); }"#,
    )
    .unwrap();

    let failures = pii_lint::lint_no_unknown_attribute_keys(dir.path())
        .expect_err("user.email should not be an approved span attribute");
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].key, "user.email");
}
