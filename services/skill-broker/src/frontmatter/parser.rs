//! SKILL.md parser — extract + deserialize the YAML frontmatter block.
//!
//! Per TASK-SKILL-103 §3:
//! - File MUST begin with `---` on line 1.
//! - Closing `---` MUST exist on its own line.
//! - YAML between fences MUST parse cleanly.
//! - All validators (description, marker, …) run before returning Ok.

use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

use super::schema::SkillFrontmatter;
use super::{description_validator, marker_validator};

/// Matches a YAML block-scalar header — `key: >-`, `key: >+`, `key: |`, etc.
///
/// The trailing `>` / `|` (with optional `-` or `+` chomp modifier) is YAML
/// syntax announcing a folded/literal block scalar that follows. It is NOT
/// an XML bracket and MUST be exempt from the TASK-SKILL-113 / SKB-040 check.
static BLOCK_SCALAR_HEADER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*[^:#\s][^:#]*:\s*[|>][-+]?\s*(#.*)?$").unwrap());

#[derive(Debug, Error)]
pub enum FrontmatterError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("missing frontmatter (no leading '---' delimiter)")]
    MissingFrontmatter,

    #[error("missing closing frontmatter delimiter")]
    MissingClosingDelimiter,

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("invalid description: {0:?}")]
    InvalidDescription(super::description_validator::DescriptionViolation),

    #[error("invalid wrap_in_marker: {0:?}")]
    InvalidMarker(super::marker_validator::MarkerViolation),

    #[error("XML bracket detected in frontmatter value at line {line}: {content}")]
    XmlBracketInFrontmatter { line: usize, content: String },

    #[error("deprecated XML-form field detected (TASK-SKILL-113 — run tools/migrate-wrap-in/migrate.sh --apply): {field}")]
    DeprecatedXmlField { field: String },
}

/// Load a SKILL.md file, parse the frontmatter, and run all validators.
pub fn load_and_validate(bundle: &Path) -> Result<(SkillFrontmatter, String), FrontmatterError> {
    let skill_md = if bundle.is_dir() {
        bundle.join("SKILL.md")
    } else {
        bundle.to_path_buf()
    };
    let text = std::fs::read_to_string(&skill_md)?;

    // 1. Find leading + closing `---` delimiters.
    if !text.starts_with("---\n") {
        return Err(FrontmatterError::MissingFrontmatter);
    }
    let after_open = &text[4..];
    let close_pos = after_open
        .find("\n---\n")
        .or_else(|| after_open.find("\n---"))
        .ok_or(FrontmatterError::MissingClosingDelimiter)?;
    let yaml = &after_open[..close_pos];
    let body_start = close_pos + "\n---\n".len();
    let body = if body_start <= after_open.len() {
        &after_open[body_start.min(after_open.len())..]
    } else {
        ""
    };

    // 2. Defensive scan — reject deprecated `wrap_in: <untrusted_content/>` form.
    //    TASK-SKILL-113 §1 #10 mandates fail-fast on legacy form.
    for (lineno, line) in yaml.lines().enumerate() {
        if line.trim_start().starts_with("wrap_in:") && line.contains('<') {
            return Err(FrontmatterError::DeprecatedXmlField {
                field: format!("wrap_in: (line {}): {}", lineno + 1, line.trim()),
            });
        }
    }

    // 3. Scan for ANY unquoted XML brackets in YAML values (SKB-040).
    //    State machine: track single + double quote context; reject `<` or `>`
    //    outside quoted strings. Exempt YAML block-scalar headers (`key: >-`
    //    etc.) — that `>` is YAML syntax, not an XML bracket.
    for (lineno, line) in yaml.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.is_empty() || trimmed == "---" {
            continue;
        }
        if BLOCK_SCALAR_HEADER.is_match(line) {
            continue;
        }
        if has_unquoted_angle_bracket(line) {
            return Err(FrontmatterError::XmlBracketInFrontmatter {
                line: lineno + 1,
                content: line.to_string(),
            });
        }
    }

    // 4. Deserialize.
    let fm: SkillFrontmatter = serde_yaml::from_str(yaml)?;

    // 5. Validate description (SKB-020..023).
    if let Err(v) = description_validator::validate(&fm.description) {
        return Err(FrontmatterError::InvalidDescription(v));
    }

    // 6. Validate marker (SKB-040..042).
    if let Some(ref ui) = fm.untrusted_inputs {
        if let Err(v) = marker_validator::validate(ui) {
            return Err(FrontmatterError::InvalidMarker(v));
        }
    }

    Ok((fm, body.to_string()))
}

/// Returns true if the line contains `<` or `>` outside a quoted string.
///
/// Handles YAML escape sequences: when inside a quoted string and we see a
/// backslash, skip the next character (so `\"` doesn't prematurely close
/// the quote context).
fn has_unquoted_angle_bracket(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_single || in_double => {
                // Escape sequence — consume the next char without interpreting it.
                let _ = chars.next();
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '<' | '>' if !in_single && !in_double => return true,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_brackets_not_flagged() {
        assert!(!has_unquoted_angle_bracket(
            r#"description: "Use \"<\" carefully""#
        ));
    }

    #[test]
    fn unquoted_brackets_flagged() {
        assert!(has_unquoted_angle_bracket("wrap_in: <untrusted_content/>"));
        assert!(has_unquoted_angle_bracket(
            "description: Author <task> from PRD"
        ));
    }
}
