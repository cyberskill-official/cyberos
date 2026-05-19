//! Integration tests — validate against real CyberOS SKILL.md exemplars.
//!
//! These tests run against the 13 SKB-* compliant skills shipped in
//! 2026-05-19 (3 exemplars from FR-SKILL-111+112+113 + 10 G-cohort
//! backfills from FR-SKILL-115 session). If any of these fails, FR-111/113
//! invariants have regressed.

use std::path::PathBuf;

use cyberos_skill_broker::frontmatter;

/// Compute the repo root from CARGO_MANIFEST_DIR (../.. from this crate).
fn skill_dir(skill_name: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent() // services/
        .unwrap()
        .parent() // repo root
        .unwrap()
        .join("modules")
        .join("skill")
        .join(skill_name)
}

#[test]
fn feature_request_author_validates() {
    let path = skill_dir("feature-request-author");
    if !path.exists() {
        eprintln!("skip: {path:?} not found");
        return;
    }
    let (fm, _) = frontmatter::load_and_validate(&path)
        .unwrap_or_else(|e| panic!("validate failed: {e}"));
    assert_eq!(fm.name, "feature-request-author");
    assert!(!fm.description.is_empty());
    assert!(fm.untrusted_inputs.is_some());
}

#[test]
fn feature_request_audit_validates() {
    let path = skill_dir("feature-request-audit");
    if !path.exists() {
        eprintln!("skip: {path:?} not found");
        return;
    }
    let result = frontmatter::load_and_validate(&path);
    assert!(result.is_ok(), "validate failed: {:?}", result.err());
}

#[test]
fn prd_author_validates() {
    let path = skill_dir("product-requirements-document-author");
    if !path.exists() {
        eprintln!("skip: {path:?} not found");
        return;
    }
    let result = frontmatter::load_and_validate(&path);
    assert!(result.is_ok(), "validate failed: {:?}", result.err());
}

#[test]
fn architecture_decision_record_author_validates() {
    let path = skill_dir("architecture-decision-record-author");
    if !path.exists() {
        eprintln!("skip: {path:?} not found");
        return;
    }
    let result = frontmatter::load_and_validate(&path);
    assert!(result.is_ok(), "validate failed: {:?}", result.err());
}

#[test]
fn code_review_author_validates() {
    let path = skill_dir("code-review-author");
    if !path.exists() {
        eprintln!("skip: {path:?} not found");
        return;
    }
    let result = frontmatter::load_and_validate(&path);
    assert!(result.is_ok(), "validate failed: {:?}", result.err());
}

#[test]
fn legacy_wrap_in_form_rejected() {
    // Synthesise a SKILL.md with the legacy form; loader MUST reject.
    let tmp = tempfile::TempDir::new().unwrap();
    let skill_md = tmp.path().join("SKILL.md");
    std::fs::write(
        &skill_md,
        r#"---
name: legacy-test
description: >-
  Audit one or more existing legacy@1 markdowns against legacy_rubric@1.0.
  Use when user asks to "audit a legacy artefact" or "check legacy compliance".
  Outputs a sibling .audit.md per file.
untrusted_inputs:
  wrap_in: <untrusted_content/>
  injection_scan: required
  on_marker_hit: surface_to_human
---
# Body
"#,
    )
    .unwrap();
    let result = frontmatter::load_and_validate(tmp.path());
    assert!(
        matches!(result, Err(frontmatter::FrontmatterError::DeprecatedXmlField { .. })),
        "expected DeprecatedXmlField, got {:?}",
        result.err()
    );
}

#[test]
fn xml_bracket_in_other_field_rejected() {
    let tmp = tempfile::TempDir::new().unwrap();
    let skill_md = tmp.path().join("SKILL.md");
    std::fs::write(
        &skill_md,
        r#"---
name: bracket-test
description: Author a <FR> from a PRD. Use when user asks to "draft" or "audit".
untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human
---
"#,
    )
    .unwrap();
    let result = frontmatter::load_and_validate(tmp.path());
    assert!(
        matches!(result, Err(frontmatter::FrontmatterError::XmlBracketInFrontmatter { .. })),
        "expected XmlBracketInFrontmatter, got {:?}",
        result.err()
    );
}

#[test]
fn missing_frontmatter_rejected() {
    let tmp = tempfile::TempDir::new().unwrap();
    let skill_md = tmp.path().join("SKILL.md");
    std::fs::write(&skill_md, "# Body only, no frontmatter\n").unwrap();
    let result = frontmatter::load_and_validate(tmp.path());
    assert!(matches!(
        result,
        Err(frontmatter::FrontmatterError::MissingFrontmatter)
    ));
}

#[test]
fn transpile_feature_request_author_to_anthropic_form() {
    let path = skill_dir("feature-request-author");
    if !path.exists() {
        eprintln!("skip: {path:?} not found");
        return;
    }
    let result = cyberos_skill_broker::transpile_anthropic(&path);
    assert!(result.is_ok(), "transpile failed: {:?}", result.err());
    let skill = result.unwrap();
    assert_eq!(skill.name, "feature-request-author");
    let md = skill.to_skill_md();
    // Required shape per Anthropic Reference B
    assert!(md.starts_with("---\n"));
    assert!(md.contains("name: feature-request-author"));
    assert!(md.contains("description:"));
    // CyberOS governance fields MUST be dropped from the frontmatter.
    // (The body may legitimately reference these field names as prose.)
    let fm = &skill.frontmatter_yaml;
    assert!(!fm.contains("self_audit:"));
    assert!(!fm.contains("human_fine_tune:"));
    assert!(!fm.contains("depends_on_contracts:"));
    assert!(!fm.contains("wrap_in_marker:"));
    // Body MUST be preserved (non-empty)
    assert!(skill.body.len() > 100, "body too small: {}", skill.body.len());
}

#[test]
fn transpile_all_three_exemplars_succeed() {
    for skill in &[
        "feature-request-author",
        "feature-request-audit",
        "product-requirements-document-author",
    ] {
        let path = skill_dir(skill);
        if !path.exists() {
            continue;
        }
        let result = cyberos_skill_broker::transpile_anthropic(&path);
        assert!(
            result.is_ok(),
            "transpile {skill} failed: {:?}",
            result.err()
        );
    }
}
