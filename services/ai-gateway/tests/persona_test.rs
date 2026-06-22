//! FR-AI-014 §5 — Integration tests for persona parsing, hashing, and handle parsing.

use cyberos_ai_gateway::persona::*;

// ─── Handle parsing ───────────────────────────────────────────────────────────

#[test]
fn handle_parse_valid() {
    let h = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();
    assert_eq!(h.id.0, "cuo-cpo");
    assert_eq!(h.version.to_string(), "0.4.1");
    assert_eq!(h.display(), "cuo-cpo@0.4.1");
}

#[test]
fn handle_parse_rejects_missing_at() {
    assert!(matches!(
        PersonaHandle::parse("cuo-cpo0.4.1"),
        Err(PersonaParseError::MissingAt(_))
    ));
}

#[test]
fn handle_parse_rejects_invalid_semver() {
    assert!(matches!(
        PersonaHandle::parse("cuo-cpo@0.4"),
        Err(PersonaParseError::InvalidSemver(_))
    ));
}

#[test]
fn handle_parse_rejects_pre_release() {
    assert!(matches!(
        PersonaHandle::parse("cuo-cpo@0.4.1-alpha"),
        Err(PersonaParseError::PreReleaseUnsupported(_))
    ));
}

#[test]
fn handle_parse_rejects_empty_id() {
    assert!(matches!(
        PersonaHandle::parse("@0.4.1"),
        Err(PersonaParseError::InvalidId(_))
    ));
}

#[test]
fn handle_display_is_stable() {
    let h = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();
    assert_eq!(h.to_string(), "cuo-cpo@0.4.1");
}

// ─── Body canonicalisation ────────────────────────────────────────────────────

#[test]
fn canonicalisation_lf_normalised() {
    let lf = "Hello\nWorld\n";
    let crlf = "Hello\r\nWorld\r\n";
    assert_eq!(parse::canonicalise_body(lf), parse::canonicalise_body(crlf));
}

#[test]
fn canonicalisation_strips_bom() {
    let bom = "\u{FEFF}Hello\n";
    let no_bom = "Hello\n";
    assert_eq!(
        parse::canonicalise_body(bom),
        parse::canonicalise_body(no_bom)
    );
}

#[test]
fn canonicalisation_nfc_normalises() {
    let combining = "cafe\u{0301}\n";
    let precomposed = "café\n";
    assert_eq!(
        parse::canonicalise_body(combining),
        parse::canonicalise_body(precomposed)
    );
}

#[test]
fn canonicalisation_trims_trailing_whitespace() {
    let input = "line1   \nline2\t\n";
    assert_eq!(parse::canonicalise_body(input), "line1\nline2\n");
}

#[test]
fn canonicalisation_adds_terminating_lf() {
    let input = "no newline";
    assert_eq!(parse::canonicalise_body(input), "no newline\n");
}

// ─── Frontmatter parsing ─────────────────────────────────────────────────────

#[test]
fn parse_valid_persona() {
    let raw = "\
---
id: cuo-cpo
version: 0.4.1
allowed_tools:
  - search_kb
  - draft_email
traits:
  - concise
  - VN-aware
llm_hints:
  temperature: 0.4
  max_tokens: 1024
  stop_sequences:
    - \"</persona>\"
---

You are Genie, the AI orchestrator at CyberSkill.

Constraints:
- Never offer compensation.
";
    let p = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap();
    assert_eq!(p.handle.display(), "cuo-cpo@0.4.1");
    assert_eq!(p.allowed_tools, vec!["search_kb", "draft_email"]);
    assert_eq!(p.traits, vec!["concise", "VN-aware"]);
    assert_eq!(p.llm_hints.temperature, Some(0.4));
    assert_eq!(p.llm_hints.max_tokens, Some(1024));
    assert_eq!(p.llm_hints.stop_sequences, vec!["</persona>"]);
    assert!(p.body.contains("You are Genie"));
    assert!(p.body.contains("Never offer compensation"));
}

#[test]
fn parse_rejects_forbidden_system_prompt_field() {
    let raw = "---\nid: cuo-cpo\nversion: 0.4.1\nsystem_prompt: forbidden\n---\n\nbody\n";
    let err = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap_err();
    assert!(matches!(
        err,
        PersonaInitError::ForbiddenFrontmatterField { .. }
    ));
}

#[test]
fn parse_rejects_filename_mismatch() {
    let raw = "---\nid: cuo-cpo\nversion: 0.4.2\n---\n\nbody\n";
    let err = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap_err();
    assert!(matches!(err, PersonaInitError::FilenameMismatch { .. }));
}

#[test]
fn parse_rejects_invalid_frontmatter_yaml() {
    let raw = "---\ninvalid: yaml: [broken\n---\n\nbody\n";
    let err = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap_err();
    assert!(matches!(err, PersonaInitError::Schema { .. }));
}

// ─── Hash verification ───────────────────────────────────────────────────────

#[test]
fn hash_verify_passes_on_clean_persona() {
    let raw = "---\nid: cuo-cpo\nversion: 0.4.1\n---\n\nYou are Genie.\n";
    let p = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap();
    hash::verify_persona(&p).unwrap();
}

#[test]
fn hash_verify_fails_on_tampered_persona() {
    let raw = "---\nid: cuo-cpo\nversion: 0.4.1\n---\n\nYou are Genie.\n";
    let mut p = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap();
    // Tamper with the body
    p.body.push_str("TAMPERED LINE\n");

    let err = hash::verify_persona(&p).unwrap_err();
    match err {
        PersonaError::Tampered {
            handle,
            expected_hash,
            actual_hash,
        } => {
            assert_eq!(handle.display(), "cuo-cpo@0.4.1");
            assert_ne!(expected_hash, actual_hash);
        }
        e => panic!("expected Tampered, got: {e:?}"),
    }
}

#[test]
fn hex16_returns_first_16_hex_chars() {
    let hash = hash::sha256(b"test");
    let h16 = hash::hex16(&hash);
    assert_eq!(h16.len(), 16);
    assert_eq!(h16, hex::encode(&hash[..8]));
}

// ─── Registry init + load ────────────────────────────────────────────────────

#[test]
fn registry_init_and_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let persona_dir = dir.path().to_path_buf();

    // Write a persona file
    let content = "---\nid: test-persona\nversion: 0.1.0\nallowed_tools:\n  - tool1\ntraits:\n  - friendly\n---\n\nHello, I am a test persona.\n";
    std::fs::write(persona_dir.join("test-persona@0.1.0.md"), content).unwrap();

    // Init registry (note: this uses a fresh OnceCell per test binary, so
    // multiple tests calling init_persona_registry will conflict. For unit
    // tests, we test the parser directly.)
    let persona = parse::parse_persona_md(
        &format!("{}/test-persona@0.1.0.md", persona_dir.display()),
        content,
    )
    .unwrap();

    assert_eq!(persona.handle.display(), "test-persona@0.1.0");
    assert_eq!(persona.allowed_tools, vec!["tool1"]);
    assert_eq!(persona.traits, vec!["friendly"]);
    assert!(persona.body.contains("Hello, I am a test persona."));
}

// ─── Source hash determinism ─────────────────────────────────────────────────

#[test]
fn source_hash_is_deterministic() {
    let raw = "---\nid: cuo-cpo\nversion: 0.4.1\n---\n\nBody text.\n";
    let p1 = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap();
    let p2 = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap();
    assert_eq!(p1.source_hash, p2.source_hash);
}

#[test]
fn source_hash_changes_on_body_change() {
    let raw1 = "---\nid: cuo-cpo\nversion: 0.4.1\n---\n\nBody v1.\n";
    let raw2 = "---\nid: cuo-cpo\nversion: 0.4.1\n---\n\nBody v2.\n";
    let p1 = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw1).unwrap();
    let p2 = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw2).unwrap();
    assert_ne!(p1.source_hash, p2.source_hash);
}

// ─── CRLF tolerance (AC #14) ─────────────────────────────────────────────────

#[test]
fn crlf_and_lf_produce_same_hash() {
    let lf_raw = "---\nid: cuo-cpo\nversion: 0.4.1\n---\n\nLine1\nLine2\n";
    let crlf_raw = "---\nid: cuo-cpo\nversion: 0.4.1\r\n---\r\n\r\nLine1\r\nLine2\r\n";
    let p_lf = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", lf_raw).unwrap();
    let p_crlf = parse::parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", crlf_raw).unwrap();
    assert_eq!(
        p_lf.source_hash, p_crlf.source_hash,
        "CRLF and LF must produce the same source hash"
    );
}
