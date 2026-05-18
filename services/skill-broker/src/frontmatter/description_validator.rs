//! Description-format validator per FR-SKILL-111 §3 (SKB-020..023).
//!
//! Enforces: 80-1024 chars · no XML brackets · ≥1 verb stem · ≥2 quoted
//! trigger phrases (paraphrase-distinct check is operator-attested, not
//! enforced here).

use once_cell::sync::Lazy;
use regex::Regex;

use super::schema::{DESCRIPTION_MAX_LEN, DESCRIPTION_MIN_LEN};

/// Verb stems indicating concrete action — conservative list, expand via PR.
static VERB_STEMS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(generate|author|audit|review|draft|emit|build|propose|render|extract|classify|tag|score|track|enforce|validate|orchestrate|chain|select|pin|halt|resume|escalate|wrap|publish|deliver|test|simulate)\b",
    )
    .unwrap()
});

/// Quoted trigger phrase: `"<phrase>"` with 1-80 non-quote chars body.
static QUOTED_TRIGGER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""([^"]{1,80})""#).unwrap()
});

/// Negative-trigger preamble: matches "Do NOT use for ..." prefix.
static NEGATIVE_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bdo\s+not\s+use\s+(for|when|with)\b").unwrap()
});

#[derive(Debug, PartialEq)]
pub enum DescriptionViolation {
    TooShort { len: usize },
    TooLong { len: usize },
    ForbiddenBrackets,
    MissingWhat,
    InsufficientTriggers { found: usize, needed: usize },
}

pub fn validate(description: &str) -> Result<(), DescriptionViolation> {
    // Flatten YAML-folded multi-line for length measurement.
    let flat = description.replace('\n', " ").trim().to_string();
    let len = flat.chars().count();
    if len < DESCRIPTION_MIN_LEN {
        return Err(DescriptionViolation::TooShort { len });
    }
    if len > DESCRIPTION_MAX_LEN {
        return Err(DescriptionViolation::TooLong { len });
    }

    if flat.contains('<') || flat.contains('>') {
        return Err(DescriptionViolation::ForbiddenBrackets);
    }

    if !VERB_STEMS.is_match(&flat) {
        return Err(DescriptionViolation::MissingWhat);
    }

    // Count quoted phrases, excluding those preceded by "Do NOT use for"
    let mut positive_triggers = 0usize;
    for m in QUOTED_TRIGGER.find_iter(&flat) {
        let preceding = &flat[..m.start()];
        let window_start = preceding.len().saturating_sub(40);
        let window = &preceding[window_start..];
        if NEGATIVE_PREFIX.is_match(window) {
            continue;
        }
        positive_triggers += 1;
    }
    if positive_triggers < 2 {
        return Err(DescriptionViolation::InsufficientTriggers {
            found: positive_triggers,
            needed: 2,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_description() {
        let d = r#"Generate a feature_request@1 markdown from PRDs. Use when user asks to "draft an FR" or "turn this PRD into a backlog". Outputs versioned FR-NNN-slug.md files with anti-fabrication discipline."#;
        assert!(validate(d).is_ok());
    }

    #[test]
    fn too_short() {
        let d = r#"Generate FRs. Use "draft" or "audit"."#;
        assert!(matches!(validate(d), Err(DescriptionViolation::TooShort { .. })));
    }

    #[test]
    fn forbidden_brackets() {
        let d = r#"Generate <FR> markdowns. Use when user asks to "draft" or "audit". This is enough characters to clear the 80-char minimum length."#;
        assert_eq!(validate(d).unwrap_err(), DescriptionViolation::ForbiddenBrackets);
    }

    #[test]
    fn missing_what_verb() {
        let d = r#"Helps with FRs in the backlog. Useful when user says "FR" or "backlog" or "story". Returns markdown output."#;
        assert_eq!(validate(d).unwrap_err(), DescriptionViolation::MissingWhat);
    }

    #[test]
    fn insufficient_triggers_single_positive() {
        let d = r#"Generate FRs from a PRD source. Use when user asks to "draft an FR". Outputs versioned files in a structured backlog directory under output_dir for the team."#;
        assert!(matches!(
            validate(d),
            Err(DescriptionViolation::InsufficientTriggers { found: 1, needed: 2 })
        ));
    }

    #[test]
    fn negative_trigger_does_not_count() {
        let d = r#"Generate FRs from a PRD source. Use when user asks to "draft an FR". Do NOT use for "audit existing FRs". Outputs versioned FR-NNN-slug.md files."#;
        assert!(matches!(
            validate(d),
            Err(DescriptionViolation::InsufficientTriggers { found: 1, needed: 2 })
        ));
    }

    #[test]
    fn two_positive_plus_one_negative_accepts() {
        let d = r#"Generate FRs from a PRD source. Use when user asks to "draft an FR" or "turn this PRD into a backlog". Do NOT use for "audit existing FRs". Outputs versioned files."#;
        assert!(validate(d).is_ok());
    }
}
