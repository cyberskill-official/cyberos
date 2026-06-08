//! FR-AI-015 §5 — Integration tests for ZDR attestation table.
//!
//! Tests the parse/validation logic and the is_zdr gate.

use std::collections::HashMap;
use std::path::Path;

use chrono::{Duration, Utc};

use cyberos_ai_gateway::policy::ProviderKind;
use cyberos_ai_gateway::zdr::*;

// ─── Parse validation tests ──────────────────────────────────────────────────

#[test]
fn parse_valid_yaml_with_all_fields() {
    let yaml = r#"
version: 1
attestations:
  bedrock:
    "anthropic.claude-3-5-sonnet-20241022-v2:0":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
      notes: "Test note"
"#;
    // Parse directly (not init_zdr_table which uses OnceCell).
    let result = parse_attestations(yaml);
    assert!(result.is_ok());
    let table = result.unwrap();
    assert_eq!(table.len(), 1);
    let att = table
        .get(&(
            ProviderKind::Bedrock,
            "anthropic.claude-3-5-sonnet-20241022-v2:0".into(),
        ))
        .unwrap();
    assert!(att.is_zdr);
    assert_eq!(att.notes.as_deref(), Some("Test note"));
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
    assert!(format!("{}", err).contains("https"));
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
    assert!(format!("{}", err).contains("attested_by"));
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
    assert!(format!("{}", err).contains("attested_by"));
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
    assert!(format!("{}", err).contains("source_url"));
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
    assert!(format!("{}", err).contains("attested_by"));
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
    assert!(format!("{}", err).contains("is_zdr"));
}

#[test]
fn parse_rejects_unknown_provider() {
    let yaml = r#"
version: 1
attestations:
  fakeprovider:
    "model-1":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://example.com"
      attested_by: "stephen@cyberos.world"
"#;
    let err = parse_attestations(yaml).unwrap_err();
    assert!(format!("{}", err).contains("unknown provider"));
}

// ─── Staleness tests ─────────────────────────────────────────────────────────

#[test]
fn soft_stale_at_91_days() {
    let att = ZdrAttestation {
        is_zdr: true,
        verified_at: Utc::now().date_naive() - Duration::days(91),
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
        verified_at: Utc::now().date_naive() - Duration::days(366),
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
        verified_at: Utc::now().date_naive() - Duration::days(30),
        source_url: "https://x".into(),
        attested_by: "stephen@cyberos.world".into(),
        notes: None,
    };
    assert!(!is_soft_stale(&att));
    assert!(!is_hard_stale(&att));
}

#[test]
fn soft_stale_boundary_at_90_days() {
    let att = ZdrAttestation {
        is_zdr: true,
        verified_at: Utc::now().date_naive() - Duration::days(90),
        source_url: "https://x".into(),
        attested_by: "stephen@cyberos.world".into(),
        notes: None,
    };
    // Exactly 90 days is NOT stale (> 90, not >= 90).
    assert!(!is_soft_stale(&att));
}

#[test]
fn init_loads_fixture_and_double_init_rejected() {
    reset_for_tests();
    init_zdr_table(Path::new("config/zdr_attestations.yaml")).unwrap();
    assert!(is_zdr(
        &ProviderKind::Bedrock,
        "anthropic.claude-3-5-sonnet-20241022-v2:0"
    ));
    let err = init_zdr_table(Path::new("config/zdr_attestations.yaml")).unwrap_err();
    assert!(matches!(err, ZdrInitError::AlreadyInitialised));
    reset_for_tests();
}

#[test]
fn missing_entry_fails_closed() {
    reset_for_tests();
    init_zdr_table(Path::new("config/zdr_attestations.yaml")).unwrap();
    assert!(!is_zdr(&ProviderKind::Vertex, "gemini-9.9.9"));
    assert!(attestation_for(&ProviderKind::Vertex, "gemini-9.9.9").is_none());
    reset_for_tests();
}

#[test]
fn hard_stale_override_forces_is_zdr_false() {
    reset_for_tests();
    let mut table = HashMap::new();
    table.insert(
        (ProviderKind::Bedrock, "test-model".to_string()),
        ZdrAttestation {
            is_zdr: true,
            verified_at: Utc::now().date_naive() - Duration::days(366),
            source_url: "https://example.com".into(),
            attested_by: "stephen@cyberos.world".into(),
            notes: None,
        },
    );
    replace_for_tests(table);
    assert!(!is_zdr(&ProviderKind::Bedrock, "test-model"));
    reset_for_tests();
}

#[test]
fn reload_adds_new_attestation_and_detects_revocation() {
    reset_for_tests();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("zdr_attestations.yaml");
    std::fs::write(
        &path,
        r#"
version: 1
attestations:
  bedrock:
    "model-a":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
"#,
    )
    .unwrap();
    init_zdr_table(&path).unwrap();
    assert!(is_zdr(&ProviderKind::Bedrock, "model-a"));

    std::fs::write(
        &path,
        r#"
version: 1
attestations:
  bedrock:
    "model-a":
      is_zdr: false
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
    "model-b":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
"#,
    )
    .unwrap();
    reload_zdr_table(&path).unwrap();
    assert!(!is_zdr(&ProviderKind::Bedrock, "model-a"));
    assert!(is_zdr(&ProviderKind::Bedrock, "model-b"));
    reset_for_tests();
}

#[test]
fn reload_deleted_true_attestation_fails_closed() {
    reset_for_tests();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("zdr_attestations.yaml");
    std::fs::write(
        &path,
        r#"
version: 1
attestations:
  bedrock:
    "model-a":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
"#,
    )
    .unwrap();
    init_zdr_table(&path).unwrap();
    assert!(is_zdr(&ProviderKind::Bedrock, "model-a"));

    std::fs::write(
        &path,
        r#"
version: 1
attestations:
  bedrock:
    "model-b":
      is_zdr: true
      verified_at: 2026-05-21
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
"#,
    )
    .unwrap();
    reload_zdr_table(&path).unwrap();
    assert!(!is_zdr(&ProviderKind::Bedrock, "model-a"));
    assert!(attestation_for(&ProviderKind::Bedrock, "model-a").is_none());
    reset_for_tests();
}
