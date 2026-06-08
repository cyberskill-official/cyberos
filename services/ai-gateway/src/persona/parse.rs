//! FR-AI-014 — Persona Markdown parser (YAML frontmatter + body extraction).

use super::hash;
use super::types::{LlmHints, Persona, PersonaHandle, PersonaInitError};

/// Frontmatter shape for YAML deserialization.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PersonaFrontmatter {
    id: String,
    version: String,
    #[serde(default)]
    allowed_tools: Vec<String>,
    #[serde(default)]
    traits: Vec<String>,
    #[serde(default)]
    llm_hints: LlmHintsRaw,
    /// Forbidden field — if present, reject at parse time (§1 #1).
    system_prompt: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmHintsRaw {
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    #[serde(default)]
    stop_sequences: Vec<String>,
}

/// Parse a persona Markdown file with YAML frontmatter + body.
///
/// The body (everything below the closing `---`) is the canonical system prompt.
/// Frontmatter `system_prompt` is forbidden (§1 #1).
pub fn parse_persona_md(path: &str, raw: &str) -> Result<Persona, PersonaInitError> {
    // 1. Split frontmatter from body
    let (frontmatter_yaml, body_raw) = split_frontmatter(raw, path)?;

    // 2. Parse YAML frontmatter
    let fm: PersonaFrontmatter =
        serde_yaml::from_str(frontmatter_yaml).map_err(|e| PersonaInitError::Schema {
            path: path.into(),
            reason: e.to_string(),
        })?;

    // 3. Forbid 'system_prompt' in frontmatter
    if fm.system_prompt.is_some() {
        return Err(PersonaInitError::ForbiddenFrontmatterField { path: path.into() });
    }

    // 4. Canonicalise body
    let body = canonicalise_body(body_raw);

    // 5. Compute source_hash
    let source_hash = hash::sha256(body.as_bytes());

    // 6. Parse handle from frontmatter id + version using the same strict
    // rules as request-time handles.
    let handle = PersonaHandle::parse(&format!("{}@{}", fm.id, fm.version)).map_err(|e| {
        PersonaInitError::Schema {
            path: path.into(),
            reason: e.to_string(),
        }
    })?;

    // 7. Assert filename matches handle
    let expected_filename = format!("{}.md", handle.display());
    if !path.ends_with(&expected_filename) {
        return Err(PersonaInitError::FilenameMismatch {
            path: path.into(),
            handle: handle.display(),
        });
    }

    Ok(Persona {
        handle,
        body,
        allowed_tools: fm.allowed_tools,
        traits: fm.traits,
        llm_hints: LlmHints {
            temperature: fm.llm_hints.temperature,
            max_tokens: fm.llm_hints.max_tokens,
            stop_sequences: fm.llm_hints.stop_sequences,
        },
        source_path: path.into(),
        source_hash,
    })
}

/// Split raw content into (frontmatter_yaml, body) using `---` markers.
fn split_frontmatter<'a>(raw: &'a str, path: &str) -> Result<(&'a str, &'a str), PersonaInitError> {
    let trimmed = raw.trim_start_matches('\u{FEFF}'); // strip BOM

    let Some(after_first) = trimmed.strip_prefix("---") else {
        return Err(PersonaInitError::Schema {
            path: path.into(),
            reason: "missing opening '---' frontmatter marker".into(),
        });
    };

    // Skip the newline immediately after the opening `---` so we don't
    // match it as the start of the closing `\n---` marker.
    let after_opening_nl = after_first
        .strip_prefix('\n')
        .or_else(|| after_first.strip_prefix("\r\n"))
        .unwrap_or(after_first);

    // Find the closing `---`
    let close_pos = after_opening_nl
        .find("\n---")
        .or_else(|| after_opening_nl.find("\r\n---"))
        .ok_or_else(|| PersonaInitError::Schema {
            path: path.into(),
            reason: "missing closing '---' frontmatter marker".into(),
        })?;

    let frontmatter_yaml = &after_opening_nl[..close_pos];
    // Skip past the closing `\n---` (4 chars) and consume the line-ending
    // newline plus an optional blank-line separator before the body.
    let body_start = close_pos + 4; // "\n---".len()
    let rest = &after_opening_nl[body_start..];
    let body_raw = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .map(|after_nl| {
            after_nl
                .strip_prefix("\r\n")
                .or_else(|| after_nl.strip_prefix('\n'))
                .unwrap_or(after_nl)
        })
        .unwrap_or(rest);

    Ok((frontmatter_yaml, body_raw))
}

/// §1 #8 canonicalisation: CRLF→LF, no BOM, NFC, trim trailing whitespace, single terminating LF.
pub fn canonicalise_body(raw: &str) -> String {
    // Strip BOM if present
    let stripped = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    // CRLF → LF, standalone CR → LF
    let lf = stripped.replace("\r\n", "\n").replace('\r', "\n");
    // NFC normalisation
    let nfc: String = unicode_normalization::UnicodeNormalization::nfc(lf.chars()).collect();
    // Trim trailing whitespace on each line
    let trimmed_lines: Vec<&str> = nfc.lines().map(|l| l.trim_end()).collect();
    let mut out = trimmed_lines.join("\n");
    // Ensure exactly one terminating LF
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalisation_is_lf_normalised() {
        let lf = "Hello\nWorld\n";
        let crlf = "Hello\r\nWorld\r\n";
        assert_eq!(canonicalise_body(lf), canonicalise_body(crlf));
    }

    #[test]
    fn canonicalisation_strips_bom_and_nfc_normalises() {
        let bom_crlf_combining = "\u{FEFF}cafe\u{0301}\r\n";
        let lf_precomposed = "café\n";
        assert_eq!(
            canonicalise_body(bom_crlf_combining),
            canonicalise_body(lf_precomposed)
        );
    }

    #[test]
    fn canonicalisation_trims_trailing_whitespace() {
        let input = "line1   \nline2\t\n";
        assert_eq!(canonicalise_body(input), "line1\nline2\n");
    }

    #[test]
    fn canonicalisation_adds_terminating_lf() {
        let input = "no newline at end";
        assert_eq!(canonicalise_body(input), "no newline at end\n");
    }

    #[test]
    fn split_frontmatter_valid() {
        let raw = "---\nid: test\nversion: 0.1.0\n---\n\nBody here\n";
        let (fm, body) = split_frontmatter(raw, "test.md").unwrap();
        assert_eq!(fm, "id: test\nversion: 0.1.0");
        assert_eq!(body, "Body here\n");
    }

    #[test]
    fn split_frontmatter_missing_markers() {
        assert!(split_frontmatter("no markers here", "test.md").is_err());
        assert!(split_frontmatter("---\nno closing", "test.md").is_err());
    }

    #[test]
    fn parse_valid_persona() {
        let raw = "---\nid: cuo-cpo\nversion: 0.4.1\nallowed_tools:\n  - search_kb\ntraits:\n  - concise\nllm_hints:\n  temperature: 0.4\n---\n\nYou are Genie.\n";
        let p = parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap();
        assert_eq!(p.handle.display(), "cuo-cpo@0.4.1");
        assert_eq!(p.allowed_tools, vec!["search_kb"]);
        assert_eq!(p.traits, vec!["concise"]);
        assert_eq!(p.llm_hints.temperature, Some(0.4));
        assert!(p.body.contains("You are Genie."));
    }

    #[test]
    fn parse_rejects_forbidden_system_prompt_field() {
        let raw = "---\nid: cuo-cpo\nversion: 0.4.1\nsystem_prompt: forbidden\n---\n\nbody\n";
        let err = parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap_err();
        assert!(matches!(
            err,
            PersonaInitError::ForbiddenFrontmatterField { .. }
        ));
    }

    #[test]
    fn parse_rejects_filename_mismatch() {
        let raw = "---\nid: cuo-cpo\nversion: 0.4.2\n---\n\nbody\n";
        let err = parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", raw).unwrap_err();
        assert!(matches!(err, PersonaInitError::FilenameMismatch { .. }));
    }

    #[test]
    fn parse_rejects_invalid_semver() {
        let raw = "---\nid: cuo-cpo\nversion: 0.4\n---\n\nbody\n";
        let err = parse_persona_md("memories/personas/cuo-cpo@0.4.md", raw).unwrap_err();
        assert!(matches!(err, PersonaInitError::Schema { .. }));
    }
}
